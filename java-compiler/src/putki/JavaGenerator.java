package putki;

import java.nio.file.Path;

import putki.Compiler.FieldType;

public class JavaGenerator
{
	public static void generateEditorProxys(Compiler comp, CodeWriter writer)
	{
		for (Compiler.ParsedTree tree : comp.allTrees())
		{
			Path mixki = tree.genCodeRoot.resolve("java").resolve("putked").resolve("inki");
			Path fn = mixki.resolve(tree.moduleName + ".java");

			StringBuilder sb = new StringBuilder();
			sb.append("package putked.inki;");
			sb.append("\n");
			sb.append("\nimport putked.ProxyObject;");
			sb.append("\nimport putked.DataObject;");
			sb.append("\nimport putked.EditorTypeService;");
			sb.append("\n");
			sb.append("\npublic class " + tree.loaderName + " implements EditorTypeService");
			sb.append("\n{");

			for (Compiler.ParsedFile file : tree.parsedFiles)
			{
				String pfx = "\n\t";
				for (Compiler.ParsedEnum en : file.enums)
				{
					sb.append(pfx).append("public enum " + en.name);
					sb.append(pfx).append("{");
					String sep = "";
					for (Compiler.EnumValue value : en.values)
					{
						sb.append(sep + pfx).append("\t" + value.name + "(" + value.value + ")");
						sep = ",";
					}
					sb.append(";");
					sb.append(pfx).append("\tprivate int v;");
					sb.append(pfx).append("\tprivate " + en.name + "(int value) { v = value; };");
					sb.append(pfx).append("\tpublic int getValue() { return v; }");
					sb.append(pfx).append("}");
				}

				for (Compiler.ParsedStruct struct : file.structs)
				{
					if ((struct.domains & Compiler.DOMAIN_INPUT) == 0)
						continue;

					sb.append(pfx).append("public static class " + struct.name);

					if (struct.resolvedParent != null)
					{
						sb.append(" extends " + struct.resolvedParent.name);
					}
					else
					{
						sb.append(" implements putked.ProxyObject");
					}

					sb.append(pfx).append("{");

					String spfx = pfx + "\t";

					sb.append(spfx).append("public static final int TYPE = " + struct.uniqueId + ";");
					sb.append(spfx).append("public static final String NAME = \"" + struct.name + "\";");
					sb.append(spfx).append("public static putki.Compiler.ParsedStruct _getType() { return null; }");
					sb.append(spfx).append("public putked.DataObject m_dataObj;");

					sb.append(spfx).append("@Override");
					sb.append(spfx).append("public void connect(putked.DataObject obj)");
					sb.append(spfx).append("{");
					if (struct.resolvedParent != null)
					{
						sb.append(spfx).append("\tsuper.connect(obj);");
					}
					sb.append(spfx).append("\tm_dataObj = obj;");
					sb.append(spfx).append("}");

					for (Compiler.ParsedField field : struct.fields)
					{
						if ((struct.domains & Compiler.DOMAIN_INPUT) == 0)
							continue;

						String getFn = "";
						String setFn = "";
						String index = ", -1";

						if (field.isArray)
						{
							getFn = "int index";
							setFn = "int index, ";
							index = ", index";
						}

						String stdType = null;
						switch (field.type)
						{
							case BYTE:
							case INT32:
								stdType = "int";
								break;
							case UINT32:
								stdType = "long";
								break;
							case FLOAT:
								stdType = "float";
								break;
							case STRING:
							case FILE:
							case PATH:
								stdType = "String";
								break;
							default:
								break;
						}

						if (stdType != null)
						{
							sb.append(spfx).append("public " + stdType + " get" + field.name + "(" + getFn + ")");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\treturn (" + stdType + ")m_dataObj.getField(" + field.index + index + ");");
							sb.append(spfx).append("}");
							sb.append(spfx).append("public void set" + field.name + "(" + setFn + stdType + " value)");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\tm_dataObj.setField(" + field.index + index + ", value);");
							sb.append(spfx).append("}");
						}
						else if (field.type == FieldType.STRUCT_INSTANCE)
						{
							String rt = field.resolvedRefStruct.name;
							sb.append(spfx).append("public " + rt + " get" + field.name + "(" + getFn + ")");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\t" + rt + " proxy = new " + rt + "();");
							sb.append(spfx).append("\tproxy.connect((DataObject)m_dataObj.getField(" + field.index + index +"));");
							sb.append(spfx).append("\treturn proxy;");
							sb.append(spfx).append("}");
							sb.append(spfx).append("public void set" + field.name + "(" + setFn + rt + " value)");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\tm_dataObj.setField(" + field.index + index + ", value.m_dataObj);");
							sb.append(spfx).append("}");
						}

						if (field.isArray)
						{
							sb.append(spfx).append("public int get" + field.name + "Size()");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\treturn m_dataObj.getArraySize(" + field.index + ");");
							sb.append(spfx).append("}");
							sb.append(spfx).append("public void erase" + field.name + "(int index)");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\tm_dataObj.arrayErase(" + field.index + ", index);");
							sb.append(spfx).append("}");
							sb.append(spfx).append("public void insert" + field.name + "(int index)");
							sb.append(spfx).append("{");
							sb.append(spfx).append("\tm_dataObj.arrayInsert(" + field.index + ", index);");
							sb.append(spfx).append("}");
						}
					}
					sb.append(pfx).append("}");
				}
			}

			sb.append("\n\tpublic ProxyObject createProxy(String type)");
			sb.append("\n\t{");
			for (Compiler.ParsedFile file : tree.parsedFiles)
			{
				for (Compiler.ParsedStruct struct : file.structs)
				{
					sb.append("\n\t\tif (type.equals(\"" + struct.name + "\"))");
					sb.append("\n\t\t\treturn new " + struct.name + "();");
				}
			}
			sb.append("\n\t\treturn null;");
			sb.append("\n\t}");
			sb.append("\n}\n");
			writer.addOutput(fn, sb.toString().getBytes());
		}
	}
}
