package putki;

import java.nio.file.Path;

import putki.Compiler.FieldType;

public class JavaGenerator
{
	public static void generateEditorProxys(Compiler comp, CodeWriter writer)
	{
		for (Compiler.ParsedTree tree : comp.allTrees())
		{
			Path mixki = tree.genCodeRoot.resolve("java").resolve("editor");
			Path fn = mixki.resolve(tree.moduleName + ".java");

			StringBuilder sb = new StringBuilder();
			sb.append("package inki;\n");
			sb.append("\n");
			sb.append("import putked.DataObject");
			/*
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
			}*/

			sb.append("\n}\n");
			writer.addOutput(fn, sb.toString().getBytes());
		}
	}
}
