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
        		sb.append("\n\t\"" + enumName(e) + "\": {");
           		String prefix = "\n\t\t\t";
    			sb.append("\n\t\tValues: [");
        		boolean first = true;
        		for (Compiler.EnumValue val : e.values)
        		{
        			if (!first) sb.append(",");
        			sb.append(prefix).append("{ Name:\"" + val.name + "\", Value:" + val.value + " }");
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
	
        		sb.append("\n\t\"" + struct.name + "\": {");
           		String prefix = "\n\t\t\t";
/*    			
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
                
                if (struct.isTypeRoot || struct.possibleChildren.size() > 0) 
                {
                    sb.append("\n");                 	
                	sb.append(pfx).append("impl putki::TypeDescriptor for " + structName(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; }");                    
                }                

                if (structNameWrap(struct).length() > 0)
                {
                    sb.append("\n");                                            	
                	sb.append(pfx).append("impl putki::TypeDescriptor for " + structNameWrap(struct) + " { const TAG: &'static str = \"" + struct.name + "\"; }");
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
	                */
       		
           		sb.append("\n\t\tPermitAsAsset:" + struct.permitAsAsset);
           		sb.append(",\n\t\tIsTypeRoot:" + struct.isTypeRoot);
           		sb.append(",\n\t\tIsValueType:" + struct.isValueType);
           		if (struct.resolvedParent != null)
           			sb.append(",\n\t\tParent:\""+  struct.resolvedParent.name + "\"");
       			sb.append(",\n\t\tFields: [");
       			
       			boolean ff = true;
                for (Compiler.ParsedField field : struct.fields)
                {
                	if (field.isParentField)
                		continue;
                	if (!ff)
                		sb.append(",");
                	ff = false;
                	sb.append(prefix).append("{ Name:\"" + field.name + "\", Type:\"");
                	switch (field.type)
                	{
                		case INT32: sb.append("I32"); break;
                		case UINT32: sb.append("U32"); break;
                		case FLOAT: sb.append("Float"); break;
                		case BYTE: sb.append("U8"); break;
                		case STRING: if (field.stringIsText) sb.append("Text"); else sb.append("String"); break;
                		case BOOL: sb.append("Bool"); break;
                		case ENUM: sb.append(field.resolvedEnum.name); break;
                		case STRUCT_INSTANCE: sb.append(field.resolvedRefStruct.name); break;
                		case HASH: sb.append("Hash"); break;
                		case POINTER: sb.append(field.resolvedRefStruct.name); break;
                		default: sb.append("Unknown"); break;
                	}
                	sb.append("\"");
                	
                	boolean ptr = field.type == FieldType.POINTER;
                	
                	sb.append(", Array:" + field.isArray + ", Pointer:" + ptr);
                	
                	if (field.defValue != null)
                	{
                		if (field.defValue.startsWith("\""))
                    		sb.append(", Default:" + field.defValue);
                		else
                			sb.append(", Default:\"" + field.defValue + "\"");
                	}
                	/*
                    if (field.isParentField && field.resolvedRefStruct != null && structNameWrap(field.resolvedRefStruct).length() == 0)
                    	continue;
                	if (field.type == FieldType.STRUCT_INSTANCE)
                	{                    		
                		if (field.isArray)
                			sb.append(pfx).append("\t\tfor cont in &mut self." + fieldName(field) + " { _pipeline.build(_br, cont)?; }");
                		else
                			sb.append(pfx).append("\t\t_pipeline.build(_br, &mut self." + fieldName(field) + ")?;");
                	}       
                	*/
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
    
