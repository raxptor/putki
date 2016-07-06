using System.Text; using System.Collections.Generic;  namespace Putki { 	public class PackageManager 	{ 		private static List<Package> s_loaded = new List<Package>(); 		private static Dictionary<string, object> s_pathToObject = new Dictionary<string, object>(); 		private static Dictionary<object, string> s_objectToPath = new Dictionary<object, string>();  		static public Package LoadFromBytes(byte[] bytes, TypeLoader loader) 		{ 			Package p = new Package(); 			p.LoadFromBytes(bytes, loader); 			s_loaded.Insert(0, p); 			return p; 		}  		static public void RegisterLoaded(string path, object o) 		{ 			s_pathToObject.Add(path, o); 			s_objectToPath.Add(o, path); 		}  		public static string PathOf(object o) 		{ 			if (o == null) 			{ 				return null; 			} 			foreach (Package p in s_loaded) 			{ 				string tmp = p.PathOf(o); 				if (tmp != null) 					return tmp; 			} 			string path; 			if (s_objectToPath.TryGetValue(o, out path)) 			{ 				return path; 			} 			return null; 		}  		public static Type Resolve<Type>(string path) 		{ 			if (path == null) 			{ 				return default(Type); 			} 			foreach (Package p in s_loaded) 			{ 				object o = p.Resolve(path); 				if (o != null) 					return (Type)o; 			} 			object tmp; 			if (s_pathToObject.TryGetValue(path, out tmp)) 			{ 				return (Type)tmp; 			}  			return default(Type); 		}  	} }    