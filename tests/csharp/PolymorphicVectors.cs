// Reads tests/fixtures/polymorphic.pkg -- written by the Rust pipeline from
// tests/rust/data/main.txt -- through the *generated* C# loader, so the rtti
// type tag and its dispatch are checked across both implementations.
//
// Unlike PackageVectors.cs this does not hand-roll a reader; it links the code
// the compiler emits for tests/rust/types, which is what real projects use.
//
//     compiler.sh tests/rust
//     mcs -out:poly.exe -recurse:'../../runtime/csharp/*.cs' \
//         -recurse:'../rust/_gen/csharp/*.cs' PolymorphicVectors.cs
//     mono poly.exe ../fixtures/polymorphic.pkg

using System;
using System.IO;

namespace PutkiPolymorphicVectors
{
	class TypeLoader : Putki.TypeLoader
	{
		public object ResolveFromPackage(int type, object obj, Putki.Package pkg)
		{
			return Outki.Loader.Test.ResolveFromPackage(type, obj, pkg);
		}

		public object LoadFromPackage(int type, Putki.PackageReader reader)
		{
			return Outki.Loader.Test.LoadFromPackage(type, reader);
		}
	}

	class MainClass
	{
		static int failures = 0;

		static void Check(bool ok, string what)
		{
			if (!ok)
			{
				Console.Error.WriteLine("FAIL: " + what);
				failures++;
			}
		}

		public static int Main(string[] args)
		{
			if (args.Length < 1)
			{
				Console.Error.WriteLine("usage: poly.exe <polymorphic.pkg>");
				return 2;
			}

			Putki.PackageManager.LoadFromBytes(File.ReadAllBytes(args[0]), new TypeLoader());
			Outki.Dialog dlg = Putki.PackageManager.Resolve<Outki.Dialog>("dlg");

			if (dlg == null)
			{
				Console.Error.WriteLine("FAIL: could not resolve 'dlg'");
				return 1;
			}

			Check(dlg.Id == "DIALOG HEJ", "Dialog.Id was '" + dlg.Id + "'");

			// Inline @DlgSay: the tag must survive the round trip and dispatch to
			// the child type, not the base.
			Outki.DlgSay say = dlg.Node1 as Outki.DlgSay;
			Check(say != null, "Node1 should be a DlgSay, got " +
				(dlg.Node1 == null ? "null" : dlg.Node1.GetType().Name));
			if (say != null)
			{
				Check(say.Text == "hej", "DlgSay.Text was '" + say.Text + "'");
				Check(say.Who == 0, "DlgSay.Who was " + say.Who);
				Check(say.Id == "dlgsay", "DlgSay.Id (inherited) was '" + say.Id + "'");
			}

			// A bare @IDlgNode stays the base type.
			Check(dlg.Node2 != null, "Node2 should resolve");
			Check(dlg.Node2 != null && dlg.Node2.GetType() == typeof(Outki.IDlgNode),
				"Node2 should be the base IDlgNode, got " +
				(dlg.Node2 == null ? "null" : dlg.Node2.GetType().Name));

			if (failures > 0)
			{
				Console.Error.WriteLine(failures + " polymorphic vector(s) failed");
				return 1;
			}
			Console.WriteLine("polymorphic vectors pass");
			return 0;
		}
	}
}
