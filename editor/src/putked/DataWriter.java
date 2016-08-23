package putked;

import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashSet;
import java.util.Iterator;
import java.util.Locale;

import putki.Compiler;
import putki.Compiler.FieldType;

public class DataWriter
{
	public DataWriter(Path root)
	{
		m_root = root;
	}

	public static void indent(StringBuilder bld, int tabs, int spaces)
	{
		for (int i=0;i<tabs;i++)
			bld.append('\t');
		for (int i=0;i<spaces;i++)
			bld.append(' ');
	}

	public static void encString(StringBuilder prop, String name)
	{
		String allow = new String("# _-'!?/(){}%&+*");
		prop.append("\"");
		for (int i=0;i<name.length();i++)
		{
			char a = name.charAt(i);
			if (a == '\\')
			{
				prop.append("\\\\");
			}
			else if (a == '\"')
			{
				prop.append("\\\"");
			}
			else if (Character.isLetterOrDigit(a) || allow.indexOf(a) != -1)
			{
				prop.append(a);
			}
			else
			{
				String hex = "0123456789abcdef";
				prop.append("\\u");
				for (int j=0;j<4;j++)
				{
					prop.append(hex.charAt((a >> 4*(3-j)) & 0xf));
				}
			}
		}
		prop.append("\"");
	}

	HashSet<String> auxRefs = new HashSet<String>();

	public void writeValue(StringBuilder bld, int tabs, Compiler.ParsedField field, Object value)
	{
		if (field.type == FieldType.POINTER)
		{
			String path = value.toString();
			int aux = path.indexOf('#');
			if (aux != -1)
			{
				path = path.substring(aux);
				auxRefs.add(path);
			}
			encString(bld, path);
		}
		else if (value.getClass() == String.class)
		{
			encString(bld, (String) value);
		}
		else if (value.getClass() == Boolean.class)
		{
			if ((Boolean)value)
			{
				bld.append("1");
			}
			else
			{
				bld.append("0");
			}
		}
		else if (value.getClass() == Float.class)
		{
			java.text.NumberFormat nf = java.text.NumberFormat.getInstance(Locale.ENGLISH);
			bld.append(nf.format(value));
		}
		else if (value.getClass() == DataObject.class)
		{
			bld.append("{\n");
			writeData(bld, tabs + 1, (DataObject)value);
			indent(bld, tabs, 0);
			bld.append("}");
		}
		else
		{
			bld.append(value);
		}
	}

	public void writeData(StringBuilder bld, int tabs, DataObject obj)
	{
		Compiler.ParsedStruct type = obj.getType();
		boolean first = true;
		for (Compiler.ParsedField field : type.fields)
		{
			if (!field.isArray)
			{
				Object value = obj.getField(field.index, -1);
				if (value == null)
				{
					continue;
				}
				if (!first)
				{
					bld.append(",\n");
				}
				first = false;
				indent(bld,  tabs, 0);
				encString(bld, field.name);
				bld.append(": ");
				writeValue(bld, tabs, field, value);
			}
			else
			{
				if (!first)
				{
					bld.append(",\n");
				}
				first = false;
				indent(bld,  tabs, 0);
				encString(bld, field.name);
				bld.append(": [");
				int count = obj.getArraySize(field.index);
				for (int i=0;i<count;i++)
				{
					Object val = obj.getField(field.index,  i);
					if (i != 0)
					{
						bld.append(",\n");
						indent(bld, tabs, 0);
					}
					writeValue(bld, tabs + 1, field, val);
				}
				if (count > 0 && field.type == FieldType.STRUCT_INSTANCE)
				{
					bld.append("\n");
					indent(bld, tabs, 0);
				}
				bld.append("]");
			}
		}

		if (!first)
		{
			bld.append("\n");
		}
	}

	public StringBuilder writeAsset(DataObject obj, boolean includeAuxes)
	{
		synchronized (obj)
		{
			auxRefs.clear();
			StringBuilder bld = new StringBuilder();
			bld.append("{\n\t");
			encString(bld, "type");
			bld.append(": ");
			encString(bld, obj.getType().name);
			bld.append(",\n\t");
			encString(bld, "data");
			bld.append(": {\n");
			writeData(bld, 2, obj);

			if (!includeAuxes)
			{
				bld.append("}\n}\n");
				return bld;
			}

			bld.append("\t},\n\t");
			encString(bld, "aux");
			bld.append(": [");

			boolean first = true;
			while (auxRefs.size() > 0)
			{
				// These will get added to as references are written.
				Iterator<String> i = auxRefs.iterator();
				String s = i.next();
				auxRefs.remove(s);

				System.out.println("writing aux [" + s + "]");

				if (first)
				{
					bld.append("\n\t\t{\n\t\t\t");
					first = false;
				}
				else
				{
					bld.append(", {\n\t\t\t");
				}

				encString(bld, "ref");
				bld.append(": ");
				encString(bld, s);

				DataObject aux = obj.getAux(s);

				if (aux == null)
				{
					System.out.println("Missing aux ref! [" + s + "]");
				}
				else
				{
					bld.append(",\n\t\t\t");
					encString(bld, "type");
					bld.append(": ");
					encString(bld, aux.getType().name);
					bld.append(",\n\t\t\t");
					encString(bld, "data");
					bld.append(": {\n");
					writeData(bld, 4, aux);
					bld.append("\t\t\t}");
				}
				bld.append("\n\t\t}");
			}
			bld.append("\n\t]\n");
			bld.append("}\n");
			return bld;
		}
	}

	public void WriteObject(DataObject obj)
	{
		try
		{
			StringBuilder sb;
			Path p = m_root;
			synchronized (obj)
			{
				String path = obj.getPath();
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
				sb = writeAsset(obj, true);
			}
			Files.createDirectories(p.getParent());
			Files.write(p, sb.toString().getBytes(Charset.forName("UTF-8")));
		}
		catch (IOException e)
		{

		}
	}

	Path m_root;
}
