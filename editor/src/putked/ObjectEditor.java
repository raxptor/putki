package putked;

import java.util.ArrayList;

import putki.Compiler;
import putki.Compiler.ParsedField;
import putki.Compiler.ParsedStruct;
import javafx.event.ActionEvent;
import javafx.event.EventHandler;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.geometry.VPos;
import javafx.scene.*;
import javafx.scene.control.*;
import javafx.scene.input.DragEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.input.TransferMode;
import javafx.scene.layout.*;

class StringEditor implements FieldEditor
{
    FieldAccess<String> m_f;

    public StringEditor(DataObject mi, Compiler.ParsedField f, int index)
    {
        m_f = new FieldAccess<String>(mi, f, index);
    }

    @Override
    public Node createUI()
    {
        TextField tf = new TextField(m_f.get());
        tf.textProperty().addListener( (obs, oldValue, newValue) -> {
            m_f.set(newValue);
        });
        return tf;
    }
}

class FileEditor implements FieldEditor
{
    FieldAccess<String> m_f;

    public FileEditor(DataObject mi, Compiler.ParsedField f, int index)
    {
        m_f = new FieldAccess<String>(mi, f, index);
    }

    @Override
    public Node createUI()
    {
        TextField tf = new TextField();
        tf.getStyleClass().add("file-field");
        tf.textProperty().addListener( (obs, oldValue, newValue) -> {
    		tf.getStyleClass().remove("error");
        	java.io.File f = new java.io.File(Main.s_instance.translateResPath(newValue));
        	if (!f.exists() || f.isDirectory())
        		tf.getStyleClass().add("error");
        	if (!newValue.equals(m_f.get()))
        		m_f.set(newValue);
        });
        tf.setText(m_f.get());
        return tf;
    }
}

class BooleanEditor implements FieldEditor
{
    FieldAccess<Boolean> m_f;
    String m_name;

    public BooleanEditor(DataObject mi, Compiler.ParsedField f, int index)
    {
    	m_name = f.name;
        m_f = new FieldAccess<Boolean>(mi, f, index);
    }

    @Override
    public Node createUI()
    {
        CheckBox cb = new CheckBox(m_name);
        cb.setSelected(m_f.get());
        cb.selectedProperty().addListener( (obs, old, ny) -> {
        	m_f.set(ny);
        });
        return cb;
    }
}

class EnumEditor implements FieldEditor
{
    DataObject m_mi;
    FieldAccess<String> m_f;
    ParsedField m_field;

    public EnumEditor(DataObject mi, Compiler.ParsedField f, int index)
    {
        m_f = new FieldAccess<String>(mi, f, index);
        m_field = f;
    }

    @Override
    public Node createUI()
    {
        ComboBox<String> cb = new ComboBox<>();
        ArrayList<String> values = new ArrayList<>();

        Compiler.ParsedEnum penum = m_field.resolvedEnum;
        for (Compiler.EnumValue v : penum.values)
        {
            values.add(v.name);
        }

        cb.getItems().setAll(values);
        cb.setValue(m_f.get());
        cb.valueProperty().addListener( (obs, oldValue, newValue) -> {
        	m_f.set(newValue);
        });

        return cb;
    }
}

class IntegerEditor implements FieldEditor
{
    FieldAccess<Long> m_f;
    long m_min, m_max;

    public IntegerEditor(DataObject mi, ParsedField f, int index, long min, long max)
    {
        m_f = new FieldAccess<Long>(mi, f, index);
        m_min = min;
        m_max = max;
    }

    @Override
    public Node createUI()
    {
        TextField tf = new TextField(m_f.get().toString());
        tf.getStyleClass().add("integer-field");
        tf.textProperty().addListener( (obs, oldValue, newValue) -> {
            try
            {
                long val = Long.parseLong(newValue);
                if (val < m_min || val > m_max)
                	throw new NumberFormatException("Out of range");
                m_f.set(val);
                tf.getStyleClass().remove("error");
            }
            catch (NumberFormatException u)
            {
                tf.getStyleClass().remove("error");
                tf.getStyleClass().add("error");
            }
        });
        return tf;
    }
}

class FloatEditor implements FieldEditor
{
    FieldAccess<Float> m_f;

    public FloatEditor(DataObject mi, ParsedField f, int index)
    {
        m_f = new FieldAccess<Float>(mi, f, index);
    }

    @Override
    public Node createUI()
    {
        TextField tf = new TextField(m_f.get().toString());
        tf.getStyleClass().add("float-field");
        tf.textProperty().addListener( (obs, oldValue, newValue) -> {
            try
            {
                float f = Float.parseFloat(newValue);
                m_f.set(f);
                tf.getStyleClass().remove("error");
            }
            catch (NumberFormatException u)
            {
                tf.getStyleClass().add("error");
            }
        });
        return tf;
    }
}

class PointerEditor implements FieldEditor
{
    FieldAccess<String> m_f;
    ParsedField m_field;
    DataObject m_mi;

    public PointerEditor(DataObject mi, ParsedField f, int index)
    {
        m_f = new FieldAccess<String>(mi, f, index);
        m_mi = mi;
        m_field = f;
    }

    @Override
    public Node createUI()
    {
        VBox tot = new VBox();

        HBox ptrbar = new HBox();

        ptrbar.setMaxWidth(Double.MAX_VALUE);

        TextField tf = new TextField(m_f.get());
        tf.setEditable(false);
        tf.setDisable(true);
        tf.setMinWidth(200);
        HBox.setHgrow(tf, Priority.ALWAYS);

        Button clear = new Button("X");
        Button point = new Button("*");

        clear.getStyleClass().add("rm-button");
        point.getStyleClass().add("point-button");

        ptrbar.getChildren().setAll(tf, point, clear);
        tot.getChildren().setAll(ptrbar);

        clear.setOnAction( (evt) -> {
        		m_f.set("");
                tf.textProperty().set("");
                tot.getChildren().setAll(ptrbar);
                ptrbar.getChildren().setAll(tf, point);
        });

        point.setOnAction( (evt) -> {

            if (m_field.isAuxPtr)
            {
                Compiler.ParsedStruct t = Main.s_instance.askForSubType(Main.s_compiler.getTypeByName(m_field.refType), true);
                if (t != null)
                {
                    DataObject naux = m_mi.createAuxInstance(t);

                    m_f.set(naux.getPath());
                    tf.textProperty().set(EditorCreatorUtil.makeInlineAuxTitle(naux));

                    VBox aux = makeObjNode(naux);
                    aux.setVisible(true);
                    aux.setManaged(true);

                    Button expand = new Button("V");
                    expand.setOnAction(new EventHandler<ActionEvent>() {
                        @Override
                        public void handle(ActionEvent event) {
                            aux.setVisible(!aux.isVisible());
                            aux.setManaged(aux.isVisible());
                        }
                    });

                    // configure for this
                    ptrbar.getChildren().setAll(expand, tf, point, clear);
                    tot.getChildren().setAll(ptrbar, aux);
                }
            }
            else
            {
                String path = Main.s_instance.askForInstancePath(Main.s_compiler.getTypeByName(m_field.refType));
                if (path != null)
                {
                    m_f.set(path);
                    tf.textProperty().set(path);
                    ptrbar.getChildren().setAll(tf, point, clear);
                    tot.getChildren().setAll(ptrbar);
                }
            }
        });

        Compiler.ParsedStruct refType = Main.s_compiler.getTypeByName(m_field.refType);

        if (!m_field.isAuxPtr)
        {
        	tot.setOnDragOver(new EventHandler<DragEvent>() {
        	    public void handle(DragEvent event) {
	                if (event.getGestureSource() != tf && event.getDragboard().hasString()) {
	                	DataObject dragging = Main.s_instance.load(event.getDragboard().getString());
	                	if (dragging != null && dragging.getType().hasParent(refType)) {
	                		event.acceptTransferModes(TransferMode.LINK);
	                	}
	                }
        	        event.consume();
        	    }
        	});

	        tot.setOnDragEntered(new EventHandler<DragEvent>() {
	            public void handle(DragEvent event) {
	            	tf.getStyleClass().remove("drag-drop-ok");
	            	tf.getStyleClass().remove("drag-drop-not-ok");
	                if (event.getGestureSource() != tf && event.getDragboard().hasString()) {
	                	DataObject dragging = Main.s_instance.load(event.getDragboard().getString());
	                	if (dragging != null && dragging.getType().hasParent(refType)) {
	                		tf.getStyleClass().add("drag-drop-ok");
	                	} else {
	                		tf.getStyleClass().add("drag-drop-not-ok");
	                	}
	                }
	                event.consume();
	            }
	        });

	        tot.setOnDragDropped(new EventHandler<DragEvent>() {
	            public void handle(DragEvent event) {
	            	boolean success = false;
	                if (event.getGestureSource() != tf && event.getDragboard().hasString()) {
	                	String path = event.getDragboard().getString();
	                	DataObject dragging = Main.s_instance.load(path);
	                	if (dragging != null && dragging.getType().hasParent(refType)) {
	                        m_f.set(path);
	                        tf.textProperty().set(path);
	                        ptrbar.getChildren().setAll(tf, point, clear);
	                        tot.getChildren().setAll(ptrbar);
	                        success = true;
	                	}
	                }
	                event.setDropCompleted(success);
	                event.consume();
	             }
	        });

	        tot.setOnDragExited(new EventHandler<DragEvent>() {
	            public void handle(DragEvent event) {
	            	tf.getStyleClass().remove("drag-drop-ok");
	            	tf.getStyleClass().remove("drag-drop-not-ok");
	            	event.consume();
	            }
	        });
        }

        if (m_field.isAuxPtr)
        {
            String ref = m_f.get();
            if (ref.length() > 0)
            {
                DataObject mi = Main.s_instance.load(ref);
                if (mi != null)
                {
                    VBox aux = makeObjNode(mi);

                    Button expand = new Button("V");
                    expand.setOnAction(new EventHandler<ActionEvent>() {
                        @Override
                        public void handle(ActionEvent event) {
                            aux.setVisible(!aux.isVisible());
                            aux.setManaged(aux.isVisible());
                        }
                    });

                    // configure for this
                    ptrbar.getChildren().setAll(expand, tf, point, clear);
                    tot.getChildren().setAll(ptrbar, aux);
                    tf.getStyleClass().remove("error");
                    tf.textProperty().set(EditorCreatorUtil.makeInlineAuxTitle(mi));
                }
                else
                {
                    tf.getStyleClass().add("error");
                }
            }
        }

        if (!m_field.isAuxPtr)
        {
			ptrbar.setOnMousePressed(new EventHandler<MouseEvent>() {
				@Override
				public void handle(MouseEvent event) {
					if (event.isPrimaryButtonDown() && event.getClickCount() == 2) {
						Main.s_instance.startEditing(tf.getText());
					}
				}
			});
        }

        tot.setFillWidth(true);
        tot.setMaxWidth(Double.MAX_VALUE);
        return tot;
    }

    private VBox makeObjNode(DataObject mi)
    {
        VBox aux = new VBox();
        StructEditor se = new StructEditor(mi, "AUX", false);
        ArrayList<Node> tmp = new ArrayList<>();
        tmp.add(se.createUI());
        aux.setFillWidth(true);
        aux.getChildren().setAll(tmp);
        aux.setPadding(new Insets(4));
        aux.setStyle("-fx-border-insets: 1;");
        aux.setVisible(false);
        aux.setManaged(false);
        return aux;
    }
}

class ArrayEditor implements FieldEditor
{
    DataObject m_mi;
    ParsedField m_f;
    ArrayList<Node> m_editors;
    VBox m_box;

    public ArrayEditor(DataObject mi, ParsedField field)
    {
        m_mi = mi;
        m_f = field;
    }

    @Override
    public Node createUI()
    {
        m_box = new VBox();
        rebuild();
        return m_box;
    }

    private void rebuild()
    {
        Label hl = new Label(m_f.name + ": Array of " + m_mi.getArraySize(m_f.index) + " items(s)");
        hl.setMaxWidth(Double.MAX_VALUE);
        hl.setAlignment(Pos.BASELINE_CENTER);
        hl.setPrefHeight(30);

        Button add = new Button("+" + m_f.name);

        add.setOnAction( (evt) -> {
            int newIndex = m_mi.getArraySize(m_f.index);
            m_mi.arrayInsert(m_f.index,  newIndex);
            rebuild();
        });

        m_box.getChildren().setAll(hl, buildGridPane(), add);
        m_box.setFillWidth(true);
    }

    private Button mkRemoveBtn(int idx)
    {
        Button rm = new Button("-");
        rm.setOnAction((v) -> {
            System.out.println("Erasing at " + idx + " with array size " + m_mi.getArraySize(m_f.index));
            m_mi.arrayErase(m_f.index, idx);
            rebuild();
        });
        return rm;
    }

    private GridPane buildGridPane()
    {
        m_editors = new ArrayList<>();
        int size = m_mi.getArraySize(m_f.index);
        for (int i=0;i<size;i++)
        {
            FieldEditor fe = ObjectEditor.createEditor(m_mi, m_f, i, false);
            m_editors.add(fe.createUI());
        }

        GridPane gridpane = new GridPane();
        gridpane.setMaxWidth(Double.MAX_VALUE);

        ColumnConstraints column0 = new ColumnConstraints(-1,-1,Double.MAX_VALUE);
        ColumnConstraints column1 = new ColumnConstraints(-1,-1,Double.MAX_VALUE);

        if (((m_f.type == Compiler.FieldType.POINTER && m_f.isAuxPtr) ||
              m_f.type == Compiler.FieldType.STRUCT_INSTANCE))
            column1.setHgrow(Priority.ALWAYS);

        gridpane.getColumnConstraints().setAll(column0, column1);
        for (int i=0;i<m_editors.size();i++)
        {
            Label lbl = new Label(" " + i);
            lbl.setMaxHeight(Double.MAX_VALUE);
            lbl.setAlignment(Pos.CENTER);
            lbl.getStyleClass().add("array-index");
            if ((i&1) == 1)
                lbl.getStyleClass().add("odd");

            gridpane.add(lbl,  0,  i);
            GridPane.setValignment(lbl, VPos.TOP);
            gridpane.add(m_editors.get(i),  1,  i);

            Button rm_btn = mkRemoveBtn(i);
            rm_btn.getStyleClass().add("rm-button");
            gridpane.add(rm_btn, 2, i);
            GridPane.setValignment(rm_btn,  VPos.TOP);
        }

        return gridpane;
    }
}

class StructEditor implements FieldEditor
{
    DataObject m_mi;
    String m_name;
    boolean m_inline;

    public StructEditor(DataObject mi, String name, boolean inline)
    {
        m_mi = mi;
        m_name = name;
        m_inline = inline;
    }

    @Override
    public Node createUI()
    {
        ArrayList<Node> nodes = new ArrayList<>();

        boolean giveRect = false;

        Label header = null;

        if (m_name != null && !m_name.equals("__parent"))
        {
            header = new Label(m_name + " (" + m_mi.getType().name + ")");
            header.setMaxWidth(Double.MAX_VALUE);
            header.getStyleClass().add("struct-header");
            header.setAlignment(Pos.CENTER);
            header.setMaxHeight(Double.MAX_VALUE);

            if (!m_inline)
            {
                nodes.add(header);
            }

            giveRect = true;
        }

        ParsedStruct str = m_mi.getType();
        for (int i=0;i<str.fields.size();i++)
        {
        	ParsedField f = str.fields.get(i);
            if (f == null)
                break;
            if (!f.showInEditor)
            	continue;

            FieldEditor fe = ObjectEditor.createEditor(m_mi, f, 0, f.isArray);
            Node ed = fe.createUI();

            // array or struct or pointer or bools dont get labels.
            if (f.isArray || f.type == Compiler.FieldType.STRUCT_INSTANCE || f.type == Compiler.FieldType.BOOL)
            {
                nodes.add(ed);
            }
            else
            {
                if (f.type == Compiler.FieldType.POINTER && f.isAuxPtr)
                {
                    // aux objs get vbox
                    VBox b = new VBox();
                    b.setMaxWidth(Double.MAX_VALUE);
                    b.setFillWidth(true);
                    b.getChildren().setAll(EditorCreatorUtil.makeLabel(f, 0), ed);
                    nodes.add(b);
                }
                else
                {
                    if (m_inline)
                    {
                        Label l = new Label(f.name);
                        l.getStyleClass().add("inline-label");
                        ed.getStyleClass().add("inline-value");
                        nodes.add(l);
                        nodes.add(ed);
                    }
                    else
                    {
                        // fields get hbox
                        HBox hb = new HBox();
                        hb.setMaxWidth(Double.MAX_VALUE);
                        hb.getChildren().setAll(EditorCreatorUtil.makeLabel(f, 0), ed);
                        HBox.setHgrow(ed, Priority.ALWAYS);
                        nodes.add(hb);
                    }
                }
            }
        }

        if (m_inline)
        {
            HBox box = new HBox();
            box.getChildren().setAll(nodes);
            if (header == null)
                return box;

            HBox main = new HBox();
            main.getChildren().setAll(EditorCreatorUtil.makeInlineEditorHeader(m_name), box);
            return main;
        }

        VBox box = new VBox();
        box.setFillWidth(true);
        box.getChildren().setAll(nodes);

        if (giveRect)
        {
            box.setMaxWidth(Double.MAX_VALUE);
        }
        return box;
    }
}

public class ObjectEditor
{
    VBox m_props;
    DataObject m_mi;
    StructEditor m_root;
	private static ArrayList<FieldEditorCreator> s_cedts = new ArrayList<>();

    public ObjectEditor(DataObject mi)
    {
        m_props = new VBox();
        m_mi = mi;

        m_root = new StructEditor(mi, null, false);
        m_props.getChildren().setAll(m_root.createUI());
        m_props.setMinWidth(400);
    }

    public void constructField(ParsedField f)
    {

    }

    public Parent getRoot()
    {
        return m_props;
    }

	public static void addCustomTypeEditorCreator(FieldEditorCreator c)
	{
		s_cedts.add(c);
	}

    public static FieldEditor createEditor(DataObject mi, Compiler.ParsedField field, int index, boolean asArray)
    {
    	for (FieldEditorCreator c : s_cedts) {
    		FieldEditor res = c.createEditor(mi, field, index,  asArray);
    		if (res != null)
    			return res;
    	}

        if (asArray)
            return new ArrayEditor(mi, field);

    	for (FieldEditorCreator c : s_cedts) {
    		FieldEditor res = c.createEditor(mi, field, index, false);
    		if (res != null)
    			return res;
    	}

        switch (field.type)
        {
            case STRUCT_INSTANCE:
            {
                String name = field.name;
                if (field.isArray)
                    name += "[" + index + "]";

                DataObject _mi = (DataObject)mi.getField(field.index, index);
                String ined = _mi.getType().inlineEditor;
                if (ined != null && ined.equals("Vec4"))
                    return new StructEditor(_mi, name, true);

                return new StructEditor(_mi, name, false);
            }
            case POINTER:
                return new PointerEditor(mi, field, index);
            case INT32:
                return new IntegerEditor(mi, field, index, Integer.MIN_VALUE, Integer.MAX_VALUE);
            case UINT32:
                return new IntegerEditor(mi, field, index, 0, 0xffffffffL );
            case BYTE:
                return new IntegerEditor(mi, field, index, 0, 255);
            case BOOL:
                return new BooleanEditor(mi, field, index);
            case FILE:
                return new FileEditor(mi, field, index);
            case FLOAT:
                return new FloatEditor(mi, field, index);
            case ENUM:
                return new EnumEditor(mi, field, index);
            default:
                return new StringEditor(mi, field, index);
        }
    }
}
