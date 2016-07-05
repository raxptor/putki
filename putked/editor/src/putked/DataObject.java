package putked;

import java.util.HashMap;
import java.util.List;

import putki.Compiler;
import putki.Compiler.ParsedField;

public class DataObject
{
	public DataObject(Compiler.ParsedStruct struct, String path)
	{
		m_data = new Object[struct.fields.size()];
		m_type = struct;
		m_path = path;
	}

	public Object makeDefaultValue(Compiler.ParsedField field)
	{
		switch (field.type)
		{
			case STRUCT_INSTANCE:
			{
				return new DataObject(field.resolvedRefStruct, "tmp-struct");
			}
			case UINT32:
			case INT32:
			case BYTE:
				try
				{
					return Long.parseLong(field.defValue);
				}
				catch (NumberFormatException e)
				{
					return new Long(0);
				}
			case FLOAT:
				try
				{
					return Float.parseFloat(field.defValue);
				}
				catch (NumberFormatException e)
				{
					return new Float(0.0f);
				}
			case BOOL:
				try
				{
					return Boolean.parseBoolean(field.defValue);
				}
				catch (NumberFormatException e)
				{
					return false;
				}
			default:
				if (field.defValue == null)
					return "";
				else
					return field.defValue;
		}
	}

	@SuppressWarnings("unchecked")
	public Object getField(int index, int arrayIndex)
	{
		Compiler.ParsedField field = m_type.fields.get(index);
		if (m_data[index] == null)
		{
			return makeDefaultValue(field);
		}

		if (field.isArray)
		{
			List<Object> list = (List<Object>) m_data[index];
			Object o = list.get(arrayIndex);
			if (o != null)
				return o;
			return makeDefaultValue(field);
		}

		return m_data[index];
	}

	@SuppressWarnings("unchecked")
	public void setField(int index, int arrayIndex, Object value)
	{
		System.out.println(m_path + ":" + m_type.fields.get(index).name + "[" + arrayIndex +"] = " + value.toString());
		Compiler.ParsedField fld = m_type.fields.get(index);
		if (!fld.isArray)
		{
			// Maybe check type?
			m_data[index] = value;
			return;
		}

		List<Object> list = (List<Object>) m_data[index];
		if (list == null)
		{
			list = new java.util.ArrayList<Object>();
			m_data[index] = list;
		}

		if (arrayIndex == list.size())
		{
			list.add(value);
		}
		else if (arrayIndex < list.size())
		{
			list.set(arrayIndex, value);
		}
		else
		{
			System.out.println("Array index out of range when setting field " + fld.name + ", ignoring");
		}
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
		if (!m_type.fields.get(field).isArray)
		{
			System.out.println("help!");
		}

		List<Object> list = (List<Object>) m_data[field];
		if (list == null)
			return 0;
		return list.size();
	}

	public void arrayErase(int field, int index)
	{
		@SuppressWarnings("unchecked")
		List<Object> list = (List<Object>) m_data[field];
		list.remove(index);
	}

	public void arrayInsert(int field, int index)
	{
		@SuppressWarnings("unchecked")
		List<Object> list = (List<Object>) m_data[field];
		list.add(index, null);
	}

	public DataObject getAux(String ref)
	{
		if (m_auxObjects == null)
			return null;
		return m_auxObjects.get(ref);
	}

	public void addAux(String ref, DataObject aux)
	{
		if (m_auxObjects == null)
		{
			m_auxObjects = new HashMap<String, DataObject>();
		}
		System.out.println("Adding aux object " + ref + " to " + aux.getPath());
		m_auxObjects.put(ref, aux);
	}

	public Object[] m_data;
	public Compiler.ParsedStruct m_type;
	public String m_path;
	public HashMap<String, DataObject> m_auxObjects;
}