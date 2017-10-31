using System;
using System.Collections.Generic;

namespace Putki
{
	// Easy edit, easy parse file format.
	public static class Eeep
	{
		public struct ParseStatus
		{
			public byte[] data;
			public int pos;
			public bool error;
			public Dictionary<string, Object> result;
		}

		public struct Object
		{
			public string Path;
			public string Type;
			public object Data;
		}

		public delegate void OnField(string name);
		public delegate void OnArrayEntry(string entry);

		enum Parsing
		{
			NOTHING,
			HEADER,
			VALUE,
			QUOTED_VALUE,
			OBJECT,
			ARRAY
		};

		public static int unhex(byte x)
		{
			if (x >= '0' && x <= '9')
				return x - '0';
			if (x >= 'a' && x <= 'f')
				return 10 + x - 'a';
			return 0;
		}

		public static String DecodeString(byte[] buf, int begin, int end)
		{
			byte[] tmp = new byte[end-begin];
			int len = 0;
			for (int i=begin;i<end;i++)
			{
				if (buf[i] != '\\')
				{
					tmp[len++] = buf[i];
				}
				else if ((i+1) < end)
				{
					if (buf[i+1] == 'u')
					{
						if ((i+5) < end)
						{
							int code = 
								16*16*16*unhex(buf[i+2]) + 
								16*16*unhex(buf[i+3]) +
								16*unhex(buf[i+4]) +
								unhex(buf[i+5]);
							tmp[len++] = (byte) code;
							i += 5;
						}
					}
				}
			}
			return System.Text.Encoding.UTF8.GetString(tmp, 0, len);
		}

		public static bool IsWhitespace(char c)
		{
			return c == ' ' || c == '\t' || c == 0xD || c == 0xA;
		}

		static string MakeAnonymousPath()
		{
			return "anonymous";
		}

		public static object Parse(ref ParseStatus status)
		{
			Parsing state = Parsing.NOTHING;
			Dictionary<string, object> o = null;
			List<object> a = null;
			String name = null;
			for (int i=status.pos;i<status.data.Length;i++)
			{
				byte b = status.data[i];
				char c = (char)b;
				switch (state)
				{
					case Parsing.NOTHING:
					{
						switch (c)
						{
							case '@': state = Parsing.HEADER; break;
							case '{': state = Parsing.OBJECT; o = new Dictionary<string, object>(); break;
							case '[': state = Parsing.ARRAY; a = new List<object>(); break;
							case ' ': case '\n': case '\t': break;
							case '"': state = Parsing.QUOTED_VALUE; status.pos = i+1; break;
							default: state = Parsing.VALUE; status.pos = i; break;
						}
						break;
					}
					case Parsing.QUOTED_VALUE:
					{
						if (c == '\\')
						{
							i++;
							break;
						}
						if (c == '"')
						{
							String v = DecodeString(status.data, status.pos, i);
							status.pos = i + 1;
							return v;
						}
						break;
					}
					case Parsing.HEADER:
						{
							if (c == '{' || c== '[')
							{
								string header = DecodeString(status.data, status.pos, i);
								string[] pcs = header.Trim().Split(new char[] {' ', (char)0xd, (char)0xA, '\t' });
								if (pcs.Length < 1)
								{
									status.error = true;
									return null;
								}

								Object no = new Object();
								no.Type = pcs[0].Replace("@", "");
								if (pcs.Length > 1)
								{
									no.Path = pcs[1].Trim();
								}
								else
								{
									no.Path = MakeAnonymousPath();
								}
								status.pos = i;
								no.Data = Parse(ref status);
								i = status.pos - 1;
								status.result.Add(no.Path, no);
								state = Parsing.NOTHING;
							}
							break;
						}
					case Parsing.VALUE:
						{
							if (IsWhitespace(c) || c == ',' || c == ']' || c == '}' || c == ':' || c == '=')
							{
								String v = DecodeString(status.data, status.pos, i);
								status.pos = i;
								return v;
							}
							break;
						}
					case Parsing.OBJECT:
						{
							if (c == '}')
							{
								status.pos = i + 1;
								return o;
							}
							if (IsWhitespace(c) || c == ',')
							{
								continue;
							}
							if (name == null)
							{
								status.pos = i;
								name = Parse(ref status) as String;
								if (name == null)
								{
									status.error = true;
									return null;
								}
								i = status.pos - 1;
							}
							else 
							{
								if (c == ':' || c == '=')
								{
									continue;
								}
								status.pos = i;
								object val = Parse(ref status);
								if (val == null)
								{
									status.error = true;
									return null;
								}
								o.Add(name, val);
								i = status.pos - 1;
								name = null;
							}
							break;
						}
					case Parsing.ARRAY:
						{
							if (c == ']')
							{
								status.pos = i + 1;
								return a;
							}
							if (IsWhitespace(c) || c == ',')
							{
								continue;
							}
							status.pos = i;
							object val = Parse(ref status);
							if (val == null)
							{
								status.error = true;
								return null;
							}
							a.Add(val);
							i = status.pos - 1;
							break;
						}
					default:
						break;
				}
			}
			return null;
		}

		public static Dictionary<string, Object> Parse(byte[] buffer)
		{			
			ParseStatus status = new ParseStatus();
			status.data = buffer;
			status.pos = 0;
			status.result = new Dictionary<string, Object>();
			Parse(ref status);
			if (status.error)
			{
				return null;
			}
			else
			{
				return status.result;
			}
		}
	}
}
