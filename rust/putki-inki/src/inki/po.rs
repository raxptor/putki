//! Writes gettext `.pot` catalogs.
//!
//! The escaping here is the point. A hand-rolled writer that only escapes
//! newlines produces a malformed catalog the moment a string contains a quote,
//! and those entries then never reach translators -- silently, because nothing
//! reads the file until msgfmt does.

use std::collections::BTreeMap;

use crate::inki::loc::{mangle, unmangle, TranslatableString};

/// Escapes a string for a `msgid` / `msgstr` value.
///
/// Everything gettext requires: backslash and quote, plus the whitespace that
/// would otherwise break the line.
pub fn escape(text: &str) -> String {
	let mut out = String::with_capacity(text.len() + 8);
	for c in text.chars() {
		match c {
			'\\' => out.push_str("\\\\"),
			'"' => out.push_str("\\\""),
			'\n' => out.push_str("\\n"),
			'\r' => out.push_str("\\r"),
			'\t' => out.push_str("\\t"),
			_ => out.push(c),
		}
	}
	out
}

/// The plural marker carried inline in authored text: `torch(es)` becomes
/// `torch` and `torches`.
const PLURAL_MARK: &str = "(s)";

fn singular(text: &str) -> String {
	text.replace(PLURAL_MARK, "")
}

fn plural(text: &str) -> String {
	text.replace(PLURAL_MARK, "s")
}

#[derive(Default)]
struct Entry {
	plural: bool,
	comments: Vec<String>,
}

/// Accumulates strings and writes them out as a `.pot`.
#[derive(Default)]
pub struct Catalog {
	/// Keyed by (category, mangled text) -- the pair gettext looks up by, so
	/// duplicates collapse exactly the way the runtime resolves them.
	entries: BTreeMap<(String, String), Entry>,
	mismatches: Vec<String>,
}

impl Catalog {
	pub fn new() -> Catalog {
		Default::default()
	}

	/// Adds one string. Repeats of the same key merge their comments.
	///
	/// An empty string is dropped: a field simply left unset in the data is not
	/// something to translate, and an empty `msgid` is the .pot header's own key,
	/// so emitting one produces a file gettext reads as having two headers.
	pub fn add(&mut self, s: &TranslatableString) {
		if s.text.is_empty() {
			return;
		}
		let m = mangle(&s.text);

		// The msgid has to survive the trip back, or the runtime will look up
		// something this never wrote.
		if unmangle(&m, true) != s.text {
			self.mismatches.push(format!(
				"[{}] at {} does not survive mangling: [{}] -> [{}]",
				s.text,
				s.path,
				m.text,
				unmangle(&m, true)
			));
		}

		let entry = self
			.entries
			.entry((s.category.clone(), m.text))
			.or_default();
		entry.plural |= s.plural;
		let comment = format!("{} ({})", s.field, s.path);
		if !entry.comments.contains(&comment) {
			entry.comments.push(comment);
		}
	}

	pub fn len(&self) -> usize {
		self.entries.len()
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	/// Strings that could not be mangled and unmangled back to themselves.
	/// Extraction should fail rather than emit these.
	pub fn mismatches(&self) -> &[String] {
		&self.mismatches
	}

	/// Renders the catalog. `project` goes in the header.
	pub fn to_pot(&self, project: &str) -> String {
		let mut out = String::new();
		out.push_str("#, fuzzy\nmsgid \"\"\nmsgstr \"\"\n");
		out.push_str(&format!("\"Project-Id-Version: {}\\n\"\n", escape(project)));
		out.push_str("\"MIME-Version: 1.0\\n\"\n");
		out.push_str("\"Content-Type: text/plain; charset=UTF-8\\n\"\n");
		out.push_str("\"Content-Transfer-Encoding: 8bit\\n\"\n\n");

		for ((category, text), entry) in &self.entries {
			for c in &entry.comments {
				out.push_str(&format!("#. {}\n", c));
			}
			if !category.is_empty() {
				out.push_str(&format!("msgctxt \"{}\"\n", escape(category)));
			}
			if entry.plural {
				out.push_str(&format!("msgid \"{}\"\n", escape(&singular(text))));
				out.push_str(&format!("msgid_plural \"{}\"\n", escape(&plural(text))));
				out.push_str("msgstr[0] \"\"\nmsgstr[1] \"\"\n");
			} else {
				out.push_str(&format!("msgid \"{}\"\n", escape(text)));
				out.push_str("msgstr \"\"\n");
			}
			out.push('\n');
		}
		out
	}
}
