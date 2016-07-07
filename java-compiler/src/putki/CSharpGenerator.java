package putki;

import java.nio.file.Path;
import java.util.HashSet;

import putki.Compiler.FieldType;
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
				case POINTER: return "Outki." + field.refType;
				case STRUCT_INSTANCE: return "Outki." + field.refType;
				case ENUM: return "Outki." + field.refType;
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
				sb.append("loader.Resolve<Outki." + field.refType + ">(path, " + src + ".ToString())");
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
				sb.append("(Outki." + field.resolvedRefStruct.name + ")" + field.resolvedRefStruct.loaderName + "." + field.resolvedRefStruct.name + "Fn(loader, path, " + src + ")");
				break;
			case ENUM:
				sb.append("(Outki." + field.resolvedEnum.name + ")" + field.resolvedEnum.loaderName + "." + field.resolvedEnum.name + "EnumFn(loader, path, " + src + ")");
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
				String outki = "Outki." + struct.name;
				sb.append("\n");
				sb.append("\t\tstatic object " + struct.name + "Fn(SourceLoader loader, string path, object obj) {");
				sb.append(npfx).append(outki + " target = new " + outki + "();");
				sb.append(npfx).append("return " + struct.name + "ParseInto(loader, path, obj as MicroJson.Object, target);");
				sb.append("\n\t\t}\n");
				sb.append("\n\t\tstatic Outki." + struct.name + " " + struct.name + "ParseInto(SourceLoader loader, string path, object src, Outki." + struct.name + " target) {");

				boolean first = true;
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

					if (first)
					{
						sb.append(npfx).append("MicroJson.Object source = src as MicroJson.Object;");
						first = false;
					}
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
					sb.append(npfx).append("\treturn Outki." + e.name + "." + v.name + ";");
					sb.append(npfx).append("}");
				}
				sb.append(npfx).append("return Outki." + e.name + "." + e.values.get(0).name + ";");
				sb.append("\n\t\t}\n");
			}
		}
	}

	public static void generateMixkiParsers(Compiler comp, CodeWriter writer)
	{
		for (Compiler.ParsedTree tree : comp.allTrees())
		{
			Path mixki = tree.genCodeRoot.resolve("csharp").resolve("mixki");
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

    static String sizeExpr(Compiler.ParsedField field)
    {
        switch (field.type)
        {
        	case FLOAT:
        	case ENUM:
        	case UINT32:
        	case INT32:
        		return "4";
        	case BYTE:
                return "1";
        	case POINTER:
        		return "2";
            case BOOL:
                return "1";
            case STRUCT_INSTANCE:
                return "LoadInfo_" + field.resolvedRefStruct.name + ".SIZE";
            case FILE:
            case STRING:
            case PATH:
                return "2";
        }
        return "ERROR";
    }

    static int fieldSize(Compiler.ParsedField field)
    {
        switch (field.type)
        {
        	case FLOAT:
        	case ENUM:
        	case UINT32:
        	case INT32:
        		return 4;
        	case BYTE:
                return 1;
        	case POINTER:
        		return 2;
            case BOOL:
                return 1;
            case FILE:
            case STRING:
            case PATH:
                return 2;
           default:
        	   return 0;
        }
    }

	public static void generateOutkiStructs(Compiler comp, CodeWriter writer)
	{
		for (Compiler.ParsedTree tree : comp.allTrees())
		{
			Path mixki = tree.genCodeRoot.resolve("csharp").resolve("outki");
			Path fn = mixki.resolve(tree.moduleName + ".cs");

			StringBuilder sb = new StringBuilder();
			sb.append("using System.Collections.Generic;\n");
			sb.append("\n");
			sb.append("namespace Outki\n");
			sb.append("{");
			for (Compiler.ParsedFile file : tree.parsedFiles)
			{
				sb.append("\n");
				String pfx = "\n\t";
				for (Compiler.ParsedEnum en : file.enums)
				{
					sb.append(pfx).append("public enum " + en.name);
					sb.append(pfx).append("{");
					String sep = "";
					for (Compiler.EnumValue value : en.values)
					{
						sb.append(sep + pfx).append("\t" + value.name + " = " + value.value);
						sep = ",";
					}
					sb.append(pfx).append("}");
				}

				for (Compiler.ParsedStruct struct : file.structs)
				{
					sb.append("\n");
					if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
						continue;

					if (struct.isValueType)
						sb.append(pfx).append("public struct " + struct.name);
					else
						sb.append(pfx).append("public class " + struct.name);

					if (struct.resolvedParent != null)
					{
						sb.append(" : " + struct.resolvedParent.name);
					}

					sb.append(pfx).append("{");

					String spfx = pfx + "\t";

					if (struct.resolvedParent != null)
					{
						sb.append(spfx).append("new public const int TYPE = " + struct.uniqueId + ";");
						sb.append(spfx).append("public " + struct.name + "()");
						sb.append(spfx).append("{");
						sb.append(spfx).append("\t_rtti_type = " + struct.uniqueId + ";");
						sb.append(spfx).append("}");
					}
					else
					{
						sb.append(spfx).append("public const int TYPE = " + struct.uniqueId + ";");
					}

					if (struct.isTypeRoot)
					{
						sb.append(spfx).append("public int _rtti_type = " + struct.uniqueId + ";");
					}


					for (Compiler.ParsedField field : struct.fields)
					{
						if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
							continue;
						sb.append(spfx).append("public " + csharpType(field, true) + " " + field.name + ";");
						if (field.type == FieldType.POINTER)
						{
							sb.append(spfx).append("public int" + (field.isArray ? "[]" : "") + " __slot_" + field.name + ";");
						}
					}
					sb.append(pfx).append("}");
				}
			}

			sb.append("\n}\n");
			writer.addOutput(fn, sb.toString().getBytes());
		}
	}

	public static void generateOutkiDataLoader(Compiler comp, CodeWriter writer)
	{
		for (Compiler.ParsedTree tree : comp.allTrees())
		{
			Path mixki = tree.genCodeRoot.resolve("csharp").resolve("outki");
			Path fn = mixki.resolve(tree.moduleName + "Loader.cs");

			StringBuilder sb = new StringBuilder();
			sb.append("using System.Collections.Generic;\n");
			sb.append("\n");
			sb.append("namespace Outki\n");
			sb.append("{\n");
			sb.append("namespace Loader\n");
			sb.append("{");
			sb.append("\n\tpublic static class " + tree.loaderName);
			sb.append("\n\t{");
			for (Compiler.ParsedFile file : tree.parsedFiles)
			{
				String pfx = "\n\t\t";
				for (Compiler.ParsedStruct struct : file.structs)
				{
					if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
						continue;

					sb.append(pfx).append("public static class LoadInfo_" + struct.name);
					sb.append(pfx).append("{");

					String sizeExtra = "";
					int size = 0;
					for (Compiler.ParsedField field : struct.fields)
					{
						if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
							continue;
						if (field.isParentField)
							continue;
						if (field.isArray)
							size += 6;
						else if (field.type != Compiler.FieldType.STRUCT_INSTANCE)
							size += fieldSize(field);
						else
							sizeExtra = sizeExtra + " + " + sizeExpr(field);
					}

					// Cannot have empty structs; putki will generate 1 byte.
					if (size == 0)
					{
						size = 1;
					}
					if (struct.isTypeRoot)
					{
						size = size + 4;
					}

					if (struct.resolvedParent != null)
					{
						sb.append(pfx).append("\tpublic const int SIZE = " + size + sizeExtra + " + LoadInfo_" + struct.resolvedParent.name + ".SIZE;");
					}
					else
					{
						sb.append(pfx).append("\tpublic const int SIZE = " + size + sizeExtra + ";");
					}

					sb.append(pfx).append("}");

					sb.append(pfx).append("public static " + struct.name + " LoadFromPackage_" + struct.name + "(Putki.PackageReader reader, Putki.PackageReader aux)");
					sb.append(pfx).append("{");
					sb.append(pfx).append("\t" + struct.name + " tmp = new " + struct.name + "();");
					sb.append(pfx).append("\tParseFromPackage_" + struct.name + "(ref tmp, reader, aux);");
					sb.append(pfx).append("\treturn tmp;");
					sb.append(pfx).append("}");

					sb.append(pfx).append("public static void ParseFromPackage_" + struct.name + "(ref " + struct.name + " target, Putki.PackageReader reader, Putki.PackageReader aux)");
					sb.append(pfx).append("{");

					String spfx = pfx + "\t";

					if (struct.isTypeRoot)
					{
						sb.append(spfx).append("target._rtti_type = reader.ReadInt32();");
					}

					for (Compiler.ParsedField field : struct.fields)
					{
						if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
							continue;

						String upfx = spfx;
						String ref = "target." + field.name;
						String contentReader = "reader";

						if (field.type == FieldType.POINTER)
						{
							ref = "target.__slot_" + field.name;
						}

						if (field.isArray)
						{
							sb.append(spfx).append("{");
							sb.append(spfx).append("\treader.ReadInt16();"); // read ptr.
							sb.append(spfx).append("\tint count = reader.ReadInt32();");
							if (field.type == FieldType.POINTER)
							{
								sb.append(spfx).append("\ttarget.__slot_" + field.name + " = new int[count];");
							}
							sb.append(spfx).append("\ttarget." + field.name + " = new " + csharpType(field,  false) + "[count];");
							ref = ref + "[i]";
							sb.append(spfx).append("\tPutki.PackageReader arrAux = aux.CloneAux(0);");
							sb.append(spfx).append("\taux.Skip(count * " + sizeExpr(field) + ");");
							sb.append(spfx).append("\tfor (int i=0;i!=count;i++)");
							sb.append(spfx).append("\t{");
							upfx = spfx + "\t\t";
							contentReader = "arrAux";
						}

						switch (field.type)
						{
							case INT32:
								sb.append(upfx).append(ref + " = " + contentReader + ".ReadInt32();");
								break;
							case UINT32:
								sb.append(upfx).append(ref + " = (uint)" + contentReader + ".ReadInt32();");
								break;
							case BYTE:
								sb.append(upfx).append(ref + " = " + contentReader + ".ReadByte();");
								break;
							case BOOL:
								sb.append(upfx).append(ref + " = " + contentReader + ".ReadByte() != 0;");
								break;
							case FLOAT:
								sb.append(upfx).append(ref + " = " + contentReader + ".ReadFloat();");
								break;
							case ENUM:
								sb.append(upfx).append(ref + " = (" + field.resolvedEnum.name + ") " + contentReader + ".ReadInt32();");
								break;
							case STRUCT_INSTANCE:
								sb.append(upfx).append(ref + " = LoadFromPackage_" + field.resolvedRefStruct.name + "(" + contentReader + ", aux);");
								break;
							case POINTER:
								sb.append(upfx).append(ref + " = " + contentReader + ".ReadInt16();");
								break;
							case STRING:
							case FILE:
							case PATH:
								sb.append(upfx).append(ref + " = aux.ReadString(" + contentReader + ".ReadInt16());");
								break;
							default:
								sb.append(upfx).append("// god help me");
								break;
						}

						if (field.isArray)
						{
							sb.append(spfx).append("\t}");
							sb.append(spfx).append("}");
						}
					}
					sb.append(pfx).append("}");
					sb.append(pfx).append("public static " + struct.name + " ResolveFromPackage_" + struct.name + "(" + struct.name + " target, Putki.Package pkg)");
					sb.append(pfx).append("{");

					for (Compiler.ParsedField field : struct.fields)
					{
						if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
							continue;
						if (field.isParentField)
							continue;
						if (field.type != Compiler.FieldType.POINTER && field.type != Compiler.FieldType.STRUCT_INSTANCE)
							continue;

						String ref = "target." + field.name;
						String slotRef = "target.__slot_" + field.name;

						String upfx = pfx + "\t";
						if (field.isArray)
						{
							sb.append(pfx).append("\tfor (int i=0;i<" + ref + ".Length;i++)");
							sb.append(pfx).append("\t{");
							upfx = pfx + "\t\t";
							ref = ref + "[i]";
							slotRef = slotRef + "[i]";
						}

						if (field.type == FieldType.POINTER)
						{
							sb.append(upfx).append(ref + " = pkg.ResolveSlot<" + field.resolvedRefStruct.name + ">(" + slotRef + ");");
						}
						else
						{
							sb.append(upfx).append(ref + " = ResolveFromPackage_" + field.resolvedRefStruct.name + "(" + ref + ", pkg);");
						}

						if (field.isArray)
						{
							sb.append(pfx).append("\t}");
							upfx = pfx + "\t";
						}
					}
					sb.append(pfx).append("\treturn target;");

					sb.append(pfx).append("}");
				}
			}

			sb.append("\n\t\tpublic static object ResolveFromPackage(int type, object obj, Putki.Package pkg)");
			sb.append("\n\t\t{");
			sb.append("\n\t\t\tswitch (type)");
			sb.append("\n\t\t\t{");
			for (Compiler.ParsedFile file : tree.parsedFiles)
			{
				String pfx = "\n\t\t\t\t";
				for (Compiler.ParsedStruct struct : file.structs)
				{
					if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
						continue;
					sb.append(pfx).append("case " + struct.name + ".TYPE: return ResolveFromPackage_" + struct.name + "((" + struct.name + ")obj, pkg);");
				}
			}
			sb.append("\n\t\t\t\tdefault: return obj;");
			sb.append("\n\t\t\t}");
			sb.append("\n\t\t}");
			sb.append("\n\t\tpublic static object LoadFromPackage(int type, Putki.PackageReader reader)");
			sb.append("\n\t\t{");
			sb.append("\n\t\t\tswitch (type)");
			sb.append("\n\t\t\t{");
			for (Compiler.ParsedFile file : tree.parsedFiles)
			{
				String pfx = "\n\t\t\t\t";
				for (Compiler.ParsedStruct struct : file.structs)
				{
					if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
						continue;
					sb.append(pfx).append("case " + struct.name + ".TYPE:");
					sb.append(pfx).append("{");
					sb.append(pfx).append("\tPutki.PackageReader aux = reader.CloneAux(LoadInfo_" + struct.name + ".SIZE);");
					sb.append(pfx).append("\tobject o = LoadFromPackage_" + struct.name + "(reader, aux);");
					sb.append(pfx).append("\treader.MoveTo(aux);");
					sb.append(pfx).append("\treturn o;");
					sb.append(pfx).append("}");
				}
			}
			sb.append("\n\t\t\t\tdefault: return null;");
			sb.append("\n\t\t\t}");
			sb.append("\n\t\t}");

			sb.append("\n\t}");
			sb.append("\n\t}");
			sb.append("\n}\n");
			writer.addOutput(fn, sb.toString().getBytes());
		}
	}
}
