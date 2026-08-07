//! Localization string mangling.
//!
//! Translatable text often differs only by a number ("Deal 20% damage" /
//! "Deal 30% damage"). Mangling replaces those spans with placeholders drawn
//! from a fixed table, so all such variants collapse to a single catalog entry
//! and rebalancing a number never invalidates a translation:
//!
//! ```text
//! authored   Deal 20% extra damage
//! mangled    Deal {12}% extra damage     <- the msgid translators see
//! restored   Deal 20% extra damage
//! ```
//!
//! Two spans are replaced: `{#text}`, written by hand to hold a piece of text
//! out of translation, and a bare `NN%`, detected automatically and marked
//! `{!NN}`. Any other `{name}` is passed through untouched.
//!
//! **This must stay byte-identical to `runtime/csharp/LocMangling.cs`.** The
//! extractor writes msgids with one implementation and the runtime looks them
//! up with the other; if they disagree, lookups miss and text silently falls
//! back to the source language. `tests/fixtures/mangling-vectors.txt` pins the
//! two together.

/// Placeholder tokens, consumed in order. **Never reorder or extend this**: the
/// values are baked into every msgid in every existing catalog.
pub const SUBST: [&str; 10] = ["12", "34", "56", "78", "90", "15", "20", "25", "30", "35"];

/// A mangled string plus the spans that were taken out of it, in the order the
/// placeholders were consumed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mangled {
	pub text: String,
	pub replacements: Vec<String>,
}

struct Insertion {
	index: usize,
	reference: String,
}

/// Strips every `{...}` span, returning the remainder and where each span sat.
///
/// Indices are byte offsets into the progressively shortened string, which is
/// what makes the re-insertion below work. The braces are ASCII, so the offsets
/// always land on character boundaries.
fn parse_replace(source: &str) -> (String, Vec<Insertion>) {
	let mut rest = String::from(source);
	let mut inserts = Vec::new();
	loop {
		let (beg, end) = match (rest.find('{'), rest.find('}')) {
			(Some(b), Some(e)) => (b, e),
			_ => break,
		};
		// C# would throw on a closing brace before an opening one; treat the
		// malformed remainder as having no further spans instead.
		if end < beg {
			break;
		}
		inserts.push(Insertion {
			index: beg,
			reference: String::from(&rest[beg + 1..end]),
		});
		rest.replace_range(beg..=end, "");
	}
	(rest, inserts)
}

/// Rewrites `source` into its catalog key form.
///
/// # Panics
/// If the string holds more than [`SUBST`] placeholders. The table cannot grow
/// without invalidating existing translations, so this is a data error.
pub fn mangle(source: &str) -> Mangled {
	// Mark bare "NN%" so the number drops out of the key.
	let pieces: Vec<String> = source
		.split(' ')
		.map(|p| {
			let digits = match p.strip_suffix('%') {
				Some(d) if p.len() >= 2 => d,
				_ => return String::from(p),
			};
			// Keep the original text rather than a reparsed number, so "007%"
			// comes back as "007%".
			if digits.parse::<i32>().is_ok() {
				format!("{{!{}}}%", digits)
			} else {
				String::from(p)
			}
		})
		.collect();

	let (mut text, inserts) = parse_replace(&pieces.join(" "));

	let mut replacements = Vec::new();
	let mut ofs = 0;
	for ins in inserts {
		let marked = matches!(ins.reference.chars().next(), Some('#') | Some('!'));
		let token = if marked {
			let token = *SUBST.get(replacements.len()).unwrap_or_else(|| {
				panic!(
					"more than {} localization placeholders in string [{}]; \
					 the substitution table cannot grow without invalidating catalogs",
					SUBST.len(),
					source
				)
			});
			replacements.push(ins.reference);
			token
		} else {
			// Not a marked span; put it back verbatim.
			let put = format!("{{{}}}", ins.reference);
			text.insert_str(ins.index + ofs, &put);
			ofs += put.len();
			continue;
		};
		let put = format!("{{{}}}", token);
		text.insert_str(ins.index + ofs, &put);
		ofs += put.len();
	}

	Mangled { text, replacements }
}

/// Puts the stripped spans back.
///
/// `recover_source` reproduces the authored string, which is what the extractor
/// checks its own round trip against. With it false the markers are dropped and
/// the plain text remains, which is what gets shown to a player.
pub fn unmangle(mangled: &Mangled, recover_source: bool) -> String {
	let (mut out, inserts) = parse_replace(&mangled.text);
	let mut ofs = 0;
	for ins in inserts {
		let mut put_back = format!("{{{}}}", ins.reference);
		for (i, token) in SUBST.iter().enumerate() {
			if ins.reference != *token {
				continue;
			}
			if let Some(repl) = mangled.replacements.get(i) {
				let first = repl.chars().next();
				put_back = if recover_source {
					match first {
						Some('!') => String::from(&repl[1..]),
						_ => format!("{{{}}}", repl),
					}
				} else {
					match first {
						Some('#') | Some('!') => String::from(&repl[1..]),
						_ => format!("{{{}}}", repl),
					}
				};
			}
			break;
		}
		out.insert_str(ins.index + ofs, &put_back);
		ofs += put_back.len();
	}
	out
}

/// Supplies translations for the strings a generated accessor asks for.
///
/// Mirrors `Putki.Translation` in the C# runtime. Putki does not implement this
/// -- the application binds it to whatever catalog it uses.
pub trait Translation {
	fn translate(&self, text: &str, category: &str) -> String;
	fn translate_plural(&self, text: &str, category: &str, plural_n: i32) -> String;
}

/// Returns the source text unchanged, wrapped so an unbound translation is
/// visible on screen rather than silently shipping the source language.
pub struct NoTranslation;

impl Translation for NoTranslation {
	fn translate(&self, text: &str, _category: &str) -> String {
		format!("<{}>", unmangle(&mangle(text), false))
	}
	fn translate_plural(&self, text: &str, _category: &str, plural_n: i32) -> String {
		let s = format!("<{}>", unmangle(&mangle(text), false));
		if plural_n > 1 { s.replace("(s)", "s") } else { s.replace("(s)", "") }
	}
}

/// One translatable string found in the data.
#[derive(Clone, Debug, PartialEq)]
pub struct TranslatableString {
	/// Source text, exactly as authored.
	pub text: String,
	/// Catalog context, from `{Category}` in the typedef.
	pub category: String,
	pub plural: bool,
	/// `Type.Field` the string came from, for the translator comment.
	pub field: String,
	/// Object path the string came from.
	pub path: String,
}

/// Collects every translatable string an object holds, recursing into nested
/// structs and arrays. Implemented by generated code.
pub trait CollectStrings {
	fn collect_strings(&self, path: &str, out: &mut dyn FnMut(TranslatableString));
}
