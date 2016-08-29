package putki;

import java.nio.file.Path;
import java.util.ArrayList;
import putki.Compiler.FieldType;

public class CppGenerator
{
	static class Platform
	{
		public Platform(String _name, int _ptrSize, int _arraySize, int _boolSize, int _enumSize, int _structAlign)
		{
			name =_name;
			ptrSize = _ptrSize;
			arraySize = _arraySize;
			boolSize = _boolSize;
			enumSize = _enumSize;
			lowByteFirst = true;
			structAlign = _structAlign;
		}
		public String name;
		public int ptrSize;
		public int arraySize;
		public int boolSize;
		public int enumSize;
		public int structAlign;
		public boolean lowByteFirst;
	}

	static Platform[] s_platforms = new Platform[] {
		new Platform("x64", 8, 4, 1, 4, 8),
		new Platform("x32", 4, 4, 1, 4, 4),
		new Platform("csharp", 2, 4, 1, 4, 1)
	};

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
		return withUnderscore(s.name);
	}

	public static String fieldName(Compiler.ParsedField s)
	{
		return withUnderscore(s.name);
	}

	public static String enumName(Compiler.ParsedEnum s)
	{
		return s.name;
	}

	public static String enumValue(Compiler.EnumValue s)
	{
		return enumValue(s.name);
	}

	public static String enumValue(String s)
	{
		// god help us.
		int lower = 0;
		int upper = 0;
		int underscore = 0;
		for (int i=0;i<s.length();i++)
		{
			if (s.charAt(i) == '_') underscore++;
			else if (Character.isUpperCase(s.charAt(i))) upper++;
			else if (Character.isLowerCase(s.charAt(i))) lower++;
		}
		if (upper > 0 && underscore > 0 && lower == 0)
			return s;
		return withUnderscore(s).toUpperCase();
	}

	public static String outkiNsName(Platform pf)
	{
		if (pf != null)
			return "outki_ns_" + pf.name;
		else
			return "outki";
	}

	static String inkiFieldtypePod(Compiler.FieldType f)
	{
		switch (f)
		{
			case STRING:
			case PATH:
			case FILE:
				return "std::string";
			case INT32:
				return "int32_t";
			case UINT32:
				return "uint32_t";
			case BYTE:
				return "unsigned char";
			case POINTER:
				return "void*";
			case FLOAT:
				return "float";
			case BOOL:
				return "bool";
			default:
				return "<error>";
		}
	}

	static String outkiFieldtypePod(Compiler.FieldType f)
	{
		switch (f)
		{
			case FILE:
				return "putki::resource_id";
			case STRING:
			case PATH:
				return "const char*";
			case INT32:
				return "int32_t";
			case UINT32:
				return "uint32_t";
			case BYTE:
				return "unsigned char";
			case FLOAT:
				return "float";
			case BOOL:
				return "bool";
			default:
				return "<error>";
		}
	}

	static String inkiFieldType(Compiler.ParsedField pf)
	{
		if (pf.type == FieldType.STRUCT_INSTANCE)
			return "inki::" + structName(pf.resolvedRefStruct);
		else if (pf.type == FieldType.ENUM)
			return "inki::" + enumName(pf.resolvedEnum);
		else if (pf.type == FieldType.POINTER)
			return "putki::ptr<const inki::" + structName(pf.resolvedRefStruct) + "> ";
		return inkiFieldtypePod(pf.type);
	}

	static String inkiFieldTypeArray(Compiler.ParsedField pf)
	{
		if (pf.isArray)
		{
			return "std::vector<" + inkiFieldType(pf) + ">";
		}
		else
		{
			return inkiFieldType(pf);
		}
	}

	static String inkiOutkiInt(int sz)
	{
		return "uint" + (sz*8) + "_t";
	}

	static String inkiOutkiFieldtypePod(Platform p, Compiler.FieldType f)
	{
		switch (f)
		{
			case INT32:
				return "int32_t";
			case UINT32:
				return "uint32_t";
			case BYTE:
				return "unsigned char";
			case FILE:
				return inkiOutkiInt(p.ptrSize);
			case POINTER:
			case STRING:
				return inkiOutkiInt(p.ptrSize);
			case FLOAT:
				return "float";
			case BOOL:
				return inkiOutkiInt(p.boolSize);
			case ENUM:
				return inkiOutkiInt(p.enumSize);
			default:
				return "<error>";
		}
	}

	static String inkiOutkiFieldtype(Platform p, Compiler.ParsedField f)
	{
		switch (f.type)
		{
			case STRUCT_INSTANCE:
				return outkiNsName(p) + "::" + structName(f.resolvedRefStruct);
			default:
				return inkiOutkiFieldtypePod(p, f.type);
		}
	}

	static String outkiArraySizeType(Platform p)
	{
		if (p == null)
		{
			return "unsigned int";
		}
		else
		{
			return inkiOutkiInt(p.arraySize);
		}
	}

	static String outkiFieldType(Platform p, Compiler.ParsedField pf)
	{
		if (p == null)
		{
			if (pf.type == FieldType.STRUCT_INSTANCE)
			{
				return structName(pf.resolvedRefStruct);
			}
			else if (pf.type == FieldType.POINTER)
			{
				return structName(pf.resolvedRefStruct) + "*";
			}
			else if (pf.type == FieldType.ENUM)
			{
				return enumName(pf.resolvedEnum);
			}
			return outkiFieldtypePod(pf.type);
		}
		else
		{
			if (pf.type == FieldType.STRUCT_INSTANCE)
			{
				return structName(pf.resolvedRefStruct);
			}
			else
			{
				return inkiOutkiFieldtypePod(p, pf.type);
			}
		}
	}

	static String outkiFieldTypeArray(Platform p, Compiler.ParsedField pf)
	{
		if (pf.isArray)
			return outkiFieldType(p, pf) + "*";
		else
			return outkiFieldType(p, pf);
	}

	static String getTypeHandler(Compiler.ParsedStruct struct)
	{
		return "type_" + struct.uniqueId + "_" + withUnderscore(struct.name) + "_handler";
	}

	static String getTypeHandlerFn(Compiler.ParsedStruct struct)
	{
		return "get_" + struct.uniqueId + "_" + withUnderscore(struct.name) + "_type_handler";
	}

	static String fillTypeFromParsed(Compiler.ParsedStruct struct)
	{
		return "fill_" + struct.uniqueId + "_" + withUnderscore(struct.name) + "_from_parsed";
	}

	static String writeIntoBlob(Compiler.ParsedStruct struct)
	{
		return "write_" + struct.uniqueId + "_" + withUnderscore(struct.name) + "_into_blob";
	}

	static String writeAux(Compiler.ParsedStruct struct)
	{
		return "write_" + struct.uniqueId + "_" + withUnderscore(struct.name) + "_aux";
	}

	static String outkiPostBlobLoader(Compiler.ParsedStruct struct)
	{
		return "obj_postload_" + struct.uniqueId + "_" + withUnderscore(struct.name);
	}

	static String outkiPtrWalker(Compiler.ParsedStruct struct)
	{
		return "obj_depwalk_" + struct.uniqueId + "_" + withUnderscore(struct.name);
	}

	static String enumFromString(Compiler.ParsedEnum e)
	{
		return withUnderscore(e.name) + "_from_string";
	}

	static String enumToString(Compiler.ParsedEnum e)
	{
		return withUnderscore(e.name) + "_to_string";
	}

	static String rttiField()
	{
		return "_rtti_type";
	}

	public static void writeEnum(Compiler comp, StringBuilder sb, Compiler.ParsedEnum e, String prefix, boolean isRuntime)
	{
		sb.append(prefix).append("enum " + enumName(e) + " {");
		for (Compiler.EnumValue val : e.values)
		{
			sb.append(prefix).append("\t" + enumValue(val) + " = " + val.value + ",");
		}

		if (!isRuntime)
		{
			sb.append(prefix).append("};");
			sb.append(prefix).append("inline " + enumName(e) + " " + enumFromString(e) + "(const char* value)");
			sb.append(prefix).append("{");
			for (Compiler.EnumValue val : e.values)
			{
				sb.append(prefix).append("\tif (!strcmp(value, \"" + val.name + "\")) return " + enumValue(val) + ";");
			}

			sb.append(prefix).append("\treturn (" + enumName(e) + ")0;");
			sb.append(prefix).append("}");
			sb.append(prefix).append("inline const char* " + enumToString(e) + "(" + enumName(e) + " value)");
			sb.append(prefix).append("{");
			sb.append(prefix).append("\tswitch (value)");
			sb.append(prefix).append("\t{");
			for (Compiler.EnumValue val : e.values)
			{
				sb.append(prefix).append("\t\tcase " + enumValue(val) + ": return \"" + val.name + "\";");
			}
			sb.append(prefix).append("\t\tdefault:");
			sb.append(prefix).append("\t\t\treturn \"invalid-enum-" + e.name + "\";");
			sb.append(prefix).append("\t}");
		}
		sb.append(prefix).append("};");
	}

	public static void writeInkiStruct(Compiler comp, StringBuilder sb, Compiler.ParsedStruct struct, String prefix)
	{
		sb.append(prefix).append("putki::type_handler_i* " + getTypeHandlerFn(struct) + "();");
		sb.append(prefix).append("void " + fillTypeFromParsed(struct) + "();");
		sb.append("\n");

		String sn = structName(struct);

		if (struct.resolvedParent != null)
		{
			sb.append(prefix).append("struct " + sn + " : " + structName(struct.resolvedParent) + " {");
		}
		else
		{
			sb.append(prefix).append("struct " + sn + " {");
		}

		String p0 = prefix + "\t";
		String p1 = prefix + "\t\t";

		sb.append(p0).append("static inline " + sn + "* alloc() { return (" + sn + " *) " + getTypeHandlerFn(struct) + "()->alloc(); }");
		sb.append(p0).append("static inline putki::type_handler_i* th() { return " + getTypeHandlerFn(struct) + "(); }");
		sb.append(p0).append("static inline int type_id() { return " + struct.uniqueId + "; }");

		if (struct.isTypeRoot)
		{
			sb.append(p0).append("int32_t & rtti_type_ref() { return " + rttiField() + "; }");
			sb.append(p0).append("int32_t rtti_type_id() const { return " + rttiField() + "; }");
			sb.append(p0).append("int32_t " + rttiField() + ";");
		}

		sb.append(p0).append(sn + "()");
		sb.append(p0).append("{");

		for (Compiler.ParsedField field : struct.fields)
		{
			if (field.isArray)
			{
				continue;
			}
			if (field.type == FieldType.STRUCT_INSTANCE)
			{
				continue;
			}
			if (field.isBuildConfig)
			{
				continue;
			}

			String defValue = field.defValue;

			if (field.type == FieldType.POINTER)
			{
				if (defValue != null && defValue.length() > 0)
				{
					sb.append(p1).append(fieldName(field) + ".init(" + getTypeHandlerFn(field.resolvedRefStruct) + "(), \"" + defValue + "\");");
				}
				else
				{
					sb.append(p1).append(fieldName(field) + ".init(" + getTypeHandlerFn(field.resolvedRefStruct) + "(), 0);");
					continue;
				}
			}

			if (defValue == null)
			{
				if (field.type == FieldType.PATH || field.type == FieldType.FILE ||
					field.type == FieldType.STRING)
				{
					continue;
				}
				else
				{
					defValue = "0";
				}
			}

			if (field.type == FieldType.ENUM)
			{
				sb.append(p1).append(fieldName(field) + " = (" + enumName(field.resolvedEnum) + ") " + enumValue(defValue) + ";");
			}
			else
			{
				sb.append(p1).append(fieldName(field) + " = " + defValue + ";");
			}
		}

		if (struct.isTypeRoot || struct.resolvedParent != null)
		{
			sb.append(p1).append("rtti_type_ref() = type_id();");
		}

		sb.append(p0).append("}");

		for (Compiler.ParsedField field : struct.fields)
		{
			if (field.isParentField)
			{
				continue;
			}
			if (field.isBuildConfig)
			{
				sb.append(p0).append(inkiFieldTypeArray(field) + " & " + fieldName(field) + "(const char *build_config) {");
				for (String s : comp.buildConfigs)
				{
					if (s.equals("Default"))
						continue;
					sb.append(p0).append("\tif (!strcmp(build_config, \"" + s + "\"))");
					sb.append(p0).append("\t\treturn " + withUnderscore(field.name + s) + ";");
				}
				sb.append(p0).append("\treturn " + withUnderscore(field.name + "Default") + ";");
				sb.append(p0).append("}");
				continue;
			}
			sb.append(p0).append(inkiFieldTypeArray(field) + " " + fieldName(field) + ";");
		}

		sb.append(prefix).append("};");
		sb.append("\n");
	}

	public static void writeOutkiStruct(Compiler comp, Platform runtime, StringBuilder sb, Compiler.ParsedStruct struct, String prefix)
	{
		String sn = structName(struct);
		String p0 = prefix + "\t";
		//String p1 = prefix + "\t\t";

		if (struct.resolvedParent != null)
		{
			sb.append(prefix).append("struct " + sn + " : " + structName(struct.resolvedParent) + " {");
		}
		else
		{
			sb.append(prefix).append("struct " + sn + " {");
			if (struct.isTypeRoot)
			{
				sb.append(p0).append("int32_t " + rttiField() + ";");
			}
		}

		if (runtime == null)
		{
			sb.append(p0).append("static inline int type_id() { return " + struct.uniqueId + "; }");
			sb.append(p0).append("enum { TYPE_ID = " + struct.uniqueId + " };");
		}

		if (struct.isTypeRoot)
		{
			sb.append(p0).append("inline int rtti_type_id() { return " + rttiField() + "; }");
			sb.append(p0).append("template<typename T>");
			sb.append(p0).append("inline T* exact_cast() { if (T::TYPE_ID == rtti_type_id()) return (T*)this; else return 0; }");
		}

		for (Compiler.ParsedField field : struct.fields)
		{
			if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
				continue;
			if (field.isParentField)
				continue;
			if (field.isArray)
			{
				sb.append(p0).append(outkiArraySizeType(runtime) + " " + fieldName(field) + "_size;");
				if (runtime == null)
				{
					sb.append(p0).append(outkiFieldType(runtime, field) + "* " + fieldName(field) + ";");
				}
				else
				{
					sb.append(p0).append(inkiOutkiInt(runtime.ptrSize) + " " + fieldName(field) + ";");
				}
			}
			else
			{
				sb.append(p0).append(outkiFieldType(runtime, field) + " " + fieldName(field) + ";");
			}
		}

		sb.append(prefix).append("};");
	}

	public static void writeInkiOutkiFns(Compiler comp, Platform runtime, StringBuilder sb, Compiler.ParsedStruct struct, String prefix)
	{
		String inkiSn = "inki::" + structName(struct);
		String outkiSn = outkiNsName(runtime) + "::" + structName(struct);
		String p0 = prefix + "\t";

		sb.append(prefix).append("char* " + writeAux(struct) + "(" + inkiSn + "* in, " + outkiSn + "* out, char* out_beg, char* out_end)");
		sb.append(prefix).append("{");

		for (Compiler.ParsedField field : struct.fields)
		{
			if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
				continue;

			String refIn = "in->" + fieldName(field);
			String refOut = "out->" + fieldName(field);
			String indent = p0;

			if (field.isParentField)
			{
				continue;
			}

			if (field.isArray)
			{
				String szExpr = refIn + ".size()";
				String outType = inkiOutkiFieldtype(runtime, field);
				sb.append(indent).append("{");
				sb.append(indent).append("\t" + refOut + " = 0;");
				sb.append(indent).append("\t" + refOut + "_size = (" + outkiArraySizeType(runtime) + ")" + szExpr +";");
				sb.append(indent).append("\t" + outType + "* outp = reinterpret_cast<" + outType + "*>(out_beg);");
				sb.append(indent).append("\tout_beg += " + szExpr + " * sizeof(" + outType + ");");
				sb.append(indent).append("\tfor (size_t i=0;i<" + szExpr + ";i++)");
				sb.append(indent).append("\t{");
				refIn = refIn + "[i]";
				refOut = "outp[i]";
				indent = indent + "\t\t";
			}

			switch (field.type)
			{
				case UINT32:
				case INT32:
					sb.append(indent).append("putki::pack_int32_field((char*)&" + refOut + ", " + refIn + ");");
					break;
				case BOOL:
					sb.append(indent).append(refOut + " = " + refIn + " ? 1 : 0;");
					break;
				case BYTE:
				case FLOAT:
					sb.append(indent).append(refOut + " = " + refIn + ";");
					break;
				case ENUM:
					sb.append(indent).append(refOut + " = (" + inkiOutkiInt(runtime.enumSize) + ") " + refIn + ";");
					break;
				case POINTER:
					sb.append(indent).append(refOut + " = (" + inkiOutkiInt(runtime.ptrSize) + ") " + refIn + ".user_data();");
					break;
				case FILE:
					sb.append(indent).append(refOut + " = atoi(" + refIn + ".c_str());");
					break;
				case PATH:
				case STRING:
					sb.append(indent).append("out_beg = putki::pack_string_field(" + runtime.ptrSize + ", (char*)&" + refOut + ", " + refIn + ".c_str(), out_beg, out_end);");
					break;
				case STRUCT_INSTANCE:
					sb.append(indent).append("out_beg = " + writeAux(field.resolvedRefStruct) + "(&" + refIn + ", &" + refOut + ", out_beg, out_end);");
					break;
				default:
					break;
			}

			if (field.isArray)
			{
				sb.append(p0).append("\t}");
				sb.append(p0).append("}");
			}
		}

		sb.append(p0).append("return out_beg;");
		sb.append(prefix).append("}");
		sb.append(prefix).append("char* " + writeIntoBlob(struct) + "(" + inkiSn + "* in, char* out_beg, char* out_end)");
		sb.append(prefix).append("{");
		sb.append(p0).append("if (out_end - out_beg < sizeof(" + outkiSn + ")) return 0;");
		sb.append(p0).append(outkiSn + "* d = (" + outkiSn + "*) out_beg;");

		ArrayList<Compiler.ParsedStruct> parents = new ArrayList<Compiler.ParsedStruct>();
		Compiler.ParsedStruct hr = struct.resolvedParent;
		while (hr != null)
		{
			parents.add(hr);
			hr = hr.resolvedParent;
		}

		sb.append(p0).append("out_beg = out_beg + sizeof(" + outkiSn + ");");
		for (int i=parents.size()-1;i>=0;i--)
		{
			sb.append(p0).append("out_beg = " + writeAux(parents.get(i)) +"(in, d, out_beg, out_end);");
		}

		sb.append(p0).append("return " + writeAux(struct) +"(in, d, out_beg, out_end);");
		sb.append(prefix).append("}");
	}

    public static void generateInkiHeader(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	            Path inki = tree.genCodeRoot.resolve("cpp").resolve("inki");
	            Path headerFn = inki.resolve(file.sourcePath).resolve(file.fileName + ".h");

	            StringBuilder sb = new StringBuilder();
	            sb.append("#pragma once\n\n");
	            sb.append("#include <putki/builder/write.h>\n");
	            sb.append("#include <putki/builder/typereg.h>\n");
	            sb.append("#include <string>\n");
	            sb.append("#include <cstring>\n");
	            sb.append("#include <vector>\n");
	            sb.append("#include <stdint.h>\n");
	            sb.append("\n");
	            for (String include : file.includes)
	            {
	            	sb.append("#include \"" + include.replace("$PFX$", "inki/") + ".h\"\n");
	            }
	            sb.append("\n");
	            sb.append("namespace inki\n");
	            sb.append("{\n");
                for (Compiler.ParsedEnum e : file.enums)
                {
                   	writeEnum(comp, sb, e, "\n\t", false);
                }

                for (Compiler.ParsedStruct struct : file.structs)
                {
                    if ((struct.domains & (Compiler.DOMAIN_OUTPUT | Compiler.DOMAIN_INPUT)) == 0)
                        continue;
                    writeInkiStruct(comp, sb, struct, "\n\t");
                }
	            sb.append("\n}");

	            sb.append("\n");
	            for (Platform p : s_platforms)
	            {
	            	sb.append("\nnamespace " + outkiNsName(p));
	            	sb.append("\n{");
	                for (Compiler.ParsedEnum e : file.enums)
	                {
	                   	writeEnum(comp, sb, e, "\n\t", false);
	                }
					sb.append("\n\t#pragma pack(push, " + p.structAlign + ")");
	                for (Compiler.ParsedStruct struct : file.structs)
	                {
	                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
	                    writeOutkiStruct(comp, p, sb, struct, "\n\t");
	                }
	                sb.append("\n\t#pragma pack(pop)");
	            	sb.append("\n}\n");
	            }
	            writer.addOutput(headerFn, sb.toString().getBytes());
        	}
        }
    }

    public static void generateInkiImplementation(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
            Path inkiMasterFn = tree.genCodeRoot.resolve("cpp").resolve("inki").resolve("inki-master.cpp");
            StringBuilder master = new StringBuilder();

        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	            Path inki = tree.genCodeRoot.resolve("cpp").resolve("inki");
	            Path implFn = inki.resolve(file.sourcePath).resolve(file.fileName + ".cpp");

	            StringBuilder sb = new StringBuilder();
	            sb.append("#include \"" + file.fileName + ".h\"\n");
	            sb.append("#include <putki/blob.h>\n");
	            sb.append("#include <putki/builder/typereg.h>\n");
	            sb.append("#include <putki/builder/write.h>\n");
	            sb.append("#include <putki/builder/parse.h>\n");
	            sb.append("#include <putki/runtime.h>\n");
	            sb.append("#include <putki/sys/sstream.h>\n");
	            sb.append("#include <cstring>\n");
	            sb.append("\n");
	            for (Platform p : s_platforms)
	            {
	            	sb.append("\nnamespace " + outkiNsName(p));
	            	sb.append("\n{");
	                for (Compiler.ParsedStruct struct : file.structs)
	                {
	                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
	                        continue;
	                    writeInkiOutkiFns(comp, p, sb, struct, "\n\t");
	                }
	            	sb.append("\n}\n");
	            }

            	sb.append("\nnamespace inki");
            	sb.append("\n{");

            	for (Compiler.ParsedStruct struct : file.structs)
            	{
                    if ((struct.domains & (Compiler.DOMAIN_OUTPUT | Compiler.DOMAIN_INPUT)) == 0)
                        continue;

            		String sn = structName(struct);
            		String thn = "s_typeHandler" + struct.uniqueId;
            		String pfx0 = "\n\t";
            		String pfx1 = "\n\t\t";
            		String pfx2 = "\n\t\t\t";
            		sb.append(pfx0).append("struct " + getTypeHandler(struct) + " : putki::type_handler_i {");
            		sb.append(pfx1).append("putki::instance_t alloc() { return new " + sn + "; }");
            		sb.append(pfx1).append("putki::instance_t clone(putki::instance_t source) { " + sn + "* tmp = (" + sn + "*)alloc(); *tmp = *((" + sn + "*) source); return tmp; }");
            		sb.append(pfx1).append("void free(putki::instance_t p) { delete (" + sn + "*) p; }");
            		sb.append(pfx1).append("const char* name() { return \"" + struct.name + "\"; }");
            		sb.append(pfx1).append("int id() { return " + struct.uniqueId + "; }");
            		if (struct.resolvedParent != null)
            			sb.append(pfx1).append("type_handler_i* parent_type() { return " + getTypeHandlerFn(struct.resolvedParent) + "(); }");
            		else
            			sb.append(pfx1).append("type_handler_i* parent_type() { return 0; }");
            		sb.append(pfx1).append("bool in_output() { return " + ((struct.domains & Compiler.DOMAIN_OUTPUT) != 0 ? "true" : "false") + "; }");
            		sb.append(pfx1).append("void query_pointers(putki::instance_t source, putki::ptr_query_result* result, bool skip_input_only, bool rtti_dispatch)");
            		sb.append(pfx1).append("{");
            		if (struct.isTypeRoot || struct.resolvedParent != null)
            		{
						sb.append(pfx2).append("if (rtti_dispatch)");
						sb.append(pfx2).append("{");						//
						sb.append(pfx2).append("\tputki::typereg_get_handler(((" + sn + "*)source)->rtti_type_ref())->query_pointers(source, result, skip_input_only, false);");
						sb.append(pfx2).append("\treturn;");
						sb.append(pfx2).append("}");
            		}

					Compiler.ParsedStruct hr = struct.resolvedParent;
					while (hr != null)
					{
						sb.append(pfx2).append(getTypeHandlerFn(hr) + "()->query_pointers(source, result, skip_input_only, false);");
						hr = hr.resolvedParent;
					}

            		boolean first = true;
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if (field.isBuildConfig)
	                    {
	                    	continue;
	                    }
	                    if (field.isParentField)
	                    {
	                    	continue;
	                    }
	                    if (field.type != FieldType.POINTER && field.type != FieldType.STRUCT_INSTANCE)
	                    {
	                    	continue;
	                    }

	                    if (first)
	                    {
            				sb.append(pfx2).append(sn + "* obj = (" + sn + "*) source;");
            				first = false;
            			}

	                	String bpfx = pfx2;
	                	boolean has_if = false;
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0 ||
	                    	(field.type == FieldType.POINTER && (field.resolvedRefStruct.domains & Compiler.DOMAIN_OUTPUT) == 0))
	                    {

	                    	sb.append(bpfx).append("if (!skip_input_only) {");
	                    	bpfx = bpfx + "\t";
	                    	has_if = true;
	                    }

            			String indent = bpfx;
	                    String ref = "obj->" + fieldName(field);

            			if (field.isArray)
            			{
							sb.append(bpfx).append("for (size_t i=0;i<" + ref + ".size();i++)");
							sb.append(bpfx).append("{");
							ref = ref + "[i]";
							indent = pfx2 + "\t";
            			}

	                    switch (field.type)
	                    {
	                    	case POINTER:
	                    		sb.append(indent).append("putki::ptr_add_to_query_result(result, &" + ref + "._ptr);");
	                    		break;
	                    	case STRUCT_INSTANCE:
	                    		sb.append(indent).append(getTypeHandlerFn(field.resolvedRefStruct) + "()->query_pointers(&" + ref + ", result, skip_input_only, false);");
	                    		break;
	                    	default:
	                    		break;
	                    }

						if (field.isArray)
            			{
							sb.append(bpfx).append("}");
            			}

						if (has_if)
						{
							sb.append(pfx2).append("}");
						}
	                }

            		sb.append(pfx1).append("}");
            		sb.append(pfx1).append("void query_files(putki::instance_t source, putki::file_query_result* result, bool skip_input_only, bool rtti_dispatch)");
            		sb.append(pfx1).append("{");
            		if (struct.isTypeRoot || struct.resolvedParent != null)
            		{
						sb.append(pfx2).append("if (rtti_dispatch)");
						sb.append(pfx2).append("{");						//
						sb.append(pfx2).append("\tputki::typereg_get_handler(((" + sn + "*)source)->rtti_type_ref())->query_files(source, result, skip_input_only, false);");
						sb.append(pfx2).append("\treturn;");
						sb.append(pfx2).append("}");
            		}

					hr = struct.resolvedParent;
					while (hr != null)
					{
						sb.append(pfx2).append(getTypeHandlerFn(hr) + "()->query_files(source, result, skip_input_only, false);");
						hr = hr.resolvedParent;
					}

            		first = true;
	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if (field.isBuildConfig || field.isParentField)
	                    {
	                    	continue;
	                    }
	                    if (field.type != FieldType.FILE && field.type != FieldType.POINTER && field.type != FieldType.STRUCT_INSTANCE)
	                    {
	                    	continue;
	                    }
	                    if (first)
	                    {
            				sb.append(pfx2).append(sn + "* obj = (" + sn + "*) source;");
            				first = false;
            			}

	                	String bpfx = pfx2;
	                	boolean has_if = false;
	                    if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0 || (field.type == FieldType.POINTER && (field.resolvedRefStruct.domains & Compiler.DOMAIN_OUTPUT) == 0))
	                    {
	                    	sb.append(bpfx).append("if (!skip_input_only) {");
	                    	bpfx = bpfx + "\t";
	                    	has_if = true;
	                    }

            			String indent = bpfx;
	                    String ref = "obj->" + fieldName(field);

            			if (field.isArray)
            			{
							sb.append(bpfx).append("for (size_t i=0;i<" + ref + ".size();i++)");
							sb.append(bpfx).append("{");
							ref = ref + "[i]";
							indent = pfx2 + "\t";
            			}

	                    switch (field.type)
	                    {
	                    	case FILE:
	                    		sb.append(indent).append("putki::add_to_file_query(result, &" + ref + ");");
	                    		break;
	                    	case STRUCT_INSTANCE:
	                    		sb.append(indent).append(getTypeHandlerFn(field.resolvedRefStruct) + "()->query_files(&" + ref + ", result, skip_input_only, false);");
	                    		break;
	                    	default:
	                    		break;
	                    }

						if (field.isArray)
            			{
							sb.append(bpfx).append("}");
            			}

						if (has_if)
						{
							sb.append(pfx2).append("}");
						}
	                }

            		sb.append(pfx1).append("}");
            		sb.append(pfx1).append("void write_json(putki::instance_t source, putki::sstream & out, int indent) {");

            		boolean firstJson = true;

            		ArrayList<Compiler.ParsedField> tmp = new ArrayList<Compiler.ParsedField>();
            		for (Compiler.ParsedField field : struct.fields)
            		{
	                	if (field.isBuildConfig)
	                		continue;
	                	tmp.add(field);
            		}

					for (int f=0;f<tmp.size();f++)
	                {
		                Compiler.ParsedField field = tmp.get(f);
	                	if (field.isBuildConfig)
	                	{
	                		continue;
	                	}
	                	if (firstJson)
	                	{
	                		sb.append(pfx2).append(sn + "* obj = (" + sn + "*) source;");
	                		sb.append(pfx2).append("char ibuf[128];");
	                		firstJson = false;
	                	}

						String ref = "obj->" + fieldName(field);
	                	String indent = pfx2;
	                	String delim = "";

	                	sb.append(pfx2).append("out << putki::write::json_indent(ibuf, indent+1) << \"\\\"" + field.name + "\\\": \";");

						if (field.isArray && field.type == FieldType.BYTE)
						{
							sb.append(pfx2).append("out << \"\\\"\"; putki::write::json_stringencode_byte_array(out, " + ref + "); out << \"\\\"\";");
							if (f < tmp.size()-1)
								sb.append(pfx2).append("out << \",\";");
							sb.append(pfx2).append("out << \"\\n\";");
							continue;
						}

						if (field.isArray)
						{
							sb.append(pfx2).append("{");
							sb.append(pfx2).append("\tout << \"[\";");
							sb.append(pfx2).append("\tconst char *delim = \"\";");
							sb.append(pfx2).append("\tfor (size_t i=0;i<" + ref + ".size();i++)");
							sb.append(pfx2).append("\t{");
							ref = ref + "[i]";
							delim = "delim <<";
							indent = pfx2 + "\t\t";
						}

						switch (field.type)
						{
							case STRING:
							case FILE:
							case PATH:
								sb.append(indent).append("out << " + delim + "putki::write::json_str(" + ref + ".c_str());");
								break;
							case BOOL:
								sb.append(indent).append("out << " + delim + "(" + ref + " ? 1 : 0);");
							case INT32:
							case UINT32:
								sb.append(indent).append("out << " + delim + ref+ ";");
								break;
							case FLOAT:
								sb.append(indent).append("out << " + delim + ref+ ";");
								break;
							case BYTE:
								sb.append(indent).append("out << " + delim + "(unsigned int)" + ref + ";");
								break;
							case ENUM:
								sb.append(indent).append("out << " + delim + " \"\\\"\" << " + enumToString(field.resolvedEnum) + "(" + ref + ") << \"\\\"\";");
								break;
							case POINTER:
								sb.append(indent).append("out << " + delim + " putki::write::json_str(" + ref + ".path());");
								break;
							case STRUCT_INSTANCE:
								{
									String ptrRef = field.isParentField ? "obj" : ("&" + ref);
									sb.append(indent).append("out << " + delim + "\"{\\n\";");
									sb.append(indent).append(getTypeHandlerFn(field.resolvedRefStruct) + "()->write_json(" + ptrRef + ", out, indent + 1);");
									sb.append(indent).append("out << putki::write::json_indent(ibuf, indent+1) << \"}\";");
									break;
								}
							default:
								sb.append(indent).append("// none " + field.type);
								break;
						}

						if (field.isArray)
						{
							sb.append(pfx2).append("\t\tdelim = \", \";");
							sb.append(pfx2).append("\t}");
							sb.append(pfx2).append("\tout << \"]\";");
							sb.append(pfx2).append("}");
						}

						if (f < tmp.size()-1)
						{
							sb.append(pfx2).append("out << \",\";");
						}

						sb.append(pfx2).append("out << \"\\n\";");
					}

            		sb.append(pfx1).append("}");
            		sb.append(pfx1).append("void fill_from_parsed(putki::parse::node *pn, putki::instance_t target_) {");

            		sb.append(pfx2).append(sn + "* target = (" + sn + "*) target_;");

	                for (Compiler.ParsedField field : struct.fields)
	                {
	                    if ((struct.domains & Compiler.DOMAIN_INPUT) == 0)
	                        continue;
	                    if (field.isBuildConfig)
	                    	continue;

	                    String ref = "target->" + fieldName(field);
	                    String node = "putki::parse::get_object_item(pn, \"" + field.name + "\")";
	                    String indent = pfx2;
	                    String arrIndent = pfx2;

	                    if (field.isParentField)
	                    {
	            			sb.append(pfx2).append("putki::parse::node *parent = " + node + ";");
	            			sb.append(pfx2).append("if (parent)");
	            			sb.append(pfx2).append("{");
	            			sb.append(pfx2).append("\t" + getTypeHandlerFn(struct.resolvedParent) + "()->fill_from_parsed(parent, target_);");
	            			sb.append(pfx2).append("}");
	            			continue;
	                    }

	                    if (field.isArray)
	                    {
	                    	if (field.type == FieldType.BYTE)
	                    	{
	                    		sb.append(pfx2).append("if (!putki::parse::parse_stringencoded_byte_array(" + node + ", " + ref + "))");
	                    		sb.append(pfx2).append("{");
	                    		arrIndent = pfx2 + "\t";
	                    	}
                    		sb.append(arrIndent).append("{");
                    		sb.append(arrIndent).append("\t" + ref + ".clear();");
                    		sb.append(arrIndent).append("\tsize_t i = 0;");
                    		sb.append(arrIndent).append("\tputki::parse::node *arr = " + node + ";");
                    		sb.append(arrIndent).append("\twhile (putki::parse::node * narr = putki::parse::get_array_item(arr, i)) {");
                    		sb.append(arrIndent).append("\t\t" + inkiFieldType(field) + " tmp;");
                    		sb.append(arrIndent).append("\t\t(void)narr;");
                    		sb.append(arrIndent).append("\t\t" + ref + ".push_back(tmp);");
                    		sb.append(arrIndent).append("\t\t++i;");
                    		sb.append(arrIndent).append("\t}");
                    		sb.append(arrIndent).append("\ti = 0;");
                    		sb.append(arrIndent).append("\twhile (putki::parse::node * narr = putki::parse::get_array_item(arr, i)) {");
                    		sb.append(arrIndent).append("\t\t" + inkiFieldType(field) + " & obj = " + ref + "[i];");
                    		ref = "obj";
                    		indent = arrIndent + "\t\t";
                    		node = "narr";
	                    }

	                    String indent2 = indent + "\t";
	                    String indent3 = indent + "\t\t";

						sb.append(indent).append("{");
	                    sb.append(indent2).append("putki::parse::node *n = " + node + ";");
	                    sb.append(indent2).append("if (n)");
	                    sb.append(indent2).append("{");

	                    switch (field.type)
	                    {
	                    	case STRING:
	                    	case FILE:
	                    	case PATH:
	                    		sb.append(indent3).append(ref + " = putki::parse::get_value_string(n);");
	                    		break;
	                    	case FLOAT:
	                    		sb.append(indent3).append(ref + " = (float)atof(putki::parse::get_value_string(n));");
	                    		break;
	                    	case UINT32:
	                    	case INT32:
	                    	case BYTE:
	                    		sb.append(indent3).append(ref + " = (" + inkiFieldtypePod(field.type) + ") putki::parse::get_value_int(n);");
	                    		break;
	                    	case BOOL:
	                    		sb.append(indent3).append(ref + " = putki::parse::get_value_int(n) != 0;");
	                    		break;
	                    	case STRUCT_INSTANCE:
	                    		sb.append(indent3).append(getTypeHandlerFn(field.resolvedRefStruct) + "()->fill_from_parsed(n, &" + ref + ");");
	                    		break;
	                    	case ENUM:
	                    		sb.append(indent3).append(ref + " = " + enumFromString(field.resolvedEnum) + "(putki::parse::get_value_string(n));");
	                    		break;
	                    	case POINTER:
	                    		sb.append(indent3).append(ref + ".set_path(putki::parse::get_value_string(n));");
	                    		break;
							default:
	                    }
	                    sb.append(indent2).append("}");
	                    sb.append(indent).append("}");

	                    if (field.isArray)
	                    {
	                    	sb.append(arrIndent).append("\t\ti++;");
	                    	sb.append(arrIndent).append("\t}");
	                    	sb.append(arrIndent).append("}");
	                    	if (field.type == FieldType.BYTE)
	                    	{
	                    		sb.append(pfx2).append("}");
	                    	}
	                    }
	                }

            		sb.append(pfx1).append("}");
            		sb.append(pfx1).append("char* write_into_buffer(putki::runtime::descptr rt, putki::instance_t source, char *beg, char *end) {");

            		if ((struct.domains & Compiler.DOMAIN_OUTPUT) != 0)
            		{
	            		for (Platform p : s_platforms)
	            		{
	            			sb.append(pfx2).append("if (rt->ptr_size == " + p.ptrSize + " && rt->array_size == " + p.arraySize + " && rt->bool_size == " + p.boolSize + " && rt->enum_size == " + p.enumSize + " && rt->struct_align == " + p.structAlign + ")");
	            			sb.append(pfx2).append("\treturn " + outkiNsName(p) + "::" + writeIntoBlob(struct) + "((" + sn + "*) source, beg, end);");
	            		}
            		}

            		sb.append(pfx2).append("return 0;");
            		sb.append(pfx1).append("}");

            		sb.append(pfx0).append("} " + thn + ";");
            		sb.append(pfx0).append("putki::type_handler_i* " + getTypeHandlerFn(struct) + "() { return &" + thn + "; }");
            	}

				sb.append("\n}\n");
	            writer.addOutput(implFn, sb.toString().getBytes());
	            master.append("#include \"" + inki.relativize(implFn) + "\"\n");
        	}

			master.append("\n");
        	master.append("\nnamespace inki");
        	master.append("\n{");
  			master.append("\n\tvoid bind_" + withUnderscore(tree.moduleName) + "()");
    		master.append("\n\t{");
        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	        	for (Compiler.ParsedStruct struct : file.structs)
    	    	{
                    if ((struct.domains & (Compiler.DOMAIN_OUTPUT | Compiler.DOMAIN_INPUT)) == 0)
                        continue;
    	    		master.append("\n\t\tputki::typereg_register(\"" + struct.name + "\", " + getTypeHandlerFn(struct) + "());");
				}
			}
    		master.append("\n\t}");
        	master.append("\n}");
        	master.append("\n");

        	writer.addOutput(inkiMasterFn, master.toString().getBytes());
        }
    }


    public static void generateOutkiHeader(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
        	Path outki = tree.genCodeRoot.resolve("cpp").resolve("outki");
        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	            Path headerFn = outki.resolve(file.sourcePath).resolve(file.fileName + ".h");

	            StringBuilder sb = new StringBuilder();
	            sb.append("#pragma once\n\n");
	            sb.append("#include <stdint.h>\n");
	            sb.append("#include <putki/types.h>\n");

	            for (String include : file.includes)
	            {
	            	sb.append("#include \"" + include.replace("$PFX$", "outki/") + ".h\"\n");
	            }

	            sb.append("\n");
	            sb.append("namespace outki\n");
	            sb.append("{");

				if (file.enums.size() > 0)
				{
	                for (Compiler.ParsedEnum e : file.enums)
	                {
	                   	writeEnum(comp, sb, e, "\n\t", true);
	                   	sb.append("\n");
	                }
	            }

                for (Compiler.ParsedStruct struct : file.structs)
                {
                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                        continue;
                    writeOutkiStruct(comp, null, sb, struct, "\n\t");
					sb.append("\n\t").append("char* " + outkiPostBlobLoader(struct) + "(void* object, char* beg, char* end);");
					sb.append("\n\t").append("void " + outkiPtrWalker(struct) + "(void* object, putki::objwalker_callback_ptr callback_ptr, putki::objwalker_callback_res callback_res, void* user_data);");
                }
	            sb.append("\n}");
	            writer.addOutput(headerFn, sb.toString().getBytes());
        	}

	        // Master header
	        Path projHeaderFn = outki.resolve(withUnderscore(tree.moduleName) + ".h");
	        StringBuilder sb = new StringBuilder();
	        sb.append("#pragma once\n");
	        sb.append("\nnamespace outki");
	        sb.append("\n{");
	        sb.append("\n\tvoid bind_" + withUnderscore(tree.moduleName) + "();");
	        sb.append("\n}");
	        writer.addOutput(projHeaderFn, sb.toString().getBytes());
        }

    }

    public static void generateOutkiImplementation(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
            Path inkiMasterFn = tree.genCodeRoot.resolve("cpp").resolve("outki").resolve("outki-master.cpp");
            StringBuilder master = new StringBuilder();

            master.append("#include <putki/types.h>\n");
            master.append("#include <putki/blob.h>\n");
            master.append("#include <stddef.h>\n");

        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	            Path outki = tree.genCodeRoot.resolve("cpp").resolve("outki");
	            Path hdrFn = outki.resolve(file.sourcePath).resolve(file.fileName + ".h");
	            master.append("#include \"" + outki.relativize(hdrFn).toString().replace('\\', '/') + "\"\n");
        	}

        	master.append("\nnamespace outki");
        	master.append("\n{");

        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	        	for (Compiler.ParsedStruct struct : file.structs)
    	    	{
                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                        continue;

    	    		String p0 = "\n\t";
    	    		String p1 = "\n\t\t";
    	    		master.append(p0).append("char* " + outkiPostBlobLoader(struct) + "(void* object, char* beg, char* end)");
    	    		master.append(p0).append("{");

    	    		master.append(p1).append(structName(struct) + "* obj = (" + structName(struct) + "*) object;");
	    			if (struct.resolvedParent != null)
	    			{
						master.append(p1).append("beg = " + outkiPostBlobLoader(struct.resolvedParent) + "(object, beg, end);");
	    			}

    	    		for (Compiler.ParsedField field : struct.fields)
    	    		{
    	    			if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
    	    			{
    	    				continue;
    	    			}
    	    			if (field.isParentField)
    	    			{
    	    				continue;
    	    			}

    	    			String ref = "obj->" + fieldName(field);
    	    			String indent = p1;

    	    			if (field.isArray)
    	    			{
    	    				master.append(p1).append(ref + " = (" + outkiFieldTypeArray(null, field) + ")beg;");
    	    				master.append(p1).append("beg += " + ref + "_size * sizeof(" + outkiFieldType(null, field) + ");");
    	    				boolean doLoop = false;
    	    				switch (field.type)
    	    				{
	    	    				case FILE:
	    	    				case PATH:
	    	    				case STRING:
	    	    				case STRUCT_INSTANCE:
		    	    				master.append(p1).append("for (size_t i=0;i!=" + ref + "_size;i++) {");
		    	    				ref = ref + "[i]";
		    	    				indent = indent + "\t";
		    	    				doLoop = true;
		    	    				break;
		    	    			default:
		    	    				break;
    	    				}
    	    				if (!doLoop)
    	    				{
    	    					continue;
    	    				}
    	    			}

    	    			switch (field.type)
    	    			{
    	    				case PATH:
    	    				case STRING:
								master.append(indent).append("beg = putki::post_blob_load_string(&" + ref + ", beg, end);");
								break;
							case STRUCT_INSTANCE:
								master.append(indent).append("beg = " + outkiPostBlobLoader(field.resolvedRefStruct) + "(&" + ref + ", beg, end);");
								break;
    	    				default:
    	    					break;
    	    			}

    	    			if (field.isArray)
    	    			{
    	    				master.append(p1).append("}");
    	    			}
    	    		}

    	    		if (struct.isTypeRoot || struct.resolvedParent != null)
    	    		{
    	    			master.append(p1).append("obj->" + rttiField() + " = " + structName(struct) + "::TYPE_ID;");
    	    		}

    	    		master.append(p0).append("\treturn beg;");
    	    		master.append(p0).append("}");
    	    		master.append(p0).append("void " + outkiPtrWalker(struct) + "(void* object, putki::objwalker_callback_ptr callback_ptr, putki::objwalker_callback_res callback_res, void* user_data)");
    	    		master.append(p0).append("{");

	    			if (struct.resolvedParent != null)
	    			{
						master.append(p1).append(outkiPtrWalker(struct.resolvedParent) + "(object, callback_ptr, callback_res, user_data);");
	    			}

	    			boolean first = true;
	    			boolean firstPtr = true;
    	    		for (Compiler.ParsedField field : struct.fields)
    	    		{
    	    			if ((field.domains & Compiler.DOMAIN_OUTPUT) == 0)
    	    			{
    	    				continue;
    	    			}
    	    			if (field.isParentField)
    	    			{
    	    				continue;
    	    			}
    	    			if (field.type != FieldType.STRUCT_INSTANCE && field.type != FieldType.POINTER && field.type != FieldType.FILE)
    	    			{
    	    				continue;
    	    			}
	   	    			if (first)
    	    			{
    	    				master.append(p1).append(structName(struct) + "* obj = (" + structName(struct) + "*) object;");
    	    				first = false;
    	    			}
    	    			if (field.type == FieldType.POINTER)
    	    			{
		   	    			if (firstPtr)
		   	    			{
	    	    				master.append(p1).append("putki::ptr_info ptr_info;");
	    	    				firstPtr = false;
		   	    			}
	    	    			master.append(p1).append("ptr_info.walker = " + outkiPtrWalker(field.resolvedRefStruct) + ";");
    	    			}

    	    			String ref = "obj->" + fieldName(field);
    	    			String indent = p1;


    	    			if (field.isArray)
    	    			{
    	    				master.append(p1).append("for (size_t i=0;i!=" + ref + "_size;i++) {");
    	    				ref = ref + "[i]";
    	    				indent = indent + "\t";
    	    			}

    	    			switch (field.type)
    	    			{
							case STRUCT_INSTANCE:
								master.append(indent).append(outkiPtrWalker(field.resolvedRefStruct) + "(&" + ref + ", callback_ptr, callback_res, user_data);");
								break;
							case POINTER:
								master.append(indent).append("ptr_info.ptr = ((putki::instance_t*)&" + ref + ");");
								master.append(indent).append("callback_ptr(&ptr_info, user_data);");
								break;
							case FILE:
								master.append(indent).append("callback_res(&" + ref + ", user_data);");
								break;
    	    				default:
    	    					break;
    	    			}

    	    			if (field.isArray)
    	    			{
    	    				master.append(p1).append("}");
    	    			}
    	    		}
    	    		master.append(p0).append("}");
				}
			}
        	master.append("\n");
  			master.append("\n\tvoid bind_" + withUnderscore(tree.moduleName) + "()");
    		master.append("\n\t{");

    		master.append("\n\t\tstatic const putki::type_record entries[] = {");
        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
	        	for (Compiler.ParsedStruct struct : file.structs)
    	    	{
                    if ((struct.domains & Compiler.DOMAIN_OUTPUT) == 0)
                        continue;
    	    		master.append("\n\t\t\t{" + struct.uniqueId + ", sizeof(" + structName(struct) + "), " + outkiPostBlobLoader(struct) + ", " + outkiPtrWalker(struct) + "},");
				}
			}
			master.append("\n\t\t};");
			master.append("\n\t\tputki::insert_type_records(&entries[0], &entries[sizeof(entries)/sizeof(entries[0])]);");
    		master.append("\n\t}");
        	master.append("\n}");
        	master.append("\n");

        	writer.addOutput(inkiMasterFn, master.toString().getBytes());
        }
    }

}
