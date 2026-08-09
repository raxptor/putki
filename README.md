Putki
=====

![Putki](misc/putkico-5.png)

Putki - Generic data system with C++, C# and Rust support, with an Electron based data editor.

Running the compiler
--------------------

The compiler is plain Java with no dependencies beyond the JDK, so it needs no build system. From a
parent project that has putki checked out as a submodule, one script builds it and runs it:

```
ext/putki/compiler.sh              # generate code for the project in the current directory
ext/putki/compiler.sh some/dir     # ...for the project rooted at some/dir
```

`putki-compiler.config` is resolved relative to that directory, so with no argument it means "wherever
I ran this from". Building is a fresh `javac` of ~3k lines -- about a second -- and is skipped entirely
when the classes are already newer than the sources, so calling this unconditionally costs nothing.

To compile without running, use `./compiler/build.sh` (`clean` as an argument removes the output). It
produces `compiler/build/classes`, which `compiler.sh` runs with `java -cp`. There is deliberately no
jar: packaging needs the `jar` tool, which a working JDK on Windows often leaves off PATH, and nothing
here needs the compiler to be a single file.

Any JDK 11 or later works, on Linux, macOS, and Windows under git bash. To skip the build step entirely
while hacking on the compiler, JDK 22+ can run the sources directly, compiling them in memory:

```
java <path-to-putki>/compiler/src/putki/Compiler.java
```

Tests
-----

```
tests/csharp/run-tests.sh                              # needs mono (mcs)
compiler.sh tests/rust && (cd tests/rust && cargo test)  # needs cargo
```

The C# suite checks the runtime's string escaping and binary package reader against the vectors in
`doc/text-format.md` and `doc/package-format.md`, then two generator tests:

- **tests/simple** -- compiles the generated C# against the runtime and loads `data/objs` through it.
  Covers inheritance, arrays, enums, nested structs and build configs, so it catches codegen that
  fails to build or fails to read real data.
- **tests/rust** -- reads `tests/fixtures/polymorphic.pkg` through the generated C# loader. That
  fixture is written by the Rust pipeline, so the rtti type tag and its dispatch are checked across
  both implementations. The fixture is checked in, so this step needs no cargo.

`tests/rust` is also a cargo crate: `tests/rust/tests/roundtrip.rs` takes `data/main.txt` through the
inki build, out as a binary package, and back in through the outki reader, asserting that polymorphic
pointers keep their type. It declares its own `[workspace]`, so cargo works even when putki is
vendored inside another project's workspace. Regenerate the fixture after a format change with:

```
cd tests/rust && cargo run --example write-fixture -- ../fixtures/polymorphic.pkg
```

Both test projects need `compiler.sh` run over them first -- the generated code is not checked in.

`tests/simple/src/Program.cs` takes the project directory as its argument and optionally a package
file as a second one; the package half needs a data builder, which this project does not run.

Localization
------------

Mark a string translatable in the typedef by putting a catalog category where the type starts, with a
trailing `+` for a string that has a plural form:

```
Character
{
	{Character} string Name
	{Item}+ string CarriedCount
}
```

The Rust generator then emits, per marked field, an accessor that takes a catalog and a collector that
finds the string during extraction:

```rust
let shown = character.name(translation);          // Name(Putki.Translation) in C#
let counted = item.carried_count(translation, n);
```

Extraction walks the data through those generated collectors -- there is no reflection, and a newly
marked field cannot be missed:

```rust
let catalog = putki_inki::Catalog::new();   // see tests/rust/src/lib.rs::extract_strings
std::fs::write("out.pot", catalog.to_pot("MyProject"))?;
```

Strings are mangled before they reach the catalog: a `{#span}` written by hand, and any bare `NN%`,
are replaced with fixed placeholders, so `Deal 20% damage` and `Deal 30% damage` collapse to the one
entry `Deal {12}% damage` and rebalancing a number never invalidates a translation. The Rust and C#
implementations of that must agree byte for byte -- one writes the msgid, the other looks it up --
which `tests/fixtures/mangling-vectors.txt` enforces from both sides.

Types
-----

You define your own data structures and store your input data in JSON, then get them packaged in binary for very efficient lodaing. Here is one example:

```
GlobalSettings
{
	string WindowTitle
	u32 WindowWidth
	u32 WindowHeight
	ptr Texture Icon
	ptr ShaderProgram ShaderSolid
	ptr ShaderProgram Shadertexture
}
```

The compiler then compiles this into automatically generated (c++) code for the following:

- Parsing the structure from JSON files
- Writing the structure into binary format for different machine configuratinos
- Loading the structure from the same binary format
- Code for traversing the structure (following pointers)

Builder
-------

When using this system you will typically arrange your data like this

```
data/obj/<JSON files here>
data/res/<Any other resources>
```

Then you run the data builder on this, where you specify what assets you want, and what the resulting packages will be. 

```
putki::package::data *pkg = putki::package::create(out);
putki::package::add(pkg, "ui/mainmenu/screen", true);
putki::build::commit_package(pkg, pconf, "mainmenu.pkg");
```

This then grabs ui/mainmenu/screen(.json), pulls in all other references objects and writes them into a binary package.

All this happens during the build step (where you can also do any processing you want to transform the data, including adding new output
objects etc).

Runtime
-------

When your application wants to load the data, it loads in the putki runtime library, which is quite tiny and efficient, but can load these packages straight into memory.
The process for loading is to load the file (minus header) into memory, resolve pointers and then everything is ready to go.

```
// package load
putki::pkgmgr::loaded_package *pkg = putki::pkgloader::from_file("mainmenu.pkg");

// grab pointer to the main menu (that was json object)
outki::ui_screen *menu = putki::pkgmgr::resolve(pkg, "ui/mainmenu/screen");
```
In the runtime, all the strings are no longer std::strings as in the build step, and the arrays are no std::vectors<>.

From the definition

```
ExampleStruct
{
   string Txt
   byte[] SomeData
}
```

would then be generated

```
struct example_struct
{
   const char *txt
   unsigned char *some_data;
   unsigned int some_data_size;
}
```

All those data bytes and the strings are pointers into the loaded package file and need no dynamic allocation. When loading a packag it will include all the data already.

Patches
-------

The system also supports making incremental builds and writing patch packages, that reference already existing packages but add anything that was modified and added. 

Editor and live editing
-----------------------

Putki comes with an Electron based editor (in `electron/`) which lets you edit your assets. It reads a
`.putked` project file and builds its property editors from the type descriptors the compiler emits for
the `js` output, so add `js` to `outputs:` in your `putki-compiler.config` to use it.

The runtime also supports live updates, so that you can get instant feedback in your application when you
make changes (although your application needs to be aware of what goes on).

Of course, if you have build steps on your objects, these are performed onto the edited assets. So you can sit in the editor and tweak build step parameters and
enjoy direct feedback in your application.

This functionality is enabled or disabled with the preprocessor, so it can be stripped out from your final builds.

```
if (LIVE_UPDATE(&object_pointer))
{
	// object has been updated! pointer has now changed and
	// we might want to do stuff here to handle getting new
	// data.
}

if (LIVE_UPDATE_ISNULL(object_pointer->sub_thing))
{
	// here we check for null conditions that we are only
	// ever interested in handling while doing live updates
	return;
}
```

Building without live updates enabled turns those expressions into permanent false.


C# support
----------

Putki also comes with code generation for C# and a small runtime library, so you can load
the binary packages into your C# applications as well.
