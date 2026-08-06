Putki project setup
===================

Here is a file listing for a minimal but still useful setup of project using
Putki.

```
Configuration files in the root:

/myproject.putked
/putki-compiler.config

Type definition files in src/types (you can change this)

/src/types/types.typedef

And all your data .jsons goes into a folder here:

/data/objs 
```

the .putked project file
------------------------

The Electron editor in `electron/` opens a `.putked` file, which is JSON and
whose directory is taken as the project root. It provides the window title:

```json
{ "title": "My super editor!" }
```

The editor builds its property editors from the type descriptors the compiler
writes for the `js` output, so the project's `putki-compiler.config` needs `js`
listed in `outputs:`.

putki-compiler.config
----------------------

```
config-version:1.0
name:CustomEdTest
genpath:_gen
outputs:rust
```

`config-version:1.0` must be the first line; it identifies the syntax version of the config file. Without
it the file is read in the legacy format, where the first two lines are taken to be the module name and
the loader name. The name is the project name, which will affect namespaces and file names for the
generated files. The genpath field is the folder name where generated code will go.

Other keys are `src:` (folder holding the `.typedef` files, default `src`), `outputs:` (space-separated
list of languages to generate: `cpp cs rust js java`; defaults to all of them), `putkipath:`, `dep:`,
`config:`, `mixki:` and `mixki-only:`. Blank lines and lines starting with `#` are ignored. Unrecognized
keys are reported as a warning and otherwise ignored.

The compiler exits with status 1 if any typedef fails to parse or resolve, or if generated code could not
be written; diagnostics go to stderr. Build scripts should check the exit code — on failure no code is
generated, so a previously generated tree will still be present and would otherwise be silently stale.

/data/objs
----------

This path is semi-hardcoded for now and the path should be relative to where
the putked file is located, or from where you want to run the data builder,
if you use it.
