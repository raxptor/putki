package putked;

import java.security.SecureRandom;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;

import putki.Compiler;
import putki.Compiler.FieldType;

public class DataObject
{
	public DataObject(Compiler.ParsedStruct struct, String path)
	{
		m_data = new Object[struct.fields.size()];
		m_type = struct;
		m_path = path;
		m_root = this;
	}

	public DataObject(Compiler.ParsedStruct struct, DataObject root, String path)
	{
		m_data = new Object[struct.fields.size()];
		m_type = struct;
		m_path = path;
		m_root = root;
	}

	public DataObject getRootAsset()
	{
		return m_root;
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
					if (field.defValue == null || field.defValue.length() == 0)
					{
						return 0L;
					}
					return Long.parseLong(field.defValue);
				}
				catch (NumberFormatException e)
				{
					return new Long(0);
				}
			case FLOAT:
				try
				{
					if (field.defValue == null || field.defValue.length() == 0)
					{
						return 0.0f;
					}
					return Float.parseFloat(field.defValue);
				}
				catch (NumberFormatException e)
				{
					return new Float(0.0f);
				}
			case BOOL:
				try
				{
					if (field.defValue == null || field.defValue.length() == 0)
					{
						return false;
					}
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

	public Object getField(int index, int arrayIndex)
	{
		return getField(index, arrayIndex, true);
	}

	@SuppressWarnings("unchecked")
	public Object getField(int index, int arrayIndex, boolean makeDefault)
	{
		Compiler.ParsedField field = m_type.fields.get(index);
		if (m_data[index] == null)
		{
			return makeDefault ? makeDefaultValue(field) : null;
		}

		if (field.isArray)
		{
			List<Object> list = (List<Object>) m_data[index];
			Object o = list.get(arrayIndex);
			if (o != null)
				return o;
			return makeDefault ? makeDefaultValue(field) : null;
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
		if (this != m_root)
		{
			return m_root.createAuxInstance(type);
		}

		if (m_auxObjects == null)
		{
			m_auxObjects = new HashMap<>();
		}

		SecureRandom r = new SecureRandom();
		String pick = "0123456789abcdefghijklmnopqrstuvxyz";
		while (true)
		{
			StringBuilder tmp = new StringBuilder();
			tmp.append("#");
			for (int i=0;i<5;i++)
			{
				tmp.append(pick.charAt(r.nextInt(pick.length())));
			}
			String ref = tmp.toString();
			if (!m_auxObjects.containsKey(ref))
			{
				DataObject aux = new DataObject(type, this, m_path + ref);
				System.out.println("Created aux [" + ref + "] onto [" + m_path + "]");
				m_auxObjects.put(ref, aux);
				return aux;
			}
		}
	}

	@SuppressWarnings("unchecked")
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
		if (list == null)
		{
			list = new ArrayList<Object>();
			m_data[field] = list;
		}

		Compiler.ParsedField fld = m_type.fields.get(field);
		if (fld.type == FieldType.STRUCT_INSTANCE)
		{
			list.add(index, new DataObject(fld.resolvedRefStruct, getRootAsset(), ""));
		}
		else
		{
			list.add(index, null);
		}
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
		System.out.println("Adding aux object " + ref + " as " + aux.getPath() + " onto " + getPath());
		m_auxObjects.put(ref, aux);
	}

	Object[] m_data;
	Compiler.ParsedStruct m_type;
	String m_path;
	HashMap<String, DataObject> m_auxObjects;
	DataObject m_root;
}
