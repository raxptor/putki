package putki;

import java.nio.file.Path;
import java.util.HashSet;

import putki.Compiler.FieldType;
import putki.Compiler.ParsedEnum;
import putki.Compiler.ParsedField;
import putki.Compiler.ParsedFile;
import putki.Compiler.ParsedStruct;
import putki.Compiler.ParsedTree;
import putki.CppGenerator.Platform;;

public class RustGenerator
{
	public static String withUnderscore(String input)
	{
		return withUnderscore(input,  false);
	}

	public static String withUnderscore(String input, boolean caps)
	{
		StringBuilder sb = new StringBuilder();
		boolean[] upc = new boolean[input.length()];

		for (int i=0;i<input.length();i++)
		{
			upc[i] = Character.isUpperCase(input.charAt(i));
		}

		int inrow = 0;
		for (int i=0;i<input.length();i++)
		{
			if (upc[i])
			{
				inrow++;
			}
			else
			{
				if (inrow > 2)
				{
					for (int k=0;k<(inrow-2);k++)
					{
						upc[i-2-k] = false;
					}
				}
				inrow = 0;
			}
		}

		for (int i=0;i<input.length();i++)
		{
			if (i > 0 && upc[i])
			{
				if (sb.length() > 0 && sb.charAt(sb.length()-1) != '_')
					sb.append('_');
			}

			char c = input.charAt(i);
			if (c == '_')
			{
				if (sb.length() > 0 && sb.charAt(sb.length()-1) == '_')
					continue;
			}

			if (caps)
			{
				sb.append(Character.toUpperCase(c));
			}
			else
			{
				sb.append(Character.toLowerCase(c));
			}
		}
		return sb.toString();
	}	
	

	public static String structName(Compiler.ParsedStruct s)
	{
		return s.name;
	}

	public static String fieldName(Compiler.ParsedField s)
	{
		return withUnderscore(s.name);
	}

	public static String enumName(Compiler.ParsedEnum s)
	{
		return s.name;
	}

	static String outkiFieldtypePod(Compiler.FieldType f)
	{
		switch (f)
		{
			case FILE:
				return "String";
			case STRING:
			case PATH:
				return "String";
			case INT32:
				return "i32";
			case UINT32:
				return "u32";
			case BYTE:
				return "u8";
			case FLOAT:
				return "f32";
			case BOOL:
				return "bool";
			default:
				return "<error>";
		}
	}
	
	static String outkiFieldType(Compiler.ParsedField pf)
	{		
		if (pf.type == FieldType.STRUCT_INSTANCE)
		{
			return structName(pf.resolvedRefStruct);
		}
		else if (pf.type == FieldType.POINTER)
		{
			return "rc::Rc<" + structName(pf.resolvedRefStruct) + ">";
		}		
		else
		{
			return outkiFieldtypePod(pf.type);
		}
	}	
	
	/*
	public static String enumValue(Compiler.EnumValue s)
	{
		return enumValue(s.name);
	}*/	

    public static void generateOutkiStructs(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
            Path lib = tree.genCodeRoot.resolve("rust").resolve("src");
            Path fn = lib.resolve("lib.rs");
            StringBuilder sb = new StringBuilder();
            sb.append("pub mod mixki\n{");
            sb.append("\n\tuse std::rc;");
            sb.append("\n\tuse std::vec;");
            

            for (Compiler.ParsedFile file : tree.parsedFiles)
            {
            	/*
            	String pfx = "\n\t";
                for (Compiler.ParsedEnum en : file.enums)
                {
                	int min = 0;
                	int max = 0;
                    for (Compiler.EnumValue value : en.values)                    	
                    {
                    	if (value.value < min) min = value.value;
                    	if (value.value > max) max = value.value;
                    }
                	
                    sb.append(pfx).append("public enum " + en.name);
                    if (max < 256 && min >= 0)
                    	sb.append(": byte");
                    sb.append(pfx).append("{");
                    String sep = "";
                    for (Compiler.EnumValue value : en.values)
                    {
                        sb.append(sep + pfx).append("\t" + value.name + " = " + value.value);
                        sep = ",";
                    }
                    sb.append(pfx).append("}");
                }
            	 */
                for (Compiler.ParsedStruct struct : file.structs)
                {
                	String pfx = "\n\t";                
                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                        continue;
                    sb.append(pfx).append("pub struct " + structName(struct));
                    //if (struct.parent != null)
//                    	sb.append("<'parent>");
//                    if (struct.isTypeRoot)
//                    	sb.append("<'me>");
                    sb.append(pfx).append("{");
                    
                    boolean first = true;
                    for (Compiler.ParsedField field : struct.fields)
                    {
                    	String spfx = pfx + "\t";
                        if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
                            continue;
                        if (!first)
                        	sb.append(",");
                        first = false;
                        sb.append(spfx).append("pub " + fieldName(field) + " : ");
//                        if (field.isParentField)
//                        	sb.append("&mut ");
                        if (field.isArray) sb.append("vec::Vec<");
                        sb.append(outkiFieldType(field));
//                        if (field.isParentField)
//                        	sb.append("<'parent>");
                        if (field.isArray) sb.append(">");
                    }
                    
                    if (struct.isTypeRoot)
                    {
                    	String spfx = pfx + "\t";
	                    for (Compiler.ParsedStruct subs : file.structs)
	                    {
	                    	if (subs.parent != null && subs.parent.equals(struct.name))
	                    	{
	                            if (!first)
	                            	sb.append(",");
	                    		sb.append(spfx).append("pub " + withUnderscore(subs.name) + " : rc::Weak<" + structName(subs) + ">");
	                    	}
	                    }
                    }                    
                    sb.append(pfx).append("}");
                }
            }

            sb.append("\n}");
            writer.addOutput(fn, sb.toString().getBytes());

            
            Path manifest = tree.genCodeRoot.resolve("rust");
            Path mfn = manifest.resolve("Cargo.toml");
            sb = new StringBuilder();
            sb.append("[package]\n");
            sb.append("name = \"putki_gen\"\n");
            sb.append("version = \"0.1.0\"\n");
            sb.append("[lib]\n");
            sb.append("name = \"putki_gen\"\n");
            writer.addOutput(mfn, sb.toString().getBytes());
        }
    }

}
