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
import sun.misc.FormattedFloatingDecimal;;

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
		String fn = withUnderscore(s.name);
		if (fn.equals("type")) return "type_";
		if (fn.equals("self")) return "self_";		
		if (fn.equals("bool")) return "bool_";
		if (fn.equals("mod")) return "mod_";
		return fn;
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
			case HASH:
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
			return structNameWrap(pf.resolvedRefStruct);
		}
		else if (pf.type == FieldType.ENUM)
		{
			return pf.resolvedEnum.name;
		}
		else if (pf.type == FieldType.POINTER)
		{
			if (pf.allowNull)
				return "outki::NullablePtr<" + structName(pf.resolvedRefStruct) + ">";
			else
				return "outki::Ptr<" + structName(pf.resolvedRefStruct) + ">";
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
			case HASH:
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
		else if (pf.type == FieldType.ENUM)
		{
			return pf.resolvedEnum.name;
		}		
		else if (pf.type == FieldType.POINTER)
		{
			return "putki::Ptr<" + structName(pf.resolvedRefStruct) + ">";
		}		
		else
		{
			return outkiFieldtypePod(pf.type);
		}
	}
	
	static String defaultEnumValue(Compiler.ParsedField pf, String prefix)
	{
		if (pf.defValue != null)
			return prefix + pf.resolvedEnum.name + "::" + pf.defValue;
		else
			return prefix + pf.resolvedEnum.name + "::" + capsToCamelCase(pf.resolvedEnum.values.get(0).name);
	}
	
	static String defaultValue(Compiler.ParsedField pf)
	{		
		if (pf.type == FieldType.FLOAT)
			return fmtFloat(pf.defValue);
		if (pf.type == FieldType.ENUM && pf.defValue != null)
			return  pf.resolvedEnum.name + "::" +capsToCamelCase(pf.defValue);
		if (pf.type == FieldType.POINTER)
			return "putki::Ptr::null()";
		if (pf.type == FieldType.STRING && pf.defValue != null)
			return pf.defValue + ".to_string()";
		else if (pf.type == FieldType.STRING || pf.type == FieldType.HASH)
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
	        sb.append("#![recursion_limit=\"128\"]");
	        sb.append("\nextern crate putki;"); 	        
	        sb.append("\npub mod inki;");
	        sb.append("\npub mod outki;");
	        writer.addOutput(fn, sb.toString().getBytes());	        
	        generateInkiStructs(comp, tree, writer);
	        generateInkiParsers(comp, tree, writer);
	        generateOutkiStructs(comp, tree, writer);
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
        sb.append("\nuse putki;");
        sb.append("\nuse putki::BinWriter;");

        sb.append("\n");
//        sb.append("\n\tpub use putki::mixki::rtti;");

        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
        	for (Compiler.ParsedEnum e : file.enums)
        	{
        		String prefix = "\n";
    			sb.append("\n");
    			sb.append("#[derive(Clone)]\n");
        		sb.append(prefix).append("pub enum " + e.name + " {");
        		boolean first = true;
        		for (Compiler.EnumValue val : e.values)
        		{
        			if (!first) sb.append(",");
        			sb.append(prefix).append("\t" + capsToCamelCase(val.name));
        			first = false;
        		}
    			sb.append(prefix).append("}");
    			sb.append("\n");
        		sb.append(prefix).append("impl Default for " + e.name + " { fn default() -> Self { " + e.name + "::" + capsToCamelCase(e.values.get(0).name) + " } }");
    			sb.append("\n");    			
        		sb.append(prefix).append("impl From<&" + e.name + "> for i32 {");
        		sb.append(prefix).append("\tfn from(val: &" + e.name + ") -> i32 {");        		
        		sb.append(prefix).append("\t\tmatch *val {");
        		for (Compiler.EnumValue val : e.values)
        		{        		
        			sb.append(prefix).append("\t\t\t" + e.name + "::" + capsToCamelCase(val.name) + " => " + val.value + ",");
        		}
        		sb.append(prefix).append("\t\t}");
        		sb.append(prefix).append("\t}");
        		sb.append(prefix).append("}");
    			sb.append("\n");
        		sb.append(prefix).append("impl From<i32> for " + e.name + " {");
        		sb.append(prefix).append("\tfn from(val:i32) -> Self {");        		
        		sb.append(prefix).append("\t\tmatch val {");
        		for (Compiler.EnumValue val : e.values)
        		{        		
        			sb.append(prefix).append("\t\t\t" + val.value + " => " + e.name + "::" + capsToCamelCase(val.name) + ",");
        		}
    			sb.append(prefix).append("\t\t\t_ => Default::default()");
        		sb.append(prefix).append("\t\t}");
        		sb.append(prefix).append("\t}");
        		sb.append(prefix).append("}");       
    			sb.append("\n");
        		sb.append(prefix).append("impl<'a> From<&'a str> for " + e.name + " {");
        		sb.append(prefix).append("\tfn from(val:&str) -> Self {");        		
        		sb.append(prefix).append("\t\tmatch val {");
        		for (Compiler.EnumValue val : e.values)
        		{        		
        			sb.append(prefix).append("\t\t\t\"" + val.name + "\" => " + e.name + "::" + capsToCamelCase(val.name) + ",");
        		}
    			sb.append(prefix).append("\t\t\t_ => Default::default()");
        		sb.append(prefix).append("\t\t}");
        		sb.append(prefix).append("\t}");
        		sb.append(prefix).append("}");
    			sb.append("\n");
        		sb.append(prefix).append("impl From<" + e.name + "> for &'static str {");
        		sb.append(prefix).append("\tfn from(val:" + e.name + ") -> Self {");        		
        		sb.append(prefix).append("\t\tmatch val {");
        		for (Compiler.EnumValue val : e.values)
        		{        		
        			sb.append(prefix).append("\t\t\t" + e.name + "::" + capsToCamelCase(val.name) + " => \"" + val.name + "\",");
        		}
        		sb.append(prefix).append("\t\t}");
        		sb.append(prefix).append("\t}");
        		sb.append(prefix).append("}");              		
        	}

    		for (Compiler.ParsedStruct struct : file.structs)
    		{
            	String pfx = "\n";
                if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                    continue;                    
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0)
                {
                	sb.append("\n");       
                	sb.append(pfx).append("#[derive(Clone)]");
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
	            	sb.append(pfx).append("#[derive(Clone)]");
	            	sb.append(pfx).append("pub struct " + structNameWrap(struct) + " {");
	                
	                boolean first = true;                
	                String spfx = pfx + "\t";           
	                for (Compiler.ParsedField field : struct.fields)
	                {	                	
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    
	                    if (!first)
	                    	sb.append(",");
	                    first = false;
	                    
	                    
	                	sb.append(spfx).append("pub " + fieldName(field) + " : ");
	                    if (field.isArray) sb.append("vec::Vec<");
	                	sb.append(inkiFieldType(field));
	                    if (field.isArray) sb.append(">");
	                }
	            
	                sb.append(pfx).append("}");
                }
                
                if (structNameWrap(struct).length() > 0) 
                {
                	sb.append("\n");
	                sb.append(pfx).append("impl putki::WriteAsText for " + structNameWrap(struct) + " {");
	                sb.append(pfx).append("\tfn write_text(&self, _output: &mut String) -> Result<(), putki::PutkiError> { Ok(()) }");
	                sb.append(pfx).append("}");
	                sb.append("\n");                
	                sb.append(pfx).append("impl putki::BinSaver for " + structNameWrap(struct) + " {");
	                sb.append(pfx).append("\tfn write(&self, _data: &mut Vec<u8>, _refwriter: &putki::PackageRefs) -> Result<(), putki::PutkiError> {");
	                String spfx = pfx + "\t\t";           
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    if (field.type == FieldType.ENUM) {
	                    	sb.append(spfx).append("i32::from(&self." + fieldName(field) + ").write(_data);");
	                    } else if (field.type == FieldType.STRUCT_INSTANCE || field.type == FieldType.POINTER) {
	                    	sb.append(spfx).append("self." + fieldName(field) + ".write(_data, _refwriter)?;");	                    
	                    } else {
	                    	sb.append(spfx).append("self." + fieldName(field) + ".write(_data);");
	                    }
	                }	                
	                sb.append(spfx).append("Ok(())");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");	               
                }
	                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                	sb.append("\n");
	                sb.append(pfx).append("impl putki::WriteAsText for " + structName(struct) + " {");
	                sb.append(pfx).append("\tfn write_text(&self, _output: &mut String) -> Result<(), putki::PutkiError> { Ok(()) }");
	                sb.append(pfx).append("}");
	                sb.append("\n");
	                sb.append(pfx).append("impl putki::BinSaver for " + structName(struct) + " {");
	                sb.append(pfx).append("\tfn write(&self, data: &mut Vec<u8>, refwriter: &putki::PackageRefs) -> Result<(), putki::PutkiError> {");
	                String spfx = pfx + "\t\t";
	                sb.append(spfx).append("match self {");
	                for (int i=0;i<=struct.possibleChildren.size();i++)
	                {
	                	if (i > 0)
	                		sb.append(",");
	                	Compiler.ParsedStruct s = (i == 0) ? struct : struct.possibleChildren.get(i-1);
	                	if (structNameWrap(s).length() > 0)
	                		sb.append(spfx).append("\t" + structName(struct) + "::" + structName(s) + "(x) => { (" + i + " as u16).write(data); x.write(data, refwriter) }");
	                	else
	                		sb.append(spfx).append("\t" + structName(struct) + "::" + structName(s) + " => { (" + i + " as u16).write(data); Ok(()) }");
	                }
	                sb.append(spfx).append("}");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");
	                /*
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    if (field.type == FieldType.ENUM) {
	                    	sb.append(spfx).append("i32::from(&self." + fieldName(field) + ").write(_data);");
	                    } else if (field.type == FieldType.STRUCT_INSTANCE || field.type == FieldType.POINTER) {
	                    	sb.append(spfx).append("self." + fieldName(field) + ".write(_data, _refwriter)?;");	                    
	                    } else {
	                    	sb.append(spfx).append("self." + fieldName(field) + ".write(_data);");
	                    }
	                }	        
	                */
                }
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl putki::TypeDescriptor for " + structName(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; }");
	                sb.append(pfx).append("impl putki::InkiObj for " + structName(struct) + " { }");               	
                }                

                if (structNameWrap(struct).length() > 0)
                {
                    sb.append("\n");                                            	
                	sb.append(pfx).append("impl putki::TypeDescriptor for " + structNameWrap(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; }");
                	sb.append(pfx).append("impl putki::InkiObj for " + structNameWrap(struct) + " { }");
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
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    if (!first)
	                    	sb.append(",");
	                    first = false;                        
	                    if (field.isArray)
		                	sb.append(spfx).append(fieldName(field) + " : Vec::new()");
	                    else
	                    	sb.append(spfx).append(fieldName(field) + " : " + defaultValue(field));
	                }
	
	                sb.append(pfx).append("\t\t}");
	                sb.append(pfx).append("\t}");                    
	                sb.append(pfx).append("}");
                }
                
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0)
                {
                	sb.append("\n");
	            	sb.append(pfx).append("impl putki::BuildCandidate for " + structName(struct) + " {");
	                sb.append(pfx).append("\tfn as_any_ref(&mut self) -> &mut any::Any { return self; }");
	                sb.append(pfx).append("\tfn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) -> Result<(), putki::PutkiError> { p.build(br, self) }");
	                sb.append(pfx).append("\tfn scan_deps(&self, _p:&putki::Pipeline, _br: &mut putki::BuildRecord) {");
	                sb.append(pfx).append("\t\tmatch self {");
                	if (structNameWrap(struct).length() == 0)
                    	sb.append(pfx).append("\t\t\t&" + structName(struct) + "::" + structName(struct) + " => { }");
                	else
                		sb.append(pfx).append("\t\t\t&" + structName(struct) + "::" + structName(struct) + "(ref c) => { c.scan_deps(_p, _br); }");                			                
	                for (Compiler.ParsedStruct s : struct.possibleChildren) {
	                	sb.append(pfx).append("\t\t\t&" + structName(struct) + "::" + structName(s) + "(ref c) => { c.scan_deps(_p, _br); },");	
	                }	                
	                sb.append(pfx).append("\t\t}");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");
                }
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0)
                {       
                	sb.append("\n");
	            	sb.append(pfx).append("impl putki::BuildFields for " + structName(struct) + " {");
	            	sb.append(pfx).append("\tfn build_fields(&mut self, _p:&putki::Pipeline, _br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {");
	                sb.append(pfx).append("\t\tmatch self {");
                	if (structNameWrap(struct).length() == 0)
                    	sb.append(pfx).append("\t\t\t" + structName(struct) + "::" + structName(struct) + " => Ok(()),");
                	else
                		sb.append(pfx).append("\t\t\t" + structName(struct) + "::" + structName(struct) + "(ref mut c) => c.build_fields(_p, _br),");                			                
	                for (Compiler.ParsedStruct s : struct.possibleChildren) {
	                	sb.append(pfx).append("\t\t\t" + structName(struct) + "::" + structName(s) + "(ref mut c) => c.build_fields(_p, _br),");	
	                }	                
	                sb.append(pfx).append("\t\t}");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");
                }                
                
                if (structNameWrap(struct).length() > 0)
                {
                    sb.append("\n");                                  
	            	sb.append(pfx).append("impl putki::BuildCandidate for " + structNameWrap(struct) + " {");
	                sb.append(pfx).append("\tfn as_any_ref(&mut self) -> &mut any::Any { return self; }");
	                sb.append(pfx).append("\tfn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) -> Result<(), putki::PutkiError> { p.build(br, self) }");
	                sb.append(pfx).append("\tfn scan_deps(&self, _p:&putki::Pipeline, _br: &mut putki::BuildRecord) {");
	            	                
	                boolean	any = false;
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.type == FieldType.POINTER)
                    	{                    		
                    		if (field.isArray)
                    			sb.append(pfx).append("\t\tfor ptr in &self." + fieldName(field) + " { _p.add_output_dependency(_br, ptr); }");
                    		else
                    			sb.append(pfx).append("\t\t_p.add_output_dependency(_br, &self." + fieldName(field) + ");");
                    		any = true;
                    	} 
                     	if (field.type == FieldType.STRUCT_INSTANCE)
                    	{
    	                    if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
    	                    	continue;
                    		if (field.isArray)
                    			sb.append(pfx).append("\t\tfor obj in &self." + fieldName(field) + " { obj.scan_deps(_p, _br); }");
                    		else
                    			sb.append(pfx).append("\t\tself." + fieldName(field) + ".scan_deps(_p, _br);");
                    		any = true;
                    	}                        	
	                }
	                if (!any) sb.append("}"); else sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");
	                
	                sb.append("\n");
	            	sb.append(pfx).append("impl putki::BuildFields for " + structNameWrap(struct) + " {");
	                sb.append(pfx).append("\tfn build_fields(&mut self, _pipeline:&putki::Pipeline, _br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {");
	                
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
	                    if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
	                    	continue;
                    	if (field.type == FieldType.STRUCT_INSTANCE)
                    	{                    		
                    		if (field.isArray)
                    			sb.append(pfx).append("\t\tfor cont in &mut self." + fieldName(field) + " { _pipeline.build(_br, cont)?; }");
                    		else
                    			sb.append(pfx).append("\t\t_pipeline.build(_br, &mut self." + fieldName(field) + ")?;");
                    	}                    			                    
	                }
	                sb.append(pfx).append("\t\tOk(())");
	                sb.append(pfx).append("\t}");	 
	                sb.append(pfx).append("}");
	                sb.append(pfx);
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
        sb.append("[dependencies]\r\nputki = { path = \""  + tree.putkiPath.resolve("rust").toAbsolutePath().toString().replaceAll("\\\\",  "/") + "\" }");
        writer.addOutput(mfn, sb.toString().getBytes());
    }
    
    public static void generateOutkiStructs(Compiler comp, Compiler.ParsedTree tree, CodeWriter writer)
    {
        Path lib = tree.genCodeRoot.resolve("rust").resolve("src").resolve("outki");
        Path fn = lib.resolve("mod.rs");
        StringBuilder sb = new StringBuilder();
        sb.append("#![allow(unused_imports)]");        
        sb.append("\nuse std::any;");
        sb.append("\nuse std::default;");            
        sb.append("\nuse std::vec;");        
        sb.append("\nuse putki::outki as outki;");

        sb.append("\n");

        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
        	for (Compiler.ParsedEnum e : file.enums)
        	{
        		String prefix = "\n";
    			sb.append("\n");
        		sb.append(prefix).append("pub enum " + e.name + " {");
        		boolean first = true;
        		for (Compiler.EnumValue val : e.values)
        		{
        			if (!first) sb.append(",");
        			sb.append(prefix).append("\t" + capsToCamelCase(val.name));
        			first = false;
        		}
    			sb.append(prefix).append("}");
    			sb.append("\n");
        		sb.append(prefix).append("impl From<&" + e.name + "> for i32 {");
        		sb.append(prefix).append("\tfn from(val: &" + e.name + ") -> i32 {");        		
        		sb.append(prefix).append("\t\tmatch val {");
        		for (Compiler.EnumValue val : e.values)
        		{        		
        			sb.append(prefix).append("\t\t\t" + e.name + "::" + capsToCamelCase(val.name) + " => " + val.value + ",");
        		}
        		sb.append(prefix).append("\t\t}");
        		sb.append(prefix).append("\t}");
        		sb.append(prefix).append("}");
    			sb.append("\n");
        		sb.append(prefix).append("impl From<i32> for " + e.name + " {");
        		sb.append(prefix).append("\tfn from(val:i32) -> Self {");        		
        		sb.append(prefix).append("\t\tmatch val {");
        		for (Compiler.EnumValue val : e.values)
        		{        		
        			sb.append(prefix).append("\t\t\t" + val.value + " => " + e.name + "::" + capsToCamelCase(val.name) + ",");
        		}
    			sb.append(prefix).append("\t\t\t_ => " + e.name + "::" + capsToCamelCase(e.values.get(0).name));
        		sb.append(prefix).append("\t\t}");
        		sb.append(prefix).append("\t}");
        		sb.append(prefix).append("}");       
    			sb.append("\n");          		
        	}

    		for (Compiler.ParsedStruct struct : file.structs)
    		{
            	String pfx = "\n";
                if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                    continue;                    
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0)
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
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    
	                    if (!first)
	                    	sb.append(",");
	                    first = false;
	                    
	                    
	                	sb.append(spfx).append("pub " + fieldName(field) + " : ");
	                    if (field.isArray) sb.append("vec::Vec<");
	                	sb.append(outkiFieldType(field));
	                    if (field.isArray) sb.append(">");
	                }
	            
	                sb.append(pfx).append("}");
                }
                
                if (structNameWrap(struct).length() > 0) 
                {                	               
	                sb.append(pfx);	                
	                sb.append(pfx).append("impl outki::BinLoader for " + structNameWrap(struct) + " {");
	                sb.append(pfx).append("\tfn read(_stream:&mut outki::BinDataStream) -> Self {");
	                sb.append(pfx).append("\t\tSelf {");
	                boolean	first = true;	                
	                String spfx = pfx + "\t\t\t";           
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
	                    if (!first)
	                    	sb.append(",");
	                    first = false;
	                    if (field.type == FieldType.ENUM) {
	                    	sb.append(spfx).append(fieldName(field) + " : " + outkiFieldType(field) + "::from(<i32 as outki::BinReader>::read(_stream))");
	                    } else if (field.type == FieldType.STRUCT_INSTANCE || field.type == FieldType.POINTER) {
	                    	sb.append(spfx).append(fieldName(field) + " : outki::BinLoader::read(_stream)");	                    
	                    } else {
	                    	sb.append(spfx).append(fieldName(field) + " : outki::BinReader::read(_stream)");
	                    }
	                }	                
	                sb.append(pfx).append("\t\t}");
	                sb.append(pfx).append("\t}");
	                	                	                
	                sb.append(pfx).append("\tfn resolve(&mut self, _context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> {");
	                spfx = pfx + "\t\t";
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;	                    
	                    if (field.type == FieldType.STRUCT_INSTANCE || field.type == FieldType.POINTER) {
	                    	sb.append(spfx).append("self." + fieldName(field) + ".resolve(_context)?;");
	                    }
	                }	                
	                sb.append(spfx).append("Ok(())");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");
	            	                          	               
                }
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0)
                {
                    sb.append(pfx);
	                sb.append(pfx).append("impl outki::BinLoader for " + structName(struct) + " {");
	                sb.append(pfx).append("\tfn read(_stream:&mut outki::BinDataStream) -> Self {");
	                String spfx = pfx + "\t\t";
	                sb.append(spfx).append("match <u16 as outki::BinReader>::read(_stream) {");
	                for (int i=struct.possibleChildren.size();i>=0;i--)
	                {		        
	                	Compiler.ParsedStruct s = (i == 0) ? struct : struct.possibleChildren.get(i-1);
	                	if (i == 0)
	                		sb.append(spfx).append("\t_");
	                	else
	                		sb.append(spfx).append("\t" + i);
	                	if (structNameWrap(s).length() > 0)	                		
	                		sb.append(" => " + structName(struct) + "::" + structName(s) + "(outki::BinLoader::read(_stream))");
	                	else
	                		sb.append(" => " + structName(struct) + "::" + structName(s));
	                	if (i != 0)
	                		sb.append(",");
	                }
	                sb.append(spfx).append("}");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("\tfn resolve(&mut self, _context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> {");
	                sb.append(spfx).append("match self {");
	                for (int i=0;i<=struct.possibleChildren.size();i++)
	                {		        
	                	if (i > 0)
	                		sb.append(",");
	                	Compiler.ParsedStruct s = (i == 0) ? struct : struct.possibleChildren.get(i-1);
	                	if (structNameWrap(s).length() > 0)	                		
	                		sb.append(spfx).append("\t" + structName(struct) + "::" + structName(s) + "(x) => x.resolve(_context)");
	                	else
	                		sb.append(spfx).append("\t" + structName(struct) + "::" + structName(s) + " => Ok(())");
	                }	                
	                sb.append(spfx).append("}");
	                sb.append(pfx).append("\t}");
	                sb.append(pfx).append("}");
	                sb.append(pfx);
                }	                 
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl putki::TypeDescriptor for " + structName(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; }");
                	sb.append(pfx).append("impl outki::OutkiObj for " + structName(struct) + " { }");
                }                

                if (structNameWrap(struct).length() > 0)
                {
                    sb.append("\n");                    
                	sb.append(pfx).append("impl putki::TypeDescriptor for " + structNameWrap(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; }");
                	sb.append(pfx).append("impl outki::OutkiObj for " + structNameWrap(struct) + " { }");
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
        sb.append("[dependencies]\r\nputki = { path = \""  + tree.putkiPath.resolve("rust").toAbsolutePath().toString().replaceAll("\\\\",  "/") + "\" }");
        writer.addOutput(mfn, sb.toString().getBytes());
    }

    static void allChildrenTags(StringBuilder sb, Compiler.ParsedStruct struct)
    {
    	for (Compiler.ParsedStruct s : struct.possibleChildren) {
    		sb.append(" | <inki::" + structName(s) + " as putki::TypeDescriptor>::TAG");
    		allChildrenTags(sb, s);
    	}    	
    }
    
    public static void generateInkiParsers(Compiler comp, Compiler.ParsedTree tree, CodeWriter writer)
    {
        Path lib = tree.genCodeRoot.resolve("rust").resolve("src").resolve("inki");
        Path fn = lib.resolve("parse.rs");
        StringBuilder sb = new StringBuilder();        
        sb.append("#![allow(unused_imports)]\nuse std::rc;\n" +
        	"use inki;\n" + 
    		"use putki;\n" +
    		"use std::sync::Arc;\n" +        	
        	"use std::ops::Deref;\n" +
    		"use std::default;\n"
    	);

        for (Compiler.ParsedFile file : tree.parsedFiles)
        {
    		for (Compiler.ParsedStruct struct : file.structs)
    		{
            	String pfx = "\n";           
            	
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl putki::ParseFromKV for inki::" + structName(struct) + " {");
                	sb.append(pfx).append("\tfn parse_with_type(_kv : &putki::LexedKv, _resolver: &Arc<putki::InkiResolver>, type_name:&str) -> Self {");
                	sb.append(pfx).append("\t\tmatch type_name {");
                	
                	if (structNameWrap(struct).length() > 0)
                		sb.append(pfx).append("\t\t\t<inki::" + structName(struct) + " as putki::TypeDescriptor>::TAG => inki::" + structName(struct) + "::" + structName(struct) + "(<inki::" + structNameWrap(struct) + " as putki::ParseFromKV>::parse(_kv, _resolver)),");
                	else
                		sb.append(pfx).append("\t\t\t<inki::" + structName(struct) + " as putki::TypeDescriptor>::TAG => inki::" + structName(struct) + "::" + structName(struct) + ",");
                	
                	for (Compiler.ParsedStruct child : struct.possibleChildren)
                	{                		
                    	sb.append(pfx).append("\t\t\t<inki::" + structName(child) + " as putki::TypeDescriptor>::TAG");
                    	allChildrenTags(sb, child);
                    	sb.append(" => inki::" + structName(struct) + "::" + structName(child) + "(<inki::" + structName(child) + " as putki::ParseFromKV>::parse_with_type(_kv, _resolver, type_name)),");                		
                	}                	
                	sb.append(pfx).append("\t\t\t_ => Default::default()");
                	sb.append(pfx).append("\t\t}");
                	sb.append(pfx).append("\t}");
                	sb.append(pfx).append("\tfn parse(_kv : &putki::LexedKv, _resolver: &Arc<putki::InkiResolver>) -> Self {");
                	if (structNameWrap(struct).length() > 0)
                		sb.append(pfx).append("\t\tinki::" + structName(struct) + "::" + structName(struct) + "(<inki::" + structNameWrap(struct) + " as putki::ParseFromKV>::parse(_kv, _resolver))");
                	else
                		sb.append(pfx).append("\t\tinki::" + structName(struct) + "::" + structName(struct));
                	sb.append(pfx).append("\t}");                	
                	sb.append(pfx).append("}");                	
                }
                
                if (structNameWrap(struct).length() > 0)
                {          
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl putki::ParseFromKV for inki::" + structNameWrap(struct) + " {");
                	sb.append(pfx).append("\tfn parse(_src : &putki::LexedKv, _resolver: &Arc<putki::InkiResolver>) -> Self {");
                	sb.append(pfx).append("\t\tSelf {");
                    String spfx = pfx + "\t\t\t";
                    boolean first = true;
                    for (Compiler.ParsedField field : struct.fields)
                    {                    	
                        if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
                            continue;
                    	if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    		continue;
                        if (!first)
                        	sb.append(",");                                                
                        first = false;                                               
                    	sb.append(spfx).append(fieldName(field) + " : {");
                    	
                    	if (field.isArray)
                    	{
                    		sb.append("putki::get_array(_src.get(\"" + field.name + "\")).and_then(|iter| { Some(iter.map(|da| { let data = Some(da); ");
                    	}
                    	else
                    	{
                    		sb.append("let data = _src.get(\"" + field.name + "\"); ");
                    	}
                    	
                      	switch (field.type)
                        {
                        	case FLOAT:                      	
                        	case INT32:
                        	case BYTE: 
                        		sb.append("putki::get_value(data, " + defaultValue(field) + ")");
                        		break;
                        	case BOOL: 
                        		sb.append("putki::get_bool(data, " + defaultValue(field) + ")");
                        		break;
                        	case HASH:
                        	case STRING:
                        		sb.append("putki::get_string(data, " + (field.defValue != null ? field.defValue : "\"\"") + ")");
                        		break;
                        	case STRUCT_INSTANCE:
                        		if (!field.isParentField && (field.resolvedRefStruct.isTypeRoot || field.resolvedRefStruct.possibleChildren.size() > 0))
                            		sb.append("<inki::" + structName(field.resolvedRefStruct) + " as putki::ParseFromKV>::parse(_resolver, data)");
                        		else if (field.isParentField)
                        			sb.append("<inki::" + structNameWrap(field.resolvedRefStruct) + " as putki::ParseFromKV>::parse(putki::get_kv(data).unwrap_or(_src), _resolver)");
                       			else
                        			sb.append("putki::get_object(data).and_then(|v| { Some(<inki::" + structNameWrap(field.resolvedRefStruct) + " as putki::ParseFromKV>::parse(v.0, _resolver)) }).unwrap_or_default()");
                        		break;
                        	case POINTER:
                    			sb.append("data.and_then(|v| { Some(putki::ptr_from_data(_resolver, v)) }).unwrap_or_default()");
                        		break;
                        	case ENUM:
                        		sb.append("inki::" + field.resolvedEnum.name + "::from(putki::get_string(data, \"");
                        		if (field.defValue != null) 
                        			sb.append(field.defValue);
                        		sb.append("\").as_ref())");
                        		break;                        		
                        	default:
                        		sb.append("Default::default()");                            	                        	
                        }                    	
           
                		sb.append("}");                    	                		                    	
                    	if (field.isArray)
                    	{
                    		sb.append(").collect())}).unwrap_or_default()}");
                    	}                		
                    }    
                	sb.append(pfx).append("\t\t}");                	                  
                	sb.append(pfx).append("\t}");
                	sb.append(pfx).append("}");                	
                }                
    		}
        }
        writer.addOutput(fn, sb.toString().getBytes());        
    }
}
    
