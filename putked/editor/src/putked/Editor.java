package putked;

import javafx.scene.Node;
import putki.Compiler;

public interface Editor
{
	String getName();
	int getPriority();
	boolean canEdit(Compiler.ParsedStruct type);
	Node createUI(DataObject obj);
}
