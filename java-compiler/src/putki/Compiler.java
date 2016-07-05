package putki;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.nio.file.Path;
import java.util.List;

import java.util.ArrayList;
import java.util.HashMap;

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
		public int index;
		public int domains;
		public boolean isArray;
		public boolean isAuxPtr;
		public boolean isBuildConfig;
		public boolean showInEditor;
		public FieldType type;
		public String name;
		public String refType;
		public String defValue;
		public ParsedStruct resolvedRefStruct;
		public ParsedEnum resolvedEnum;
	}

	public class ParsedStruct
	{
		public int domains;
		public int uniqueId;
		public String name;
		public String parent;
		public String loaderName;
		public String moduleName;
		public List<ParsedField> fields;
		public String inlineEditor;
		public List<String> targets;
		public boolean isTypeRoot;
		public boolean permitAsAux;
		public boolean permitAsAsset;

		public ParsedStruct resolvedParent;

		public boolean hasParent(ParsedStruct p)
		{
			ParsedStruct c = this;
			while (c != null)
			{
				if (c == p)
				{
					return true;
				}
				c = c.resolvedParent;
			}
			return false;
		}
	}

	public class EnumValue
	{
		public String name;
		public int value;
	}

	public class ParsedEnum
	{
		public String loaderName;
		public String name;
		public List<EnumValue> values;
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

	List<ParsedStruct> allTypes = new ArrayList<Compiler.ParsedStruct>();
	List<ParsedEnum> allEnums = new ArrayList<Compiler.ParsedEnum>();
	HashMap<String, ParsedStruct> typesByName = new HashMap<String, ParsedStruct>();
	HashMap<String, ParsedEnum> enumsByName = new HashMap<String, ParsedEnum>();
	List<ParsedTree> allTrees = new ArrayList<Compiler.ParsedTree>();
	List<String> allTargets = new ArrayList<String>();

	public void error(String path, int line, String err)
	{
		System.out.println(path + ":" + line + " Error! " + err);
	}

	public List<ParsedStruct> getAllTypes()
	{
		return allTypes;
	}

	public ParsedStruct getTypeByName(String s)
	{
		return typesByName.get(s);
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
				val.value = (int)tmp;
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
		next.showInEditor = true;

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
			else if (pieces[k].equals("[no-out]"))
			{
				next.domains = next.domains & ~DOMAIN_OUTPUT;
			}
			else if (pieces[k].equals("[no-in]"))
			{
				next.domains = next.domains & ~DOMAIN_INPUT;
			}
			else if (pieces[k].equals("[hidden]"))
			{
				next.showInEditor = false;
			}
			else if (pieces[k].equals("[build-config]"))
			{
				next.isBuildConfig = true;
				next.domains = DOMAIN_INPUT;
				next.showInEditor = false;
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
					next.refType = typeName;
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

				boolean readParent = false;
				for (int j=readModifiers;curStruct != null&&j<pieces.length;j++)
				{
					if (pieces[j].equals(":"))
					{
						readParent = true;
					}
					else if (readParent)
					{
						curStruct.parent = pieces[j];
						readParent = false;
					}
					else if (pieces[j].equals("rtti"))
					{
						curStruct.isTypeRoot = true;
					}
					else if (pieces[j].equals("no-auxptr"))
					{
						curStruct.permitAsAux = false;
					}
					else if (pieces[j].equals("no-asset"))
					{
						curStruct.permitAsAsset = false;
					}
					else if (pieces[j].equals("non-instantiable"))
					{
						curStruct.permitAsAux = false;
						curStruct.permitAsAsset = false;
					}
					else if (pieces[j].equals("no-out"))
					{
						curStruct.domains = curStruct.domains & ~DOMAIN_OUTPUT;
					}
				}
				if (readParent)
				{
					error(path, i, "Unexpected error; no parent followed.");
				}
			}
		}


		return tmp;
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
					else if (line.startsWith("config:"))
					{
						String target = line.substring(7);
						if (!allTargets.contains(target))
							allTargets.add(target);
					}
				}
			}

			scanTree(pt, start.resolve("src"), start.resolve("src"));
			allTrees.add(pt);
		}
		catch (java.io.IOException e)
		{

		}
		return true;
	}

	// Post processing
	public boolean resolve()
	{
		int unique_id = 1;
		for (ParsedTree tree : allTrees)
		{
			for (ParsedFile file : tree.parsedFiles)
			{
				for (ParsedStruct struct : file.structs)
				{
					struct.moduleName = tree.moduleName;
					struct.loaderName = tree.loaderName;
					struct.uniqueId = unique_id;
					allTypes.add(struct);
					if (typesByName.put(struct.name, struct) != null)
					{
						System.out.println("Error: Duplicate entries of struct " + struct.name + "!");
						return false;
					}
					if (struct.parent != null)
					{
						ParsedField parent = new ParsedField();
						parent.domains = struct.domains;
						parent.name = "parent";
						parent.type = FieldType.STRUCT_INSTANCE;
						parent.refType = struct.parent;
						struct.fields.add(0, parent);
					}
					for (int i=0;i<struct.fields.size();i++)
					{
						struct.fields.get(i).index = i;
					}
				}
				for (ParsedEnum e : file.enums)
				{
					e.loaderName = tree.loaderName;
					allEnums.add(e);
					if (enumsByName.put(e.name, e) != null)
					{
						System.out.println("Error: Duplicate entries of enum " + e.name + "!");
						return false;
					}
				}
			}
		}

		for (ParsedStruct struct : allTypes)
		{
			for (ParsedField field : struct.fields)
			{
				if (field.type == FieldType.ENUM)
				{
					field.resolvedEnum = enumsByName.get(field.refType);
					if (field.resolvedEnum == null)
					{
						System.out.println("Unresolved enum name [" + field.refType + "] in field " + struct.name + "." + field.name);
						return false;
					}
				}
				else if (field.refType != null)
				{
					field.resolvedRefStruct = typesByName.get(field.refType);
					if (field.resolvedRefStruct == null)
					{
						System.out.println("Unresolved type name [" + field.refType + "] in field " + struct.name + "." + field.name);
						return false;
					}
				}
			}
			if (struct.parent != null)
			{
				struct.resolvedParent = typesByName.get(struct.parent);
				if (struct.resolvedParent == null)
				{
					System.out.println("Unresolved parent name [" + struct.parent + "] in struct " + struct.name);
					return false;
				}
			}
		}

		return true;
	}

	public boolean compile(Path start)
	{
		if (!scanPath(start))
			return false;
		if (!resolve())
			return false;
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
