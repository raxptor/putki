#!/bin/sh
# Compiles and runs the C# conformance tests against the shared fixtures.
# Needs mono (mcs). From this directory: ./run-tests.sh
set -e
cd "$(dirname "$0")"
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT

echo "== string escaping (doc/text-format.md) =="
mcs -out:"$OUT/escapes.exe" ../../runtime/csharp/MicroJson.cs EscapeVectors.cs
mono "$OUT/escapes.exe"

echo "== binary package (doc/package-format.md) =="
mcs -out:"$OUT/package.exe" ../../runtime/csharp/Package.cs PackageVectors.cs
mono "$OUT/package.exe" ../fixtures/sample.pkg

# Generator check: the C# the compiler emits has to compile against the runtime.
# tests/simple's typedefs cover inheritance, arrays, enums, nested structs and
# build configs, so this catches generator regressions that produce code which
# does not build -- which unit tests over hand-written vectors miss.
echo "== generated C# loads source data (tests/simple typedefs) =="
../../compiler.sh ../simple > "$OUT/codegen.log" || { cat "$OUT/codegen.log"; exit 1; }
mcs -out:"$OUT/simple.exe" \
    -recurse:'../../runtime/csharp/*.cs' -recurse:'../simple/_gen/csharp/*.cs' \
    ../simple/src/Program.cs
# Run from the project root so the loader finds data/objs.
(cd ../simple && mono "$OUT/simple.exe" .)

# Cross-language check of the rtti path: the fixture is written by the Rust
# pipeline (tests/rust, see its README note) and read here through the generated
# C# loader. Needs no cargo -- the fixture is checked in. Regenerate it with
# `cargo run --example write-fixture -- ../fixtures/polymorphic.pkg` from tests/rust.
echo "== polymorphic package, Rust-written / C#-read (tests/rust typedefs) =="
../../compiler.sh ../rust > "$OUT/codegen-rust.log" || { cat "$OUT/codegen-rust.log"; exit 1; }
mcs -out:"$OUT/poly.exe" \
    -recurse:'../../runtime/csharp/*.cs' -recurse:'../rust/_gen/csharp/*.cs' \
    PolymorphicVectors.cs
mono "$OUT/poly.exe" ../fixtures/polymorphic.pkg
