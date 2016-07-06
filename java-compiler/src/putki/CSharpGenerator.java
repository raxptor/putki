package putki;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashSet;

import putki.Compiler.ParsedEnum;
import putki.Compiler.ParsedField;
import putki.Compiler.ParsedFile;
import putki.Compiler.ParsedStruct;
import putki.Compiler.ParsedTree;;

public class CSharpGenerator
{
	public static void writeParserList(StringBuilder sb, ParsedTree tree, HashSet<ParsedTree> included)
	{
		if (included != null)
		{
			if (included.contains(tree))
			{
				return;
			}
			for (ParsedTree dep : tree.deps.values())
			{
				writeParserList(sb, dep, included);
			}
		}
		for (ParsedFile file : tree.parsedFiles)
		{
			for (ParsedStruct struct : file.structs)
			{
				if ((struct.domains & Compiler.DOMAIN_INPUT) == 0 ||
					(struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
						continue;
				sb.append("\t\t\tnew SourceLoader.Parser(\"" + struct.name + "\", " + struct.name + "Fn),\n");
			}
			for (ParsedEnum e : file.enums)
			{
				sb.append("\t\t\tnew SourceLoader.Parser(\"" + e.name + "\", " + e.name + "EnumFn),\n");
			}
		}
	}

	public static String csharpType(Compiler.ParsedField field, boolean arrayForm)
	{
		if (!arrayForm)
		{
			switch (field.type)
			{
				case BOOL: return "bool";
				case UINT32: return "uint";
				case INT32: return "int";
				case BYTE: return "byte";
				case STRING: return "string";
				case POINTER: return "outki." + field.refType;
				case STRUCT_INSTANCE: return "outki." + field.refType;
				case ENUM: return "outki." + field.refType;
				case FILE: return "string";
				case FLOAT: return "float";
				case PATH: return "string";
			}
		}
		else
		{
			if (arrayForm && field.isArray)
			{
				return csharpType(field, false) + "[]";
			}
			else
			{
				return csharpType(field, false);
			}
		}
		return "HELP ME // CsharpGenerator.cs!";
	}

	public static void writeExpr(StringBuilder sb, String src, Compiler.ParsedField field)
	{
		switch (field.type)
		{
			case STRING:
				sb.append(src + ".ToString()");
				break;
			case POINTER:
				sb.append("loader.Resolve<outki." + field.refType + ">(path, " + src + ".ToString())");
				break;
			case INT32:
				sb.append("int.Parse(" + src + ".ToString())");
				break;
			case UINT32:
				sb.append("uint.Parse(" + src + ".ToString())");
				break;
			case BYTE:
				sb.append("byte.Parse(" + src + ".ToString())");
				break;
			case BOOL:
				sb.append("int.Parse(" + src + ".ToString()) != 0");
				break;
			case FLOAT:
				sb.append("float.Parse(" + src + ".ToString())");
				break;
			case STRUCT_INSTANCE:
				sb.append(field.resolvedRefStruct.loaderName + "." + field.resolvedRefStruct.name + "Fn(loader, path, " + src + ") as outki." + field.resolvedRefStruct.name);
				break;
			case ENUM:
				sb.append("(outki." + field.resolvedEnum.name + ")" + field.resolvedEnum.loaderName + "." + field.resolvedEnum.name + "EnumFn(loader, path, " + src + ")");
				break;
			default:
				sb.append("0 /* TODO: Implement me */");
				break;
		}
	}

	public static void writeParsers(StringBuilder sb, ParsedTree tree)
	{
		for (ParsedFile file : tree.parsedFiles)
		{
			for (ParsedStruct struct : file.structs)
			{
				if ((struct.domains & Compiler.DOMAIN_INPUT) == 0 ||
					(struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
					continue;

				String npfx = "\n\t\t\t";
				String outki = "outki." + struct.name;
				sb.append("\n");
				sb.append("\t\tstatic object " + struct.name + "Fn(SourceLoader loader, string path, object obj) {");
				sb.append(npfx).append(outki + " target = new " + outki + "();");
				sb.append(npfx).append("return " + struct.name + "ParseInto(loader, path, obj as MicroJson.Object, target);");
				sb.append("\n\t\t}\n");
				sb.append("\n\t\tstatic outki." + struct.name + " " + struct.name + "ParseInto(SourceLoader loader, string path, object src, outki." + struct.name + " target) {");
				sb.append(npfx).append("MicroJson.Object source = src as MicroJson.Object;");

				for (ParsedField fld : struct.fields)
				{
					if ((fld.domains & Compiler.DOMAIN_OUTPUT) == 0)
					{
						continue;
					}
					if ((fld.domains & Compiler.DOMAIN_INPUT) == 0)
					{
						continue;
					}
					String tmp = "__" + fld.name;
					String ref = "target." + fld.name;

					if (fld.isParentField)
					{
						sb.append(npfx).append("object parentObj;");
						sb.append(npfx).append("if (source.Data.TryGetValue(\"" + fld.name + "\", out parentObj))");
						sb.append(npfx).append("{");
						sb.append(npfx).append("\t" + struct.resolvedParent.loaderName + "." + struct.resolvedParent.name + "ParseInto(loader, path, parentObj, target);");
						sb.append(npfx).append("}");
						continue;
					}
					if (!fld.isArray)
					{
						sb.append(npfx).append("object " + tmp + "Obj;");
						sb.append(npfx).append("if (source.Data.TryGetValue(\"" + fld.name + "\", out " + tmp + "Obj))");
						sb.append(npfx).append("{");
						sb.append(npfx).append("\t" + ref + " = ");
						writeExpr(sb, tmp + "Obj", fld);
						sb.append(";");
						sb.append(npfx).append("}");
					}
					else
					{
						String arrTmp = "__Arr" + tmp;
						sb.append(npfx).append("object " + tmp + "Obj;");
						sb.append(npfx).append("List<" + csharpType(fld, false) + "> " + arrTmp + " = new List<" + csharpType(fld, false) + ">();");
						sb.append(npfx).append("if (source.Data.TryGetValue(\"" + fld.name + "\", out " + tmp + "Obj))");
						sb.append(npfx).append("{");
						sb.append(npfx).append("\tMicroJson.Array array = " + tmp + "Obj as MicroJson.Array;");
						sb.append(npfx).append("\tfor (int i=0;i<array.Data.Count;i++)");
						sb.append(npfx).append("\t{");
						sb.append(npfx).append("\t\t" + arrTmp + ".Add(");
						writeExpr(sb, "array.Data[i]", fld);
						sb.append(");");
						sb.append(npfx).append("\t}");
						sb.append(npfx).append("}");
						sb.append(npfx).append(ref + " = " + arrTmp + ".ToArray();");
					}
				}
				sb.append(npfx).append("return target;\n");
				sb.append("\t\t}\n");
			}
			for (ParsedEnum e : file.enums)
			{
				sb.append("\n");
				sb.append("\t\tstatic object " + e.name + "EnumFn(SourceLoader loader, string path, object obj)");
				sb.append("\n\t\t{");
				String npfx = "\n\t\t\t";
				sb.append(npfx).append("string tmp = obj.ToString();");
				for (Compiler.EnumValue v : e.values)
				{
					sb.append(npfx).append("if (tmp == \"" + v.name + "\")");
					sb.append(npfx).append("{");
					sb.append(npfx).append("\treturn outki." + e.name + "." + v.name + ";");
					sb.append(npfx).append("}");
				}
				sb.append(npfx).append("return outki." + e.name + "." + e.values.get(0).name + ";");
				sb.append("\n\t\t}\n");
			}
		}
	}

	public static void generateMixkiParsers(Compiler comp, CodeWriter writer)
	{
		for (Compiler.ParsedTree tree : comp.allTrees())
		{
			Path mixki = tree.genCodeRoot.resolve("mixki");
			Path fn = mixki.resolve(tree.loaderName + ".cs");

			StringBuilder sb = new StringBuilder();
			sb.append("using Putki;\n");
			sb.append("using Mixki;\n");
			sb.append("using System.Collections.Generic;\n");
			sb.append("\n");
			sb.append("namespace Mixki\n");
			sb.append("{\n");
			sb.append("\tpublic static class " + tree.loaderName + "\n");
			sb.append("\t{\n");
			sb.append("\t\tpublic static SourceLoader.Parser[] Parsers = new SourceLoader.Parser[] {\n");
			writeParserList(sb,  tree, null);
			sb.append("\t\t};\n\n");
			sb.append("\t\tpublic static SourceLoader.Parser[] ParsersWithDeps = new SourceLoader.Parser[] {\n");
			writeParserList(sb,  tree, new HashSet<>());
			sb.append("\t\t};\n");
			writeParsers(sb, tree);
			sb.append("\t}\n");
			sb.append("}\n");
			writer.addOutput(fn, sb.toString().getBytes());
			sb.append("\t}");

		}
	}
}
