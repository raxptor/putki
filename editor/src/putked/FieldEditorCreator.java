package putked;

import putki.Compiler;

public interface FieldEditorCreator
{
	// Can return null for arrays.
	FieldEditor createEditor(DataObject mi, Compiler.ParsedField field, int index, boolean asArray);
}
