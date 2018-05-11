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
	
	public static String structNameWrap(Compiler.ParsedStruct s)
	{
		if (s.isTypeRoot && s.possibleChildren.size() == 0 || s.fields.size() == 0)
			return "";
		if (s.possibleChildren.size() > 0 || s.isTypeRoot)
			return s.name + "Data";
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
			if (pf.allowNull)
				return "Option<rc::Rc<" + structName(pf.resolvedRefStruct) + ">>";
			else
				return "rc::Rc<" + structName(pf.resolvedRefStruct) + ">";
		}		
		else
		{
			return outkiFieldtypePod(pf.type);
		}
	}	
	
	static String inkiFieldtypePod(Compiler.FieldType f)
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
	
	static String inkiFieldType(Compiler.ParsedField pf)
	{		
		if (pf.type == FieldType.STRUCT_INSTANCE)
		{
			return structNameWrap(pf.resolvedRefStruct);
		}
		else if (pf.type == FieldType.POINTER)
		{
			return "putki_inki::Ptr<" + structName(pf.resolvedRefStruct) + ">";
		}		
		else
		{
			return outkiFieldtypePod(pf.type);
		}
	}
	
	static String defaultValue(Compiler.ParsedField pf)
	{		
		if (pf.type == FieldType.POINTER)
			return "putki_inki::Ptr::null()";
		if (pf.type == FieldType.STRING && pf.defValue != null)
			return pf.defValue + ".to_string()";
		else if (pf.type == FieldType.STRING)
			return "String::new()";
		else if (pf.defValue != null)
			return pf.defValue;
		else
			return "Default::default()";
	}		
	
	public static String moduleName(String in)
	{
		return "gen_" + withUnderscore(in);
	}
	
	/*
	public static String enumValue(Compiler.EnumValue s)
	{
		return enumValue(s.name);
	}*/	
	
    public static void generateCrate(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {    	
	        Path lib = tree.genCodeRoot.resolve("rust").resolve("src");
	        Path fn = lib.resolve("lib.rs");
	        StringBuilder sb = new StringBuilder();
	        sb.append("\nextern crate putki;"); 	        
	        sb.append("\npub mod inki;");
	        writer.addOutput(fn, sb.toString().getBytes());	        
	        generateInkiStructs(comp, tree, writer);
	        generateInkiParsers(comp, tree, writer);
        }
    }

    public static void generateInkiStructs(Compiler comp, Compiler.ParsedTree tree, CodeWriter writer)
    {
        Path lib = tree.genCodeRoot.resolve("rust").resolve("src").resolve("inki");
        Path fn = lib.resolve("mod.rs");
        StringBuilder sb = new StringBuilder();
        sb.append("#![allow(unused_imports)]");
        sb.append("\nuse std::rc;");
        sb.append("\nuse std::any;");
        sb.append("\nuse std::default;");            
        sb.append("\nuse std::vec;");
        sb.append("\nmod parse;");
        sb.append("\nuse putki::inki::source;");
        sb.append("\nuse putki::inki as putki_inki;");

        sb.append("\n");
//        sb.append("\n\tpub use putki::mixki::rtti;");

        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
    		for (Compiler.ParsedStruct struct : file.structs)
    		{
            	String pfx = "\n";
                if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                    continue;                    
                
                if (struct.isTypeRoot)
                {
                	sb.append("\n");                	
                	sb.append(pfx).append("pub enum " + structName(struct) + " {");
                	if (structNameWrap(struct).length() == 0)
                    	sb.append(pfx).append("\t" + structName(struct));
                	else
                		sb.append(pfx).append("\t" + structName(struct) + "(" + structNameWrap(struct) + ")");
                	for (Compiler.ParsedStruct ch : struct.possibleChildren)
                	{
                		sb.append(",").append(pfx).append("\t" + structName(ch) + "(" + structName(ch) + ")");
                	}
                	sb.append(pfx).append("}");
                }
  
                // Pure enum roots have type ()
                if (structNameWrap(struct).length() > 0) 
                {                
	            	sb.append("\n");                
	            	sb.append(pfx).append("pub struct " + structNameWrap(struct) + " {");
	                
	                boolean first = true;                
	                String spfx = pfx + "\t";           
	                for (Compiler.ParsedField field : struct.fields)
	                {	                	
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
	                    if (!first)
	                    	sb.append(",");
	                    first = false;
	                    
                    	if (field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    
	                	sb.append(spfx).append("pub " + fieldName(field) + " : ");
	                    if (field.isArray) sb.append("vec::Vec<");
	                	sb.append(inkiFieldType(field));
	                    if (field.isArray) sb.append(">");
	                }
	            
	                sb.append(pfx).append("}");
                }
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl source::InkiTypeDescriptor for " + structName(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; type OutkiType = (); }");                    
                }                

                if (structNameWrap(struct).length() > 0)
                {
                    sb.append("\n");                                            	
                	sb.append(pfx).append("impl source::InkiTypeDescriptor for " + structNameWrap(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; type OutkiType = (); }");
                }

                               
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0)
                {          
	                sb.append("\n");                	
	            	sb.append(pfx).append("impl default::Default for " + structName(struct) + " {");
	            	if (structNameWrap(struct).length() == 0)
	            		sb.append(pfx).append("\tfn default() -> Self { " + structName(struct) + "::" + structName(struct) + " }");
	            	else
	            		sb.append(pfx).append("\tfn default() -> Self { " + structName(struct) + "::" + structName(struct) + "(Default::default()) }");
	                sb.append(pfx).append("}");                    
                }
                
                if (structNameWrap(struct).length() > 0)
                {
                    sb.append("\n");                                  
	            	sb.append(pfx).append("impl default::Default for " + structNameWrap(struct) + " {");
	                sb.append(pfx).append("\tfn default() -> Self {");    
	                sb.append(pfx).append("\t\treturn " + structNameWrap(struct) + " {");    
	                boolean	first = true;	                
	                String spfx = pfx + "\t\t\t";           
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    
	                    if (!first)
	                    	sb.append(",");
	                    first = false;                        
	                	sb.append(spfx).append(fieldName(field) + " : " + defaultValue(field));
	                }
	
	                sb.append(pfx).append("\t\t}");
	                sb.append(pfx).append("\t}");                    
	                sb.append(pfx).append("}");
                }
    		}
        }           
        
        writer.addOutput(fn, sb.toString().getBytes());
        
        Path manifest = tree.genCodeRoot.resolve("rust");
        Path mfn = manifest.resolve("Cargo.toml");
        sb = new StringBuilder();
        sb.append("[package]\n");
        sb.append("name = \"putki_gen\"\n");
        sb.append("version = \"0.1.0\"\n");
        sb.append("[lib]\n");
        sb.append("name = \"" + moduleName(tree.moduleName) + "\"\n");
        sb.append("[dependencies]\r\nputki = { path = \"../../../../rust\" }");
        writer.addOutput(mfn, sb.toString().getBytes());
    }
    
    public static void generateInkiParsers(Compiler comp, Compiler.ParsedTree tree, CodeWriter writer)
    {
        Path lib = tree.genCodeRoot.resolve("rust").resolve("src").resolve("inki");
        Path fn = lib.resolve("parse.rs");
        StringBuilder sb = new StringBuilder();        
        sb.append("#![allow(unused_imports)]\nuse std::rc;\n" +
        	"use inki;\n" + 
        	"use putki::inki as putki_inki;\n" +
    		"use putki::inki::source;\n" + 
    		"use putki::inki::lexer;\n" +
        	"use std::ops::Deref;\n" +
    		"use std::default;\n"
    	);
        
/*
        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
        	String pfx = "\n\t\t\t";
    		for (Compiler.ParsedStruct struct : file.structs)
    		{                	                
    	        sb.append("\n");                  			
                sb.append(pfx).append("\"" + struct.name + "\" => ");
                if (struct.possibleChildren.size() > 0 || struct.isTypeRoot)
                	sb.append("return Some(rc::Rc::<mixki::" + struct.name + ">::new(parser::ParseToPure::parse(ctx, obj))), ");
                else if (struct.resolvedParent == null)
                	sb.append("return Some(rc::Rc::<mixki::" + struct.name + ">::new(parser::ParseSpecific::parse(ctx, obj))), ");
                else
                	sb.append("return Some(rc::Rc::<mixki::" + struct.resolvedParent.name + ">::new(parser::ParseToRoot::<'a, 'b, ParseRc, mixki::" + struct.name + ">::parse(ctx, obj))), ");
    		}
        }
        
        sb.append("\n\t\t\t_ => return None");
        sb.append("\n\t\t}");
        sb.append("\n\t}");
        sb.append("\n}");            		
        sb.append("\n");
        */
        
        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
    		for (Compiler.ParsedStruct struct : file.structs)
    		{
            	String pfx = "\n";           
            	
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl source::ParseFromKV for inki::" + structName(struct) + " {");
                	sb.append(pfx).append("\tfn parse(_kv : &lexer::LexedKv, _pctx: &putki_inki::InkiPtrContext) -> Self {");
                	sb.append(pfx).append("\t\tDefault::default()");                	
                	sb.append(pfx).append("\t}");
                	sb.append(pfx).append("}");                	
                }
                
                if (structNameWrap(struct).length() > 0)
                {          
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl source::ParseFromKV for inki::" + structNameWrap(struct) + " {");
                	sb.append(pfx).append("\tfn parse(_kv : &lexer::LexedKv, _pctx: &putki_inki::InkiPtrContext) -> Self {");
                	sb.append(pfx).append("\t\tDefault::default()");                	
                	sb.append(pfx).append("\t}");
                	sb.append(pfx).append("}");                	
                }                
    		}
        }
        /*
                sb.append(pfx).append("\t{");
                sb.append(pfx).append("\t\treturn mixki::" + structNameWrap(struct) + " {");
                String spfx = pfx + "\t\t\t";
                boolean first = true;
                for (Compiler.ParsedField field : struct.fields)
                {                    	
                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
                        continue;
                    if (field.isParentField)
                    	continue;
                    
                        
                    if (!first)
                    	sb.append(",");
                    first = false;
                	sb.append(spfx).append(fieldName(field) + " : ");
                    switch (field.type)
                    {
                    	case INT32:
                    	case FLOAT:
                    	case BYTE: 
                    		sb.append("lexer::get_value(_obj, \"" + field.name + "\", " + defaultValue(field) + ")");
                    		break;
                    	case BOOL: 
                    		sb.append("lexer::get_bool(_obj, \"" + field.name + "\", " + defaultValue(field) + ")");
                    		break;
                    	case STRING:
                    		sb.append("lexer::get_string(_obj, \"" + field.name + "\", " + (field.defValue != null ? field.defValue : "\"\"") + ")");
                    		break;
                    	case STRUCT_INSTANCE:
                    		sb.append("mixki::" + structName(field.resolvedRefStruct) + "::parse_or_default(_ctx, _obj.get(\"" + field.name + "\"))");
                    		break;
                    	case POINTER:
                    		if (field.allowNull)
                    			sb.append("_obj.get(\"" + field.name + "\").and_then(|v| { return parser::resolve_from_value(_ctx, v); })");
                    		else
                    			sb.append("_obj.get(\"" + field.name + "\").and_then(|v| { return parser::resolve_from_value(_ctx, v); }).unwrap_or_default()");
                    		break;
                    	default:
                    		sb.append("Default::default()");                            	                        	
                    }
                }         
                sb.append(pfx).append("\t\t}");
                sb.append(pfx).append("\t}");                                        
                sb.append(pfx).append("}");                    
                
    		}
        }
/*            
                    boolean first = true;
                    

                    for (Compiler.ParsedField field : struct.fields)
                    {
                    	
                        if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
                            continue;
                        if (!first)
                        	sb.append(",");
                        first = false;
                        
                    	sb.append(spfx).append("pub " + fieldName(field) + " : ");
                        if (field.isArray) sb.append("vec::Vec<");
                    	sb.append(outkiFieldType(field));
                        if (field.isArray) sb.append(">");
                    }
                    
                    if (struct.isTypeRoot)
                    {
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
        		
        		for (Compiler.ParsedStruct struct : file.structs)
        		{
                	String pfx = "\n\t";                
                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                        continue;
                	sb.append(pfx).append("impl default::Default for " + structName(struct));
                    sb.append(pfx).append("{");
                    sb.append(pfx).append("\tfn default() -> Self {");    
                    sb.append(pfx).append("\t\treturn " + structName(struct) + " {");    
                    boolean first = true;
                    
                    String spfx = pfx + "\t\t\t";           
                    for (Compiler.ParsedField field : struct.fields)
                    {
                    	
                        if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
                            continue;
                        if (!first)
                        	sb.append(",");
                        first = false;                        
                    	sb.append(spfx).append(fieldName(field) + " : " + defaultValue(field));
                    }

                    sb.append(pfx).append("\t\t}");
                    sb.append(pfx).append("\t}");                    
                    sb.append(pfx).append("}");                    
        		}        		
            }           
        }
    }
    */
        writer.addOutput(fn, sb.toString().getBytes());        
    }
}
    
