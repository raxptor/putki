using System;

namespace TestSimple
{
	public class TypeLoader : Putki.TypeLoader
	{
		public object ResolveFromPackage(int type, object obj, Putki.Package pkg)
		{
			return Outki.Loader.TestProj.ResolveFromPackage(type, obj, pkg);
		}

		public object LoadFromPackage(int type, Putki.PackageReader reader)
		{
			return Outki.Loader.TestProj.LoadFromPackage(type, reader);
		}
	}

	class MainClass
	{
		// Usage: simple.exe [project-dir] [package]
		// project-dir defaults to the current directory and must be the one
		// holding data/objs -- i.e. tests/simple.
		public static void Main(string[] args)
		{
			string root = args.Length > 0 ? args[0] : ".";

			Mixki.SourceLoader sl = new Mixki.SourceLoader(
				System.IO.Path.Combine(root, "data", "objs"), Mixki.TestProj.Parsers);
			Outki.Everything sourceEverything = sl.Resolve<Outki.Everything>("everything");
			if (sourceEverything == null)
			{
				Console.Error.WriteLine("failed to resolve 'everything' from source");
				Environment.Exit(1);
			}
			Console.WriteLine("loaded from source: " + sourceEverything);

			// Loading the same data from a built package. Optional: producing one
			// needs a data builder, which this test project does not run.
			if (args.Length < 2)
			{
				return;
			}

			Putki.PackageManager.LoadFromBytes(System.IO.File.ReadAllBytes(args[1]), new TypeLoader());
			Outki.Everything everything = Putki.PackageManager.Resolve<Outki.Everything>("everything");
			if (everything == null)
			{
				Console.Error.WriteLine("failed to resolve 'everything' from " + args[1]);
				Environment.Exit(1);
			}
			Console.WriteLine("loaded from package: " + everything);
		}
	}
}

