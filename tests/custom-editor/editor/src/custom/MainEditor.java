package custom;

import java.util.Random;

import javafx.geometry.Insets;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ScrollPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.Priority;
import javafx.scene.layout.VBox;
import putked.DataHelper;
import putked.DataObject;
import putked.Editor;
import putked.Main;
import putki.Compiler;
import putked.inki.*;

public class MainEditor implements Editor
{
	public String getName()
	{
		return "MainEditor";
	}

	public int getPriority()
	{
		return -1;
	}

	public boolean canEdit(Compiler.ParsedStruct type)
	{
		return type.name.equals("Main") && type.moduleName.equals("CustomEdTest");
	}

	private Random r = new Random();

	public Node createUI(DataObject obj)
	{
		HBox desc = new HBox();
		Label path = new Label("[" + obj.getPath() + "]");

		CustomEdTest.Main mobj = (CustomEdTest.Main) DataHelper.createPutkEdObj(obj);
		if (mobj == null)
			return null;

		Label k = new Label("Label is [" + mobj.getName() + "] and digit is [" + mobj.getDigit() + "]");

		Button save = new Button("MakeRandomAndSave");
		save.setOnAction( (actionEvent) -> {
			mobj.setName("Random!");
			mobj.setDigit(r.nextInt());
			k.setText("Label is [" + mobj.getName() + "] and digit is [" + mobj.getDigit() + "]");
			Main.s_dataWriter.WriteObject(obj);
			actionEvent.consume();
		});

		desc.getChildren().setAll(path, save);
		desc.getStyleClass().add("proped-header");


		ScrollPane sp = new ScrollPane(k);
		sp.setPrefHeight(-1);
		VBox.setVgrow(sp,  Priority.ALWAYS);
		VBox.setVgrow(desc, Priority.NEVER);

		VBox box = new VBox();
		box.getChildren().setAll(desc, sp);
		return box;
	}
}
