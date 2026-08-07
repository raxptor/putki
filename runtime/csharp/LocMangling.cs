using System;
using System.Collections.Generic;
using System.Text;

namespace Putki
{
	/// Localization string mangling. See rust/putki-inki/src/inki/loc.rs for the
	/// rationale -- numeric variants of a string collapse to one catalog entry.
	///
	/// This must stay byte-identical to that Rust implementation: the extractor
	/// writes msgids with one and the runtime looks them up with the other, and a
	/// disagreement shows up as text silently falling back to the source language.
	/// tests/fixtures/mangling-vectors.txt pins the two together.
	public static class LocMangling
	{
		/// Placeholder tokens, consumed in order. Never reorder or extend: these
		/// values are baked into every msgid in every existing catalog.
		public static readonly string[] Subst = new string[10] {
			"12", "34", "56", "78", "90", "15", "20", "25", "30", "35"
		};

		public struct Data
		{
			public string Mangled;
			public List<string> Replacements;
		}

		struct Insertion
		{
			public int Index;
			public string Reference;
		}

		/// Strips every {...} span, returning the remainder and where each sat.
		static string ParseReplace(string source, out List<Insertion> inserts)
		{
			inserts = new List<Insertion>();
			while (true)
			{
				int beg = source.IndexOf('{');
				int end = source.IndexOf('}');
				if (beg == -1 || end == -1 || end < beg)
					break;
				inserts.Add(new Insertion {
					Index = beg,
					Reference = source.Substring(beg + 1, end - beg - 1)
				});
				source = source.Remove(beg, end - beg + 1);
			}
			return source;
		}

		static bool IsMarked(string reference)
		{
			return reference.Length > 0 && (reference[0] == '#' || reference[0] == '!');
		}

		public static Data Mangle(string source)
		{
			if (source == null)
			{
				return new Data { Mangled = "", Replacements = new List<string>() };
			}

			// Mark bare "NN%" so the number drops out of the key.
			string[] pieces = source.Split(' ');
			for (int i = 0; i < pieces.Length; i++)
			{
				string p = pieces[i];
				if (p.Length < 2 || p[p.Length - 1] != '%')
					continue;
				string digits = p.Substring(0, p.Length - 1);
				int parsed;
				// Keep the original text rather than a reparsed number, so "007%"
				// comes back as "007%".
				if (int.TryParse(digits, out parsed))
					pieces[i] = "{!" + digits + "}%";
			}

			List<Insertion> inserts;
			string text = ParseReplace(String.Join(" ", pieces), out inserts);

			List<string> replacements = new List<string>();
			int ofs = 0;
			foreach (var ins in inserts)
			{
				string put;
				if (IsMarked(ins.Reference))
				{
					if (replacements.Count >= Subst.Length)
					{
						// A BCL type on purpose: this file must not depend on the
						// rest of the runtime, so it can be dropped into any project.
						throw new ArgumentException(
							"more than " + Subst.Length + " localization placeholders in string [" + source +
							"]; the substitution table cannot grow without invalidating catalogs");
					}
					put = "{" + Subst[replacements.Count] + "}";
					replacements.Add(ins.Reference);
				}
				else
				{
					// Not a marked span; put it back verbatim.
					put = "{" + ins.Reference + "}";
				}
				text = text.Insert(ins.Index + ofs, put);
				ofs += put.Length;
			}

			return new Data { Mangled = text, Replacements = replacements };
		}

		/// Puts the stripped spans back. recoverSource reproduces the authored
		/// string; with it false the markers are dropped, which is what a player
		/// sees.
		public static string Unmangle(Data mangled, bool recoverSource)
		{
			List<Insertion> inserts;
			string outStr = ParseReplace(mangled.Mangled, out inserts);
			int ofs = 0;
			foreach (var ins in inserts)
			{
				string putBack = "{" + ins.Reference + "}";
				for (int i = 0; i < Subst.Length; i++)
				{
					if (ins.Reference != Subst[i])
						continue;
					if (i < mangled.Replacements.Count)
					{
						string repl = mangled.Replacements[i];
						char first = repl.Length > 0 ? repl[0] : '\0';
						if (recoverSource)
							putBack = first == '!' ? repl.Substring(1) : "{" + repl + "}";
						else
							putBack = (first == '#' || first == '!') ? repl.Substring(1) : "{" + repl + "}";
					}
					break;
				}
				outStr = outStr.Insert(ins.Index + ofs, putBack);
				ofs += putBack.Length;
			}
			return outStr;
		}
	}
}
