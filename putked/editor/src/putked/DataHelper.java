package putked;

import java.util.ArrayList;

public class DataHelper
{
	public static ArrayList<EditorTypeService> _installed = new ArrayList<>();

	public static void addTypeService(EditorTypeService serv)
	{
		_installed.add(serv);
	}

	public static ProxyObject createPutkEdObj(DataObject obj)
	{
		for (EditorTypeService srv : _installed)
		{
			ProxyObject tmp = srv.createProxy(obj.getType());
			if (tmp != null)
			{
				tmp.connect(obj);
				return tmp;
			}
		}
		return null;
	}
}
