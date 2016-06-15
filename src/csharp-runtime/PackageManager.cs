using System.Text;
using System.Collections.Generic;

namespace Putki
{
	public class PackageManager
	{
		private static List<Package> s_loaded = new List<Package>();

		static public Package LoadFromBytes(byte[] bytes, TypeLoader loader)
		{
			Package p = new Package();
			p.LoadFromBytes(bytes, loader);
			s_loaded.Insert(0, p);
			return p;
		}

		public static string PathOf(object o)
		{
			foreach (Package p in s_loaded)
			{
				string tmp = p.PathOf(o);
				if (tmp != null)
					return tmp;
			}
			return null;
		}

		public static Type Resolve<Type>(string path)
		{
			foreach (Package p in s_loaded)
			{
				object o = p.Resolve(path);
				if (o != null)
					return (Type)o;
			}
			return default(Type);
		}

	}
}
