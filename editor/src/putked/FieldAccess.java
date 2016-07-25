package putked;

import putki.Compiler.ParsedField;


public class FieldAccess<Type>
{
	int m_index;
	int m_arrayIndex;
	DataObject m_object;

	public FieldAccess(ParsedField f)
	{
		m_index = f.index;
	}

	public FieldAccess(DataObject obj, ParsedField f, int arrayIndex)
	{
		m_index = f.index;
		m_object = obj;
		m_arrayIndex = arrayIndex;
	}

	@SuppressWarnings("unchecked")
	public Type get(DataObject obj, int arrayIndex)
	{
		return (Type)obj.getField(m_index, arrayIndex);
	}

	public void set(DataObject obj, int arrayIndex, Type value)
	{
		obj.setField(m_index, arrayIndex, value);
	}

	@SuppressWarnings("unchecked")
	public Type get()
	{
		Object o = m_object.getField(m_index, m_arrayIndex);
		System.out.println("get " + m_object.getType().fields.get(m_index) + "[" + m_arrayIndex + "] = [" + o.toString() + "]");
		return (Type)o;
	}

	public void set(Type value)
	{
		m_object.setField(m_index, m_arrayIndex, value);
	}
}
