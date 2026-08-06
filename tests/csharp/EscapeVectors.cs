// Escaping vectors from doc/text-format.md, checked against MicroJson.DecodeString.
//
// Standalone on purpose, so it needs nothing but MicroJson.cs:
//
//     csc -out:escapetest.exe ../../runtime/csharp/MicroJson.cs EscapeVectors.cs
//     mono escapetest.exe
//
// or with dotnet, drop both files into a throwaway console project.
//
// C# only reads the text format -- there is no writer on this side -- so only
// decoding is covered here. The Rust and JS suites additionally check encoding.

using System;
using System.Text;
using Putki;

namespace PutkiEscapeVectors
{
	class EscapeVectors
	{
		static int failures = 0;

		// body is what sits between the quotes in the data file.
		static string Decode(string body)
		{
			byte[] bytes = Encoding.UTF8.GetBytes(body);
			return MicroJson.DecodeString(bytes, 0, bytes.Length);
		}

		static void Check(string body, string want)
		{
			string got = Decode(body);
			if (got != want)
			{
				failures++;
				Console.WriteLine("FAIL decode [" + body + "]");
				Console.WriteLine("  got  " + (got == null ? "<null>" : "[" + got + "]"));
				Console.WriteLine("  want " + (want == null ? "<null>" : "[" + want + "]"));
			}
		}

		static void CheckRejected(string body, string why)
		{
			string got = Decode(body);
			if (got != null)
			{
				failures++;
				Console.WriteLine("FAIL " + why + " was accepted, got [" + got + "]");
			}
		}

		public static int Main(string[] args)
		{
			Check("plain", "plain");
			Check("with \\\"quote\\\"", "with \"quote\"");
			Check("back\\\\slash", "back\\slash");
			Check("trailing\\\\", "trailing\\");
			Check("two\\\\\\\\slashes", "two\\\\slashes");
			Check("line\\nbreak", "line\nbreak");
			Check("tab\\there", "tab\there");
			Check("quote\\\"then\\\\", "quote\"then\\");
			Check("\\\\\\\"", "\\\"");
			Check("unicode: åäö", "unicode: åäö");
			Check("", "");

			// Legacy \uXXXX carries bytes of the original UTF-8, so a run
			// reassembles into one char: U+00E5 is C3 A5 in UTF-8.
			Check("\\u00c3\\u00a5", "å");
			Check("pre \\u00c3\\u00a5 post", "pre å post");
			// Newline is newline.
			Check("a\\rb", "a\nb");
			Check("a\r\nb\rc", "a\nb\nc");

			// A lone å is the Latin-1 byte, not UTF-8: malformed legacy data.
			CheckRejected("\\u00e5", "lone latin-1 byte");
			CheckRejected("bad \\q escape", "unknown escape");
			CheckRejected("short \\u00", "truncated \\u");
			CheckRejected("\\uzzzz", "non-hex \\u");
			CheckRejected("trailing backslash \\", "dangling backslash");

			if (failures > 0)
			{
				Console.WriteLine();
				Console.WriteLine(failures + " failure(s)");
				return 1;
			}
			Console.WriteLine("all escape vectors pass");
			return 0;
		}
	}
}
