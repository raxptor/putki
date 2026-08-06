using System;
using System.Collections.Generic;

namespace Putki
{
	public static class MicroJson
	{
		public struct ParseStatus
		{
			public byte[] data;
			public int pos;
			public bool error;
		}

		public delegate void OnField(string name);
		public delegate void OnArrayEntry(string entry);

		enum Parsing
		{
			NOTHING,
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

		// Hex digit value, or -1 if it is not a hex digit. Unlike unhex above,
		// which cannot distinguish a bad digit from a zero.
		static int HexVal(byte x)
		{
			if (x >= '0' && x <= '9')
				return x - '0';
			byte lower = (byte)(x | 0x20);
			if (lower >= 'a' && lower <= 'f')
				return 10 + lower - 'a';
			return -1;
		}

		// Rejects malformed UTF-8 instead of substituting replacement
		// characters, so bad data fails loudly.
		static readonly System.Text.UTF8Encoding StrictUtf8 = new System.Text.UTF8Encoding(false, true);

		// Decode the body of a quoted literal, i.e. what sits between the
		// quotes. See doc/text-format.md, which is the normative definition.
		//
		// Returns null on an invalid escape rather than guessing; callers must
		// treat that as a parse error. Decoding is done in bytes because legacy
		// \uXXXX encoded the individual bytes of the original UTF-8, not a code
		// point, so a run of them has to be reassembled before it is valid text.
		public static String DecodeString(byte[] buf, int begin, int end)
		{
			// No escape expands, so the input length is always enough.
			byte[] tmp = new byte[end-begin];
			int len = 0;
			for (int i=begin;i<end;i++)
			{
				byte b = buf[i];
				if (b == '\r')
				{
					// Newline is newline: CRLF and a lone CR both mean one.
					if ((i+1) < end && buf[i+1] == '\n')
						i++;
					tmp[len++] = (byte)'\n';
					continue;
				}
				if (b != '\\')
				{
					tmp[len++] = b;
					continue;
				}
				if ((i+1) >= end)
					return null;
				byte n = buf[++i];
				if (n == 'n')
					tmp[len++] = (byte)'\n';
				else if (n == 't')
					tmp[len++] = (byte)'\t';
				else if (n == 'r')
					tmp[len++] = (byte)'\n';
				else if (n == '\\')
					tmp[len++] = (byte)'\\';
				else if (n == '"')
					tmp[len++] = (byte)'"';
				else if (n == 'u')
				{
					if ((i+4) >= end)
						return null;
					int code = 0;
					for (int k=1;k<=4;k++)
					{
						int h = HexVal(buf[i+k]);
						if (h < 0)
							return null;
						code = (code << 4) | h;
					}
					if (code > 0xff)
						return null;
					tmp[len++] = (byte)code;
					i += 4;
				}
				else
					return null;
			}
			try
			{
				return StrictUtf8.GetString(tmp, 0, len);
			}
			catch (ArgumentException)
			{
				return null;
			}
		}

		public static bool IsWhitespace(char c)
		{
			return c == ' ' || c == '\t' || c == 0xD || c == 0xA;
		}

		static string Normalize(string s)
		{
			return s.ToLowerInvariant().Replace("-", "").Replace("_", "");
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
							if (v == null)
							{
								status.error = true;
								return null;
							}
							status.pos = i + 1;
							return v;
						}
						break;
					}
					case Parsing.VALUE:
						{
							if (IsWhitespace(c) || c == ',' || c == ']' || c == '}' || c == ':')
							{
								String v = DecodeString(status.data, status.pos, i);
								if (v == null)
								{
									status.error = true;
									return null;
								}
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
								if (c == ':')
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
			status.error = true;
			return null;
		}

		public static Dictionary<string, object> Parse(byte[] buffer)
		{
			ParseStatus status = new ParseStatus();
			status.data = buffer;
			status.pos = 0;
			object root = Parse(ref status);
			if (status.error)
			{
				return null;
			}
			else
			{
				return root as Dictionary<string, object>;
			}
		}
	}
}
