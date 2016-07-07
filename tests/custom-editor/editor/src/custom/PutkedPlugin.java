package custom;

public class PutkedPlugin implements putked.EditorPluginDescription
{
        @Override
        public String getName() { return "Custom Plugin"; }

        @Override
        public String getVersion() { return "1.0"; }

        @Override
        public PluginType getType() { return PluginType.PLUGIN_EDITOR; }

        @Override
        public void start()
        {
        	System.out.println("Initializing PutkedPlugin!");
            putked.DataHelper.addTypeService(new putked.inki.CustomEdTest());
            putked.Main.addEditor(new MainEditor());
        }
}

