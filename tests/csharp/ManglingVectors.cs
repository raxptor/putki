// Runs tests/fixtures/mangling-vectors.txt through Putki.LocMangling. The Rust
// suite runs the same file through putki-inki, which is what keeps the two
// implementations from drifting -- the extractor writes msgids with one and the
// runtime looks them up with the other.
//
//     mcs -out:mangling.exe ../../runtime/csharp/LocMangling.cs ManglingVectors.cs
//     mono mangling.exe ../fixtures/mangling-vectors.txt

using System;
using System.IO;
using System.Text;
using Putki;

namespace PutkiManglingVectors
{
	class MainClass
	{
		static int failures = 0;

		static void Check(string got, string want, string what)
		{
			if (got != want)
			{
				Console.Error.WriteLine("FAIL: " + what + "\n  got  '" + got + "'\n  want '" + want + "'");
				failures++;
			}
		}

		public static int Main(string[] args)
		{
			if (args.Length < 1)
			{
				Console.Error.WriteLine("usage: mangling.exe <mangling-vectors.txt>");
				return 2;
			}

			int cases = 0;
			foreach (string line in File.ReadAllLines(args[0], Encoding.UTF8))
			{
				if (line.Length == 0 || line[0] == '#')
					continue;
				string[] f = line.Split('\t');
				if (f.Length < 3)
				{
					Console.Error.WriteLine("FAIL: malformed vector line: " + line);
					failures++;
					continue;
				}
				string source = f[0], expectMangled = f[1], expectDisplay = f[2];

				var m = LocMangling.Mangle(source);
				Check(m.Mangled, expectMangled, "mangling [" + source + "]");
				Check(LocMangling.Unmangle(m, false), expectDisplay, "display of [" + source + "]");
				Check(LocMangling.Unmangle(m, true), source, "round trip of [" + source + "]");
				cases++;
			}

			if (cases == 0)
			{
				Console.Error.WriteLine("FAIL: no vectors loaded from " + args[0]);
				return 1;
			}
			if (failures > 0)
			{
				Console.Error.WriteLine(failures + " mangling vector(s) failed");
				return 1;
			}
			Console.WriteLine(cases + " mangling vectors pass");
			return 0;
		}
	}
}
