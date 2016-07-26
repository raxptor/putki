package putki;

import java.nio.file.Path;

import putki.Compiler.FieldType;

public class CppGenerator
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


		for (int i=0;i<input.length();i++)
		{
			if (i > 0 && upc[i])
			{
				sb.append('_');
			}

			char c = input.charAt(i);
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

    public static void generateInkiHeader(Compiler comp, CodeWriter writer)
    {
        for (Compiler.ParsedTree tree : comp.allTrees())
        {
        	for (Compiler.ParsedFile file : tree.parsedFiles)
        	{
        		/*
	            Path inki = tree.genCodeRoot.resolve("cpp").resolve("inki");
	            Path fn = inki.resolve(file.sourcePath).resolve(tree.moduleName + ".h");

	            StringBuilder sb = new StringBuilder();
	            String guard = "__INKI_" + withUnderscore(file.sourcePath) + "_H__";
	            sb.append("#ifndef " + guard + "\n");
	            sb.append("#define " + guard + "\n");

	            sb.append("#endif");

	            sb.append("\n}\n");
	            writer.addOutput(fn, sb.toString().getBytes());
	            */
        	}
        }
    }
}
