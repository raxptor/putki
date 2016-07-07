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
		public static void Main(string[] args)
		{
			Mixki.SourceLoader sl = new Mixki.SourceLoader("/Users/dannilsson/git/putki-cleanup/tests/simple/data/objs", Mixki.TestProj.Parsers);
			Outki.Everything sourceEverything = sl.Resolve<Outki.Everything>("everything");
			{
				Console.WriteLine("I loaded from source " + sourceEverything);
			}

			// load built
			Putki.PackageManager.LoadFromBytes(
				System.IO.File.ReadAllBytes("/Users/dannilsson/git/putki-cleanup/tests/simple/out/csharp-default/packages/default.pkg"),
				new TypeLoader()
			);

			Outki.Everything everything = Putki.PackageManager.Resolve<Outki.Everything>("everything");
			Console.WriteLine("I loaded built " + everything);
		}
	}
}

