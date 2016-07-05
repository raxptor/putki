package putki;

import java.io.IOException;
import java.nio.file.*;
import java.util.*;

public class CodeWriter
{
	HashMap<Path, byte[]> m_results;

	public CodeWriter()
	{
		m_results = new HashMap<>();
	}

	public void addOutput(Path p, byte[] blob)
	{
		System.out.println("adding [" + p.toAbsolutePath() + "] " + blob.length);
		m_results.put(p,  blob);
	}

	public void write()
	{
		for (Map.Entry<Path, byte[]> entry : m_results.entrySet())
		{
			try
			{
				Files.createDirectories(entry.getKey().getParent());
				Files.write(entry.getKey(),  entry.getValue());
			}
			catch (IOException e)
			{
				e.printStackTrace();
			}
		}
	}
}
