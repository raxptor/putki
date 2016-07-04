package putki;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.nio.file.Path;
import java.util.List;
import java.util.ArrayList;

public class Compiler
{
	public static int DOMAIN_INPUT  = 1;
	public static int DOMAIN_OUTPUT = 2;

	public enum FieldType
	{
        INT32,
        UINT32,
        FLOAT,
        BYTE,
        BOOL,
        STRING,
        POINTER,
        PATH,
        STRUCT_INSTANCE,
        ENUM,
        FILE
	}

	public class ParsedField
	{
		int domains;
		boolean isArray;
		boolean isAuxPtr;
		boolean isBuildConfig;
		boolean showInEditor;
		FieldType type;
		String name;
		String refType;
		String defValue;
		ParsedStruct resolvedRefStruct;
		ParsedEnum resolvedEnum;
	}

	public class ParsedStruct
	{
		int domains;
		int uniqueId;
		String name;
		String parent;
		String loaderName;
		List<ParsedField> fields;
		String inlineEditor;
		List<String> targets;
		boolean isTypeRoot;
		boolean permitAsAux;
		boolean permitAsAsset;
	}

	public class EnumValue
	{
		String name;
		int value;
	}

	public class ParsedEnum
	{
		String loaderName;
		String name;
		List<EnumValue> values;
	}

	public class ParsedFile
	{
		String sourcePath;
		String fileName;
		String moduleName;
		String signature;
		List<ParsedStruct> structs;
		List<ParsedEnum> enums;
		List<String> includes;
	};

	public class ParsedTree
	{
		public String moduleName;
		public String loaderName;
		public String typeFileEnding;
		public List<ParsedFile> parsedFiles;
	}

	public void error(String path, int line, String err)
	{
		System.out.println(path + ":" + line + " Error! " + err);
	}

	public boolean parseEnumLine(ParsedEnum cur, String line)
	{
		EnumValue val = new EnumValue();
		String[] pieces = line.split("\\s+");
		boolean readValue = false;
		for (int k=0;k<pieces.length;k++)
		{
			if (readValue)
			{
				Integer tmp = Integer.parseInt(pieces[k]);
				if (tmp != null)
					val.value = (int)tmp;
				else
					return false;
			}
			else if (pieces[k].equals("="))
			{
				if (val.name != null)
					readValue = true;
				else
					return false;
			}
			else if (val.name == null)
			{
				val.name = pieces[k];
			}
			else
			{
				return false;
			}
		}
		return true;
	}

	public class TypeMap
	{
		public String name;
		public FieldType type;

		public TypeMap(String _s, FieldType _f)
		{
			name = _s;
			type = _f;
		}
	}

	TypeMap[] stdTypes = new TypeMap[] {
			new TypeMap("u32", FieldType.UINT32),
			new TypeMap("uint", FieldType.UINT32),
			new TypeMap("s32", FieldType.INT32),
			new TypeMap("int", FieldType.INT32),
			new TypeMap("bool", FieldType.BOOL),
			new TypeMap("byte", FieldType.BYTE),
			new TypeMap("string", FieldType.STRING),
			new TypeMap("float", FieldType.FLOAT),
			new TypeMap("file", FieldType.FILE),
			new TypeMap("path", FieldType.PATH)
	};

	public boolean parseStructLine(ParsedStruct cur, String line)
	{
		boolean gotType = false;
		boolean readRefType = false;
		boolean readDefValue = false;
		ParsedField next = new ParsedField();
		next.domains = DOMAIN_INPUT | DOMAIN_OUTPUT;

		String[] pieces = line.split("\\s+");
		for (int k=0;k<pieces.length;k++)
		{
			if (pieces[k].equals("="))
			{
				if (next.name != null)
				{
					readDefValue = true;
				}
				else
				{
					return false;
				}
			}
			else if (pieces[k].equals("no-out"))
			{
				next.domains = next.domains & ~DOMAIN_OUTPUT;
			}
			else if (!gotType)
			{
				String typeName = pieces[k];
				if (pieces[k].endsWith("[]"))
				{
					next.isArray = true;
					typeName = pieces[k].substring(0,  pieces[k].length() - 2);
				}

				for (int i=0;i<stdTypes.length;i++)
				{
					if (typeName.equals(stdTypes[i].name))
					{
						next.type = stdTypes[i].type;
						gotType = true;
						break;
					}
				}

				if (typeName.equals("ptr"))
				{
					next.type = FieldType.POINTER;
					readRefType = true;
					gotType = true;
				}
				else if (typeName.equals("auxptr"))
				{
					next.type = FieldType.POINTER;
					next.isAuxPtr = true;
					readRefType = true;
					gotType = true;
				}
				else if (typeName.equals("enum"))
				{
					next.type = FieldType.ENUM;
					readRefType = true;
					gotType = true;
				}
				else if (!gotType)
				{
					next.type = FieldType.STRUCT_INSTANCE;
					readRefType = true;
					gotType = true;
				}
			}
			else if (readRefType)
			{
				next.refType = pieces[k];
				readRefType = false;
			}
			else if (readDefValue)
			{
				next.defValue = pieces[k];
				readDefValue = false;
			}
			else if (next.name == null)
			{
				next.name = pieces[k];
			}
			else
			{
				System.out.println("Unexpected token when parsing field. Line [" + line + "]");
				return false;
			}
		}

		if (gotType && !readRefType && !readDefValue)
		{
			cur.fields.add(next);
			return true;
		}
		else
		{
			System.out.println("line [" + line + "] parse status gotType:" + gotType + " type=" + next.type + "  name:" + next.name + " readRef:" + readRefType + " readDef:" + readDefValue);
		}
		return false;
	}

	public ParsedFile parseFile(File f, String path) throws IOException
	{
		System.out.println("Parsing [" + path + "]");
		List<String> lines = Files.readAllLines(f.toPath());

		ParsedFile tmp = new ParsedFile();
		tmp.structs = new ArrayList<ParsedStruct>();
		tmp.enums = new ArrayList<ParsedEnum>();
		tmp.includes = new ArrayList<String>();

		ParsedEnum curEnum = null;
		ParsedStruct curStruct = null;
		boolean entered = false;
		for (int i=0;i<lines.size();i++)
		{
			String line = lines.get(i).trim();

			int comment = line.indexOf("//");
			if (comment != -1)
			{
				line = line.substring(0,  comment);
			}

			if (line.length() == 0)
			{
				continue;
			}

			if (line.startsWith("%include"))
			{
				tmp.includes.add(line.substring(8));
				continue;
			}

			if (curEnum != null || curStruct != null)
			{
				if (line.equals("{"))
				{
					if (entered)
					{
						error(path, i, "Double opening {?!");
					}
					else
					{
						entered = true;
					}
					continue;
				}
				else if (line.equals("}"))
				{
					if (entered)
					{
						if (curStruct != null)
						{
							tmp.structs.add(curStruct);
							curStruct = null;
						}
						else if (curEnum != null)
						{
							tmp.enums.add(curEnum);
							curEnum = null;
						}
						else
						{
							error(path, i, "Internal compiler error!");
						}
						entered = false;
					}
					else
					{
						error(path, i, "Mismatched }");
					}
					continue;
				}
				else
				{
					// do field or values
					if (curEnum != null)
					{
						if (!parseEnumLine(curEnum, line))
						{
							error(path, i, "Parse error in enum.");
							System.out.println("Line is [" + line + "]");
						}
					}
					else if (curStruct != null)
					{
						if (!parseStructLine(curStruct, line))
						{
							error(path, i, "Parse error in struct.");
							System.out.println("Line is [" + line + "]");
						}
					}
				}
			}

			if (!entered && curEnum == null && curStruct == null)
			{
				String[] pieces = line.split("\\s+");
				int readModifiers = -1;
				for (int j=0;j<pieces.length && readModifiers == -1;j++)
				{
					if (pieces[j].equals("enum"))
					{
						if ((j+1) < pieces[j].length())
						{
							curEnum = new ParsedEnum();
							curEnum.name = pieces[j+1];
							curEnum.values = new ArrayList<EnumValue>();
							readModifiers = j + 2;
						}
						else
						{
							error(path, i, "Invalid enum definition! Needs a name");
							break;
						}
					}
					else
					{
						curStruct = new ParsedStruct();
						curStruct.name = pieces[j];
						curStruct.domains = 0xff;
						curStruct.fields = new ArrayList<ParsedField>();
						readModifiers = j + 1;
						break;
					}
				}
			}
		}
		return null;
	}

	public void scanTree(ParsedTree tree, Path root, Path where) throws IOException
	{
		File[] files = where.toFile().listFiles();
		for (int i=0;i!=files.length;i++)
		{
			if (files[i].isDirectory())
			{
				scanTree(tree, root, files[i].toPath());
			}
			else
			{
				String name = files[i].getName();
				int dot = name.lastIndexOf('.');
				if (dot == -1)
					continue;

				if (name.substring(dot+1).equals(tree.typeFileEnding))
				{
					Path th = root.relativize(files[i].toPath());
					ParsedFile pf = parseFile(files[i], th.toString());
					if (pf != null)
					{
						tree.parsedFiles.add(pf);
					}
				}
			}
		}
	}

	public boolean scanPath(Path start)
	{
		try
		{
			System.out.println("Scanning [" + start.toString() + "]");
			Path configPath = start.resolve("putki-compiler.config");
			List<String> lines = Files.readAllLines(configPath);

			ParsedTree pt = new ParsedTree();
			pt.moduleName = "module";
			pt.loaderName = "loader";
			pt.typeFileEnding = "typedef";
			pt.parsedFiles = new ArrayList<ParsedFile>();
			if (lines.size() > 1)
			{
				pt.moduleName = lines.get(0);
				pt.loaderName = lines.get(1);
			}

			for (int i=0;i<lines.size();i++)
			{
				String line = lines.get(i);
				if (line.length() > 4)
				{
					if (line.startsWith("dep:"))
					{
						scanPath(start.resolve(line.substring(4)));
					}
				}
			}

			scanTree(pt, start.resolve("src"), start.resolve("src"));

		}
		catch (java.io.IOException e)
		{

		}
		return true;
	}


	public static void main(String [] args)
	{
		Compiler c = new Compiler();
		if (!c.scanPath(Paths.get("/Users/dannilsson/git/neocrawler/")))
		{
			return;
		}
	}
}

