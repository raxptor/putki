Putki project setup
===================

Here is a file listing for a minimal but still useful setup of project using
Putki.

```
Configuration files in the root:

/putked.conf
/putki-compiler.conf

Type definition files in src/types (you can change this)

/src/types/types.typedef

And all your data .jsons goes into a folder here:

/data/objs 
```

putked.conf
-----------

Below is an example of putked.conf. You can add plugin loading there when
you get advanced, but for now it provides the title, and lets the editor
know where your project root is (same place as the file).

```
title=My super editor!
```

putki-compiler.conf
-------------------

```
version:1.0
name:CustomEdTest
genpath:_gen
```

The version is identifies syntax verison of the config file.  The name is the project name, which will affect
namespaces and file names for the generated files. The genpath field is the folder name
where generated code will go.

/data/objs
----------

This path is semi-hardcoded for now and the path should be relative to where
the putked file is located, or from where you want to run the data builder,
if you use it.
