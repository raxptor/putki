package putki;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.nio.file.Path;
import java.util.List;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;

public class Compiler
{
	public static int DOMAIN_INPUT  = 1;
	public static int DOMAIN_OUTPUT = 2;
	public static int DOMAIN_NETKI  = 4;

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
		public boolean allowNull;
		public boolean isAuxPtr;
		public boolean isBuildConfig;
		public boolean showInEditor;
		public boolean isParentField;
		public FieldType type;
		public String name;
		public String refType;
		public String defValue;
		public ParsedStruct resolvedRefStruct;
		public ParsedEnum resolvedEnum;
		public ParsedField buildConfigTargetField;
		public boolean stringIsText;
		public String localizationCategory;
		public boolean localizationPlural;
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
		public boolean isTypeRoot;
		public boolean isValueType;
		public boolean permitAsAux;
		public boolean permitAsAsset;

		public ParsedStruct resolvedParent;
		public List<ParsedStruct> possibleChildren;

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
		public Integer value;
	}

	public class ParsedEnum
	{
		public String loaderName;
		public String name;
		public List<EnumValue> values;
	}

	public class ParsedFile
	{
		public String sourcePath;
		public String fileName;
		public String moduleName;
		public String signature;
		public List<ParsedStruct> structs;
		public List<ParsedEnum> enums;
		public List<String> includes;
	};

	public class ParsedTree
	{
		public String moduleName;
		public String loaderName;
		public String typeFileEnding;
		public Path genCodeRoot;
		public List<ParsedFile> parsedFiles;
		public HashMap<String, ParsedTree> deps;
	}

	List<ParsedStruct> allTypes = new ArrayList<Compiler.ParsedStruct>();
	List<ParsedEnum> allEnums = new ArrayList<Compiler.ParsedEnum>();
	HashMap<String, ParsedStruct> typesByName = new HashMap<String, ParsedStruct>();
	HashMap<String, ParsedEnum> enumsByName = new HashMap<String, ParsedEnum>();
	List<ParsedTree> allTrees = new ArrayList<Compiler.ParsedTree>();
	HashMap<String, ParsedTree> allModules = new HashMap<String, ParsedTree>();
	List<String> buildConfigs = new ArrayList<String>();

	public boolean mixkiOnly = false;
	
	public void error(String path, int line, String err)
	{
		System.out.println(path + ":" + line + " Error! " + err);
	}

	public List<ParsedTree> allTrees()
	{
		return allTrees;
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
				cur.values.add(val);
				return true;
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

		if (val.name != null)
		{
			cur.values.add(val);
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

				if (pieces[k].length() > 2 && pieces[k].charAt(0) == '{' && pieces[k].charAt(pieces[k].length()-1) == '}')
				{
					next.localizationCategory = pieces[k].substring(1,  pieces[k].length() - 1);
					continue;
				}
				else if (pieces[k].length() > 3 && pieces[k].charAt(0) == '{' && pieces[k].charAt(pieces[k].length()-2) == '}' && pieces[k].charAt(pieces[k].length()-1) == '+')
				{
					next.localizationCategory = pieces[k].substring(1,  pieces[k].length() - 2);
					next.localizationPlural = true;
					continue;
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
				if (typeName.equals("text"))
				{
					next.type = FieldType.STRING;
					next.stringIsText = true;
					gotType = true;
				}
				if (typeName.equals("ptr") || typeName.equals("ptr!"))
				{
					next.type = FieldType.POINTER;
					next.allowNull = typeName.equals("ptr");
					readRefType = true;
					gotType = true;
				}
				else if (typeName.equals("auxptr") || typeName.equals("auxptr!"))
				{
					next.type = FieldType.POINTER;
					next.isAuxPtr = true;
					next.allowNull = typeName.equals("auxptr");
					readRefType = true;
					gotType = true;
				}
				else if (typeName.equals("enum"))
				{
					next.type = FieldType.ENUM;
					readRefType = true;
					gotType = true;
				}
				else if (typeName.equals("file"))
				{
					next.type = FieldType.FILE;
					gotType = true;
				}
				else if (typeName.equals("path"))
				{
					next.type = FieldType.FILE;
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
				// HACK
				if (next.defValue == null || next.defValue.length() == 0)
					next.defValue = pieces[k];
				else
					next.defValue = next.defValue + " " + pieces[k];
			}
			else if (next.name == null)
			{
				next.name = pieces[k];
			}
			else
			{
				System.out.println("Unexpected token when parsing field. Line [" + line + "] piece=[" + pieces[k] + "]");
				return false;
			}
		}

		if (gotType && !readRefType)
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

		int l0 = path.lastIndexOf('/');
		int l1 = path.lastIndexOf('\\');
		int w = l0 > l1 ? l0 : l1;
		if (w != -1)
		{
			tmp.fileName = path.substring(w+1);
			tmp.sourcePath = path.substring(0, w);
		}
		else
		{
			tmp.fileName = path;
			tmp.sourcePath = "";
		}

		int k = tmp.fileName.lastIndexOf('.');
		if (k != -1)
		{
			tmp.fileName = tmp.fileName.substring(0, k);
		}

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
				String inc = line.substring(9);
				int lastDot = inc.lastIndexOf('.');
				if (lastDot != -1)
					inc = inc.substring(0, lastDot);
				tmp.includes.add("$PFX$" + inc);
				continue;
			}

			if (line.startsWith("#include"))
			{
				String inc = line.substring(9);
				int lastDot = inc.lastIndexOf('.');
				if (lastDot != -1)
					inc = inc.substring(0, lastDot);
				tmp.includes.add(inc);
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
							int max = -1;
							for (EnumValue en : curEnum.values)
							{
								if (en.value != null && en.value > max)
									max = en.value;
							}
							for (EnumValue en : curEnum.values)
							{
								if (en.value == null)
									en.value = ++max;
							}

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
						curStruct.domains = DOMAIN_INPUT | DOMAIN_OUTPUT;
						curStruct.fields = new ArrayList<ParsedField>();
						curStruct.permitAsAsset = true;
						curStruct.permitAsAux = true;
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
					else if (pieces[j].equals("non-instantiable") || pieces[j].equals("no-instance"))
					{
						curStruct.permitAsAux = false;
						curStruct.permitAsAsset = false;
					}
					else if (pieces[j].equals("value-type"))
					{
						curStruct.permitAsAsset = false;
						curStruct.isValueType = true;
					}
					else if (pieces[j].equals("no-out"))
					{
						curStruct.domains = curStruct.domains & ~DOMAIN_OUTPUT;
					}
					else if (pieces[j].equals("@netki"))
					{
						curStruct.domains = DOMAIN_NETKI;
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
		System.out.println("Scanning tree [" + where + "]");
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

	public ParsedTree scanModule(Path start)
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
			pt.genCodeRoot = start.resolve("_gen");
			pt.deps = new HashMap<>();

			if (lines.size() > 0 && !lines.get(0).trim().equals("config-version:1.0"))
			{
				if (lines.size() > 1)
				{
					pt.moduleName = lines.get(0);
					pt.loaderName = lines.get(1);
				}
			}

			String sourceFolder = "src";

			for (int i=0;i<lines.size();i++)
			{
				String line = lines.get(i);
				if (line.length() > 4)
				{
					if (line.startsWith("dep:"))
					{
						String name = line.substring(4);
						ParsedTree module = scanModule(start.resolve(name));
						if (module == null)
						{
							return null;
						}
						pt.deps.put(module.moduleName, module);
					}
					else if (line.startsWith("config:"))
					{
						String target = line.substring(7);
						if (!buildConfigs.contains(target))
							buildConfigs.add(target);
					}
					else if (line.startsWith("genpath:"))
					{
						pt.genCodeRoot = start.resolve(line.substring(8));
					}
					else if (line.startsWith("name:"))
					{
						pt.moduleName = line.substring(5);
						pt.loaderName = line.substring(5);
					}
					else if (line.startsWith("mixki-only:"))
					{
						mixkiOnly = Boolean.parseBoolean(line.substring(11));
					}
					else if (line.startsWith("src:"))
					{
						sourceFolder = line.substring(4);
					}
				}
			}

			// Don't scan again.
			if (allModules.containsKey(pt.moduleName))
			{
				return allModules.get(pt.moduleName);
			}

			Path startPath = start.resolve(sourceFolder);
			scanTree(pt, startPath, startPath);
			allTrees.add(pt);
			return pt;
		}
		catch (java.io.IOException e)
		{
			System.out.println("Error " + e.toString());
		}
		return null;
	}

	// Post processing
	public boolean resolve()
	{
		int unique_id = 1;

		for (ParsedTree tree : allTrees)
		{
			tree.parsedFiles.sort(new Comparator<ParsedFile>() {
				@Override
				public int compare(ParsedFile o1, ParsedFile o2) {
					String c1 = o1.sourcePath + "/" + o1.fileName;
					String c2 = o2.sourcePath + "/" + o2.fileName;
					return c1.compareTo(c2);
				}
			});
			for (ParsedFile file : tree.parsedFiles)
			{
				file.moduleName = tree.moduleName;
				for (ParsedStruct struct : file.structs)
				{
					if (struct.isValueType)
					{
						if (struct.isTypeRoot)
						{
							System.out.println("Type " + struct.name + " is value type; cannot be rtti");
							return false;
						}
						if (struct.resolvedParent != null)
						{
							System.out.println("Type " + struct.name + " is value type; cannot have a parent.");
							return false;
						}
					}

					struct.moduleName = tree.moduleName;
					struct.loaderName = tree.loaderName;
					struct.uniqueId = unique_id++;
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
						parent.isParentField = true;
						parent.type = FieldType.STRUCT_INSTANCE;
						parent.refType = struct.parent;
						parent.showInEditor = true;
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

				// For all build config fields; spawn separate ones.
				for (ParsedStruct struct : file.structs)
				{
					for (int i=0;i<struct.fields.size();i++)
					{
						ParsedField field = struct.fields.get(i);
						if (field.isBuildConfig)
						{
							if (field.domains != DOMAIN_INPUT)
							{
								System.out.println("Warning: Forcing DOMAIN_INPUT on field " + field.name);
								field.domains = DOMAIN_INPUT;
							}
							for (String config : buildConfigs)
							{
								ParsedField np = new ParsedField();
								np.name = field.name + config;
								np.type = field.type;
								np.defValue = field.defValue;
								np.domains = field.domains;
								np.showInEditor = true;
								np.index = struct.fields.size();
								np.isArray = field.isArray;
								np.isAuxPtr = field.isAuxPtr;
								np.refType = field.refType;
								np.resolvedRefStruct = field.resolvedRefStruct;
								np.resolvedEnum = field.resolvedEnum;
								np.domains = DOMAIN_INPUT;
								struct.fields.add(np);
							}
						}
					}
				}
			}
		}

		for (ParsedStruct struct : allTypes)
		{
			struct.possibleChildren = new ArrayList<ParsedStruct>();
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
		
		for (ParsedStruct struct : allTypes)
		{
			if (struct.parent != null)
			{
				struct.resolvedParent.possibleChildren.add(struct);
			}
		}
		
		return true;
	}

	public boolean compile(Path start)
	{
		buildConfigs.add("Default");
		ParsedTree root = scanModule(start);
		if (root == null)
			return false;
		if (!resolve())
			return false;
		return true;
	}


	public static void main(String [] args)
	{
		Compiler c = new Compiler();
		if (args.length > 0)
		{
			if (!c.compile(Paths.get(args[0])))
			{
				return;
			}
		}
		else
		{
			if (!c.compile(Paths.get(".")))
			{
				return;
			}
		}

		CodeWriter writer = new CodeWriter();
		CSharpGenerator.generateMixkiParsers(c, writer);
		CSharpGenerator.generateOutkiStructs(c, writer);
		if (!c.mixkiOnly)
		{
			CSharpGenerator.generateOutkiDataLoader(c, writer);
			CSharpGenerator.generateNetkiStructs(c, writer);
		}
		JavaGenerator.generateEditorProxys(c, writer);
		CppGenerator.generateInkiHeader(c, writer);
		CppGenerator.generateInkiImplementation(c, writer);
		CppGenerator.generateOutkiHeader(c, writer);
		CppGenerator.generateOutkiImplementation(c, writer);
		RustGenerator.generateCrate(c, writer);
		writer.write();
	}
}

