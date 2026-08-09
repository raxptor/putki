#!/bin/sh
# Builds the putki compiler if needed, then runs it. This is the entry point
# parent projects should call:
#
#   ext/putki/compiler.sh              # generate code for the project in the current dir
#   ext/putki/compiler.sh some/dir     # ...for the project rooted at some/dir
#
# The compiler resolves putki-compiler.config relative to that directory, so
# running with no argument means "the directory I invoked this from".
#
# Building is a fresh javac of ~3k lines (about a second) and is skipped
# entirely when the jar is already up to date, so calling this unconditionally
# is cheap.
set -e

PUTKI=$(dirname "$0")

# build.sh cd's to its own directory; run it as a subprocess so our caller's
# working directory -- which is what the compiler generates against -- survives.
"$PUTKI/compiler/build.sh"

# Run from the class files rather than the jar: packaging needs the `jar` tool,
# which a working JDK on Windows often does not put on PATH, and nothing here
# needs the compiler to be a single file.
exec java -cp "$PUTKI/compiler/build/classes" putki.Compiler "$@"
