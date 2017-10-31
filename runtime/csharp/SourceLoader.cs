using System;
using System.IO;
using System.Collections.Generic;
using Putki;

namespace Mixki
{
	public class SourceLoader
	{
		public delegate object ParseFn(SourceLoader loader, string path, Dictionary<string, object> obj);
		public delegate void LogFn(string txt);

		public struct Parser
		{
			public Parser(string type, ParseFn fn)
			{
				Type = type;
				Fn = fn;
			}
			public string Type;
			public ParseFn Fn;
		}

		string m_root;
		Dictionary<String, object> m_raw;
		Dictionary<String, object> m_parsed;
		Dictionary<String, ParseFn> m_parsers;

		public LogFn Logger;

		public SourceLoader(string root, Parser[] parsers)
		{
			m_root = root;
			m_raw = new Dictionary<string, object>();
			m_parsed = new Dictionary<string, object>();
			m_parsers = new Dictionary<string, ParseFn>();
			foreach (Parser p in parsers)
			{
				m_parsers.Add(p.Type, p.Fn);
			}
			Logger = delegate {				
			};
		}

		public Type Resolve<Type>(string assetPath, string path)
		{
			if (path == null || path == "")
				return default(Type);
			
			if (path.StartsWith("#"))
				return Resolve<Type>(assetPath + path);
			else
				return Resolve<Type>(path);
		}

		public Type Resolve<Type>(string path)
		{
			if (path == null || path == "")
				return default(Type);
			
			string assetPath = path;
			int auxref = assetPath.IndexOf('#');
			if (auxref != -1)
			{
				assetPath = path.Substring(0, auxref);
			}

			object val;
			if (m_parsed.TryGetValue(path, out val))
			{
				return (Type) val;
			}
			else
			{
				object raw;
				if (!m_raw.TryGetValue(path, out raw))
				{
					Load(assetPath);
					if (!m_raw.TryGetValue(path, out raw))
					{
						return default(Type);
					}
				}

				Dictionary<string, object> ro = raw as Dictionary<string, object>;
				object typeObj;
				if (!ro.TryGetValue("type", out typeObj))
				{
					Logger("Failed to read type field of [" + path + "]");
					return default(Type);
				}

				object dataObj;
				if (!ro.TryGetValue("data", out dataObj))
				{
					Logger("Failed to read data field of [" + path + "]");
					return default(Type);
				}

				Dictionary<string, object> datas = dataObj as Dictionary<string, object>;
				if (datas == null)
				{
					Logger("Not a dictionary for object at [" + path + "]");
					return default(Type);
				}

				string type = typeObj.ToString();
				ParseFn p;
				if (m_parsers.TryGetValue(type, out p))
				{
					object parsed = p(this, assetPath, datas);
					Logger("Parsed [" + path + "] as [" + type + "]");
					m_parsed.Add(path, parsed);
					Putki.PackageManager.RegisterLoaded(path, parsed);
					return (Type) parsed;
				}
				else
				{
					Logger("No parser for type [" + type + "] for path [" + path + "]");
					return default(Type);
				}
			}
		}

		public object GetRaw(string path)
		{
			object val;
			if (m_raw.TryGetValue(path, out val))
			{
				return val;
			}
			else
			{
				Load(path);
				m_raw.TryGetValue(path, out val);
				return val;
			}
		}

		public void InsertRawData(string path, byte[] bytes)
		{
			Dictionary<string, object> file = MicroJson.Parse(bytes);
			if (file == null)
			{
				Logger("Failed to load [" + path + "]");
				return;
			}

			m_raw.Add(path, file);
			Logger("Raw: adding main " + path);

			object auxesObj;
			file.TryGetValue("aux", out auxesObj);
			var auxesArr = auxesObj as List<object>;
			if (auxesArr != null)
			{
				for (int i=0;i<auxesArr.Count;i++)
				{
					var ao = auxesArr[i] as Dictionary<string, object>;
					if (ao == null)
					{
						continue;
					}
					object refObj;
					if (!ao.TryGetValue("ref", out refObj))
					{
						continue;
					}
					string refName = refObj.ToString();
					string auxPath = refName.StartsWith("#") ? (path + refName) : refName;
					Logger("Raw: adding aux " + auxPath);
					m_raw.Add(auxPath, ao);
				}
			}
		}

		void Load(string path)
		{
			string fn = m_root;
			string tmp = path;
			while (tmp.Length > 0)
			{
				int slash = tmp.IndexOf('/');
				if (slash != -1)
				{
					fn = Path.Combine(fn, tmp.Substring(0, slash));
					tmp = tmp.Substring(slash + 1);
					continue;
				}
				fn = Path.Combine(fn, tmp);
				break;
			}

			fn = fn + ".json";

			Logger("Opening file [" + fn + "]");
			byte[] bytes = System.IO.File.ReadAllBytes(fn);
			if (bytes != null)
			{
				InsertRawData(path, bytes);
			}
		}
	}
}

