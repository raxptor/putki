using System;
using System.Text;
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
					else if (buf[i+1] =='n')
					{
						tmp[len++] = (byte)'\n';
						i++;
					}
					else if (buf[i + 1] == '\\')
					{
						tmp[len++] = (byte)'\\';
						i++;
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

		static string Normalize(string s)
		{
			return s.ToLowerInvariant().Replace("-", "").Replace("_", "");
		}

		static int m_anonCount;

		public static object Parse(ref ParseStatus status, bool rootlevel = false)
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
							if (!IsWhitespace(c))
							{
								switch (c)
								{
									case '@': state = Parsing.HEADER; break;
									case '{': state = Parsing.OBJECT; o = new Dictionary<string, object>(); break;
									case '[': state = Parsing.ARRAY; a = new List<object>(); break;
									case ' ': case '\n': case '\t': break;
									case '"': state = Parsing.QUOTED_VALUE; status.pos = i + 1; break;
									default: state = Parsing.VALUE; status.pos = i; break;
								}
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
							if (v.StartsWith("$FIX-WS:"))
							{
								StringBuilder sb = new StringBuilder();
								bool ws = true;
								for (int j=8;j<v.Length;j++)
								{
									char vc = v[j];
									if (vc == ' ' || vc == '\t' || vc == '\r' || vc == '\n')
									{
										if (!ws)
										{
											ws = true;
											sb.Append(' ');
										}
									}
									else
									{
										ws = false;
										sb.Append(vc);
									}
								}
								v = sb.ToString().Replace("\\n", "\n");
							}

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

								status.pos = i;
								Dictionary<string, object> data = Parse(ref status) as Dictionary<string, object>;
								if (status.error || data == null)
									return null;
								i = status.pos - 1;

								Dictionary<string, object> no = new Dictionary<string, object>();
								no.Add("type", Normalize(pcs[0]).Replace("@", ""));
								no.Add("data", data);

								if (pcs.Length > 1)
								{
									// it has path
									status.result.Add(pcs[1].Trim(), no);
                                    if (!rootlevel)
                                    {
                                        return pcs[1].Trim();
                                    }
                                }
								else
								{
									// anonymous object, add with path and return.
									string path = "%" + (m_anonCount++);
									status.result.Add(path, no);
									status.pos = i + 1;
									if (!rootlevel)
									{
										return path; 
									}
								}
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
								o.Add(Normalize(name), val);
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
			// Strip comments
			byte[] stripped = new byte[buffer.Length];
			int outp = 0;
			bool comment = false;
			bool quote = false;
			bool escape = false;
			int cstart = 0;
			for (int i=0;i<buffer.Length;i++)
			{
				char c = (char)buffer[i];
				char n = '\x0';
				if (i < (buffer.Length-1))
					n = (char)buffer[i+1];
				
				if (!quote && c == '\"')
				{
					quote = true;
				}
				else if (quote && !escape && c == '\"')
				{
					quote = false;
				}
				if (quote && !escape && c == '\\')
				{
					escape = true;
				}
				if (c != '\\')
				{
					escape = false;
				}
				if (!quote && !comment && (c == '#' || (c == '/' && n == '/')))
				{
					comment = true;
					cstart = i;
				}
				if (comment && (c == 0xd || c == 0xa))
				{
					comment = false;
				}	
				if (!comment)
				{
					stripped[outp++] = buffer[i];
				}
			}

			Array.Resize(ref stripped, outp);

			ParseStatus status = new ParseStatus();
			status.data = stripped;
			status.pos = 0;
			status.result = new Dictionary<string, Object>();
			Parse(ref status, true);
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
