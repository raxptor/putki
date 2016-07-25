package putked;

import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Label;
import putki.Compiler;

public class EditorCreatorUtil
{
    public static Node makeArrayFieldLabel(Compiler.ParsedField fi, int index)
    {
        Label lbl = new Label(fi.name + "[" + index + "]");
        lbl.setMinWidth(120);
        return lbl;
    }

    public static Node makeFieldLabel(Compiler.ParsedField fi)
    {
        Label lbl = new Label(fi.name);
        lbl.getStyleClass().add("field-label");
        lbl.setAlignment(Pos.CENTER_LEFT);
        return lbl;
    }

    public static Node makeInlineEditorHeader(String name)
    {
        Label lbl = new Label(name);
        lbl.getStyleClass().add("field-label");
        lbl.setAlignment(Pos.CENTER_LEFT);
        return lbl;
    }

    public static Node makeLabel(Compiler.ParsedField fi, int index)
    {
        if (fi.isArray)
            return makeArrayFieldLabel(fi, index);
        else
            return makeFieldLabel(fi);
    }

    public static String makeInlineAuxTitle(DataObject mi)
    {
        return mi.getType().name + " [" + filterAux(mi.getPath()) + "]";
    }

    public static String makeInlineTitle(DataObject mi)
    {
        return mi.getPath() + " (" + mi.getType().name + ")";
    }

    public static String filterAux(String input)
    {
    	for (int i=0;i<input.length();i++)
    		if (input.charAt(i) == '#')
    			return input.substring(i);
    	return input;
    }
}
