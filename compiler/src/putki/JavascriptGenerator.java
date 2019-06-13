package putki;

import java.nio.file.Path;
import java.util.HashSet;

import putki.Compiler.FieldType;
import putki.Compiler.ParsedEnum;
import putki.Compiler.ParsedField;
import putki.Compiler.ParsedFile;
import putki.Compiler.ParsedStruct;
import putki.Compiler.ParsedTree;
import putki.CppGenerator.Platform;

public class JavascriptGenerator
{
    public static void generateDescriptors(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
	        generateTypeDescriptors(comp, tree, writer);
        }
    }

	public static String capsToCamelCase(String name)
	{
		StringBuilder sb = new StringBuilder();
		boolean word = true;
		for (int i=0;i<name.length();i++)
		{
			if (name.charAt(i) == '_')
			{
				word = true;
				continue;
			}
			if (word)
				sb.append(Character.toUpperCase(name.charAt(i)));
			else
				sb.append(Character.toLowerCase(name.charAt(i)));
			word = false;
		}
		return sb.toString();
	}

	public static String fmtFloat(String value)
	{
		if (value == null || value.length() == 0)
			return "0f32";
		return value.replace("f", "") + "f32";
	}

	public static String enumName(Compiler.ParsedEnum e)
	{
		return e.name;
	}

    public static void generateTypeDescriptors(Compiler comp, Compiler.ParsedTree tree, CodeWriter writer)
    {
        Path lib = tree.genCodeRoot.resolve("js");
        Path fn = lib.resolve("types.js");

        StringBuilder sb = new StringBuilder();
        sb.append("exports.Types = {\n");

        boolean ftype = true;
        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
        	for (Compiler.ParsedEnum e : file.enums)
        	{
        		if (!ftype)
        			sb.append(",");
        		ftype = false;
        		sb.append("\n\t\"" + e.name.toLowerCase() + "\": {");
           		String prefix = "\n\t\t\t";
            	sb.append("\n\t\tPrettyName:\"" + e.name + "\"");
    			sb.append(",\n\t\tValues: [");
        		boolean first = true;
        		for (Compiler.EnumValue val : e.values)
        		{
        			if (!first) sb.append(",");
        			sb.append(prefix).append("{ Name:\"" + val.name + "\", PrettyName:\"" + val.name + "\", Value:" + val.value + " }");
        			first = false;
        		}
    			sb.append("\n\t\t]");
           		sb.append("\n\t}");
        	}

    		for (Compiler.ParsedStruct struct : file.structs)
    		{
        		if (!ftype)
        			sb.append(",");
        		ftype = false;

        		sb.append("\n\t\"" + struct.name.toLowerCase() + "\": {");
           		String prefix = "\n\t\t\t";

            	sb.append("\n\t\tPrettyName:\"" + struct.name + "\"");
           		sb.append(",\n\t\tPermitAsAsset:" + struct.permitAsAsset);
           		sb.append(",\n\t\tIsTypeRoot:" + struct.isTypeRoot);
           		sb.append(",\n\t\tIsValueType:" + struct.isValueType);
           		sb.append(",\n\t\tRequirePath:" + struct.requirePath);
           		if (struct.resolvedParent != null)
           			sb.append(",\n\t\tParent:\""+  struct.resolvedParent.name.toLowerCase() + "\"");
       			sb.append(",\n\t\tFields: [");

       			boolean ff = true;
                for (Compiler.ParsedField field : struct.fields)
                {
                	if (field.isParentField)
                		continue;
                	if (!ff)
                		sb.append(",");
                	ff = false;
                	sb.append(prefix).append("{ Name:\"" + field.name.toLowerCase() + "\", Type:\"");
                	switch (field.type)
                	{
                		case INT32: sb.append("I32"); break;
                		case UINT32: sb.append("U32"); break;
                		case FLOAT: sb.append("Float"); break;
                		case BYTE: sb.append("U8"); break;
                		case STRING: if (field.stringIsText) sb.append("Text"); else sb.append("String"); break;
                		case BOOL: sb.append("Bool"); break;
                		case ENUM: sb.append(field.resolvedEnum.name.toLowerCase()); break;
                		case STRUCT_INSTANCE: sb.append(field.resolvedRefStruct.name.toLowerCase()); break;
                		case HASH: sb.append("Hash"); break;
                		case POINTER: sb.append(field.resolvedRefStruct.name.toLowerCase()); break;
                		default: sb.append("Unknown"); break;
                	}
                	sb.append("\"");

                	boolean ptr = field.type == FieldType.POINTER;

                	sb.append(", PrettyName:\"" + field.name + "\"");
                	sb.append(", Array:" + field.isArray + ", Pointer:" + ptr);

                	if (field.localizationCategory != null)
                		sb.append(", LocalizationCategory: \"" + field.localizationCategory + "\"");

                	if (field.defValue != null)
                	{
                		if (field.defValue.startsWith("\""))
                    		sb.append(", Default:" + field.defValue);
                		else
                			sb.append(", Default:\"" + field.defValue + "\"");
                	}

                	sb.append(" }");
                }
    			sb.append("\n\t\t]");
           		sb.append("\n\t}");
    		}
        }
        sb.append("\n};\n");
        writer.addOutput(fn, sb.toString().getBytes());
    }
}

