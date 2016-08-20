package putked;

import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;

import putki.Compiler;
import putki.Compiler.ParsedField;
import putki.Compiler.ParsedStruct;

public class DataLoader
{
	public DataLoader(Path root)
	{
		m_root = root;
	}

	class ParseStatus
	{
		byte[] data;
		int pos;
		boolean error;
		String lastValue;
	}

	public interface JsonFn
	{
		void onProperty(ParseStatus status, String name);
	}

	public interface JsonArrayFn
	{
		void onEntry(ParseStatus status);
	}

	public String decodeString(byte[] data, int begin, int end)
	{
		String str = new String(data,  begin,  end-begin,  Charset.forName("UTF-8"));
		int strLen = str.length();
		StringBuilder tmp = new StringBuilder(data.length);
		boolean doEscape = false;
		int u = -1;
		int uval = 0;
		String hex = new String("0123456789abcdef");

		for (int i=0;i<strLen;i++)
		{
			char b = str.charAt(i);
			if (u != -1)
			{
				uval = uval + hex.indexOf(b) << (3-u)*4;
				if (++u == 4)
				{
					tmp.append((char)(uval));
					u = -1;
				}
			}
			else if (b == '\\')
			{
				doEscape = true;
				continue;
			}
			else if (doEscape)
			{
				switch (b)
				{
					case 'n': tmp.append("\n"); break;
					case 't': tmp.append("\t"); break;
					case 'r': tmp.append("\r"); break;
					case '"': tmp.append("\""); break;
					case 'u': u = 0; uval = 0; break;
					default: System.out.println("Unknown escape code [" + b + "]!"); break;
				}
				doEscape = false;
				continue;
			}
			else
			{
				tmp.append((char)b);
			}
		}

		return tmp.toString();
	}

	public void parseArray(ParseStatus status, JsonArrayFn fn)
	{
		int pos = status.pos;
		int length = status.data.length;
		int arrayStart = -1;
		for (;pos<length;pos++)
		{
			char b = (char)status.data[pos];
			if (b == '[')
			{
				arrayStart = pos + 1;
				break;
			}
		}
		if (arrayStart == -1)
		{
			status.error = true;
			status.pos = pos;
			System.out.println("Error parsing array - unstarted");
			return;
		}

		boolean comma = false;
		for (pos=arrayStart;pos<length;pos++)
		{
			char b = (char)status.data[pos];
			if (b == ']')
			{
				status.pos = pos + 1;
				return;
			}
			else if (comma)
			{
				if (b == ',')
				{
					comma = false;
					status.pos = pos + 1;
					continue;
				}
				else if (!Character.isWhitespace(b))
				{
					System.out.println("Unexpected character [" + b + "] in array.");
					status.error = true;
					return;
				}
			}
			else if (!Character.isWhitespace(b))
			{
				status.pos = pos;
				fn.onEntry(status);
				if (status.error)
				{
					return;
				}
				if (status.pos == pos)
					parseValue(status);
				pos = status.pos - 1;
				comma = true;
			}
		}
		System.out.println("Error parsing array - unterminated");
		status.error = true;
	}

	public String parseValue(ParseStatus status)
	{
		int pos = status.pos;
		int length = status.data.length;
		boolean quoted = false;
		int number = -1;
		for (;pos<length;pos++)
		{
			char b = (char)status.data[pos];
			if (b == '\\')
			{
				pos++;
				continue;
			}
			if (b == '\"')
			{
				if (!quoted)
				{
					quoted = true;
					status.pos = pos + 1;
				}
				else
				{
					String value = decodeString(status.data, status.pos, pos);
					status.pos = pos + 1;
					status.lastValue = value;
					return value;
				}
			}
			else if (quoted)
			{
				continue;
			}
			if (b == '[')
			{
				status.pos = pos;
				parseArray(status,  new JsonArrayFn() {
					@Override
					public void onEntry(ParseStatus status) {
						System.out.println("=> Ignoring value array entry");
						parseValue(status);
					}
				});
				return null;
			}
			if (b == '{')
			{
				status.pos = pos;
				parseObject(status, new JsonFn() {
					@Override
					public void onProperty(ParseStatus status, String name) {
						System.out.println("=> Ignoring value object entry [" + name + "]");
						parseValue(status);
					}
				});
				return null;
			}
			if (Character.isDigit(b) || b == '.')
			{
				if (number == -1)
					number = pos;
			}
			else
			{
				if (number != -1)
				{
					String value = new String(status.data, number, pos - number);
					status.pos = pos;
					status.lastValue = value;
					return value;
				}
			}
		}
		status.error = true;
		return null;
	}

	public void parseObject(ParseStatus status, JsonFn fn)
	{
		boolean inside = false;
		int pos = status.pos;
		int length = status.data.length;
		for (;pos<length;pos++)
		{
			byte b = status.data[pos];
			if (b == '{')
			{
				inside = true;
				pos++;
				break;
			}
			else if (!Character.isWhitespace(b))
			{
				System.out.println("Invalid JSON object at pos " + pos);
				status.error = true;
				return;
			}
		}

		if (!inside)
		{
			System.out.println("Missing JSON object at pos " + pos);
			status.error = true;
			return;
		}

		int nameStart = -1;
		String name = null;
		boolean quoted = false;
		boolean comma = false;
		for (;pos<length;pos++)
		{
			char b = (char)status.data[pos];
			if (b == '\\')
			{
				pos++;
				continue;
			}
			if (b == '}')
			{
				status.pos = pos + 1;
				return;
			}
			if (comma)
			{
				if (b == ',')
				{
					comma = false;
					continue;
				}
			}

			if (name == null)
			{
				if (nameStart == -1)
				{
					if (b == '"')
					{
						nameStart = pos + 1;
						quoted = true;
						continue;
					}
					else if (!Character.isWhitespace(b))
					{
						nameStart = pos;
						continue;
					}
				}
				else if (quoted && b == '"')
				{
					name = decodeString(status.data, nameStart, pos);
					continue;
				}
				else if (b == ':' || Character.isWhitespace(b))
				{
					name = decodeString(status.data, nameStart, pos);
					pos--;
				}
			}
			else if (b == ':')
			{
				status.pos = pos + 1;
				fn.onProperty(status,  name);
				if (status.pos == (pos + 1))
				{
					System.out.println("Ignoring field [" + name + "]");
					parseValue(status);
				}
				if (status.error)
				{
					System.out.println("Error parsing property [" + name + "]");
					return;
				}
				pos = status.pos - 1;
				name = null;
				nameStart = -1;
				quoted = false;
				comma = true;
				continue;
			}
		}

		System.out.println("Error parsing json object!");
		status.error = true;
		return;
	}


	class Tmp
	{
		DataObject result;
		String ref;
		int index;
	}

	void parseInto(ParseStatus status, DataObject obj, ParsedField field, int arrayIndex)
	{
		try
		{
			switch (field.type)
			{
				case STRUCT_INSTANCE:
				{
					Tmp t = new Tmp();
					t.result = new DataObject(field.resolvedRefStruct, obj.getRootAsset(), obj.getPath());
					parseData(status, t);
					if (!status.error)
					{
						obj.setField(field.index, arrayIndex, t.result);
					}
					break;
				}
				case POINTER:
				{
					String s = parseValue(status);
					int aux = s.indexOf('#');
					if (aux != -1)
					{
						obj.setField(field.index, arrayIndex, obj.getRootAsset().getPath() + s.substring(aux));
					}
					else
					{
						obj.setField(field.index, arrayIndex, s);
					}
					break;
				}
				case PATH:
				case FILE:
				case STRING:
				case ENUM:
				{
					String s = parseValue(status);
					obj.setField(field.index, arrayIndex, s);
					break;
				}
				case FLOAT:
				{
					Float f = Float.parseFloat(parseValue(status));
					obj.setField(field.index,  arrayIndex, f);
					break;
				}
				case INT32:
				case UINT32:
				case BYTE:
				{
					Long l = Long.parseLong(parseValue(status));
					obj.setField(field.index, arrayIndex, l);
					break;
				}
				case BOOL:
				{
					Boolean b = Long.parseLong(parseValue(status)) != 0;
					obj.setField(field.index, arrayIndex, b);
					break;
				}

			}
		}
		catch (NumberFormatException e)
		{
			System.out.println("Bad number format parsing string " + field.name + "! [lastValue=" + status.lastValue + "]");
			e.printStackTrace();
		}
	}

	void parseData(ParseStatus status, Tmp tmp)
	{
		final Compiler.ParsedStruct struct = tmp.result.getType();
		parseObject(status,  new JsonFn() {
			@Override
			public void onProperty(ParseStatus status, String name) {
				boolean found = false;
				for (Compiler.ParsedField field : struct.fields)
				{
					if (name.equals(field.name))
					{
						if (field.isArray)
						{
							final Tmp itmp = new Tmp();
							itmp.result = tmp.result;
							parseArray(status,  new JsonArrayFn() {
								@Override
								public void onEntry(ParseStatus status) {
									parseInto(status, itmp.result, field, itmp.index);
									itmp.index++;
								}
							});
						}
						else
						{
							parseInto(status, tmp.result, field, -1);
						}
						found = true;
						break;
					}
				}
				if (!found)
				{
					System.out.println("Error; unrecognized field " + name);
				}
			}
		});
	}

	void parseAux(ParseStatus status, Tmp tmp)
	{
		final Tmp tmp2 = new Tmp();
		parseArray(status, new JsonArrayFn() {
			@Override
			public void onEntry(ParseStatus status) {
				parseObject(status, new JsonFn() {
					@Override
					public void onProperty(ParseStatus status, String name) {
						if (name.equals("ref"))
						{
							tmp2.ref = parseValue(status);
						}
						else if (name.equals("type") && tmp2.ref != null)
						{
							ParsedStruct struct = Main.s_compiler.getTypeByName(parseValue(status));
							if (!status.error && struct != null)
								tmp2.result = new DataObject(struct, tmp.result.getRootAsset(), tmp.result.getPath() + tmp2.ref);
						}
						else if (name.equals("data"))
						{
							parseData(status,  tmp2);
							if (!status.error)
							{
								tmp.result.getRootAsset().addAux(tmp2.ref, tmp2.result);
							}
						}
					}
				});
				tmp2.ref = null;
				tmp2.result = null;
			}
		});
	}

	HashMap<String, DataObject> m_loaded = new HashMap<>();

	public DataObject load(String path)
	{
		int aux = path.indexOf('#');
		if (aux != -1)
		{
			DataObject p = load(path.substring(0, aux));
			if (p == null)
			{
				return null;
			}
			return p.getAux(path.substring(aux));
		}

		DataObject d = m_loaded.get(path);
		if (d != null)
		{
			return d;
		}
		d = loadFromDisk(path);
		m_loaded.put(path, d);
		return d;
	}

	public DataObject loadFromDisk(String path)
	{
		try
		{
			Path p = m_root;
			int st = 0;
			for (int k=0;k<path.length();k++)
			{
				if (path.charAt(k) == '/' && k > (st+1))
				{
					p = p.resolve(path.substring(st, k));
					st = k + 1;
				}
			}
			p = p.resolve(path.substring(st) + ".json");
			ParseStatus ps = new ParseStatus();
			ps.data = Files.readAllBytes(p);
			ps.pos = 0;
			final Tmp tmp = new Tmp();
			parseObject(ps,  new JsonFn() {
				public void onProperty(ParseStatus status, String name)
				{
					if (name.equals("type"))
					{
						String val = parseValue(status);
						System.out.println("Parsed [" + name + "] = [" + val + "] error=" + status.error);
						Compiler.ParsedStruct t = Main.s_compiler.getTypeByName(val);
						if (t == null)
						{
							System.out.println("Unknown type [" + val + "] in file [" + path + "]!");
							status.error = true;
						}
						else
						{
							tmp.result = new DataObject(t, path);
						}
					}
					else if (name.equals("data") && tmp.result != null)
					{
						parseData(status, tmp);
					}
					else if (name.equals("aux") && tmp.result != null)
					{
						parseAux(status, tmp);
					}
					else
					{
						System.out.println("Malformed object file!");
					}
				}
			});

			/*
			 * Test writing the object back as .json2 for manual inspection on load.
			 *
			if (tmp.result != null)
			{
				DataWriter dw = new DataWriter(m_root);
				StringBuilder sb = dw.writeAsset(tmp.result);

				java.io.File f = new java.io.File(p.toAbsolutePath() + "2");
				Files.write(f.toPath(), sb.toString().getBytes(Charset.forName("UTF-8")));
			}
			*/

			return tmp.result;
		}
		catch (IOException e)
		{
			return null;
		}
	}

	Path m_root;
}
