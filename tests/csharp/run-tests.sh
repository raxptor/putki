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
