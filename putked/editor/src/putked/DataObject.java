package putked;

import java.util.HashMap;
import putki.Compiler;

public class DataObject
{
	public DataObject(Compiler.ParsedStruct struct, String path)
	{
		m_data = new Object[struct.fields.size()];
		m_type = struct;
		m_path = path;
	}

	public Object getField(int index)
	{
		if (index >= 0 && index < m_data.length)
			return m_data[index];
		return null;
	}

	public void setField(int index, Object value)
	{
		if (index >= 0 && index < m_data.length)
			m_data[index] = value;
	}

	public Object getField(int index, int arrayIndex)
	{
		return null;
	}

	public void setField(int index, int arrayIndex, Object value)
	{

	}

	public Compiler.ParsedStruct getType()
	{
		return m_type;
	}

	public String getPath()
	{
		return m_path;
	}

	public String getContentHash()
	{
		return "abcdefgh";
	}

	public DataObject createAuxInstance(Compiler.ParsedStruct type)
	{
		return null;
	}

	public int getArraySize(int field)
	{
		return 0;
	}

	public void arrayErase(int field, int index)
	{

	}

	public void arrayInsert(int field, int index)
	{

	}

	public Object[] m_data;
	public Compiler.ParsedStruct m_type;
	public String m_path;
	public HashMap<String, DataObject> m_auxObjects;
}
