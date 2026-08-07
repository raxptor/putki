#!/bin/sh
# Builds dist/putki-compiler.jar. No Ant, no Gradle -- the compiler is a handful
# of source files with no dependencies outside the JDK.
#
#   ./build.sh          build the jar if sources changed
#   ./build.sh clean    remove build output
#
# Run the result with:  java -jar compiler/dist/putki-compiler.jar
set -e

cd "$(dirname "$0")"

BUILD=build
DIST=dist
JAR=$DIST/putki-compiler.jar

if [ "$1" = "clean" ]; then
    rm -rf "$BUILD" "$DIST"
    exit 0
fi

# Skip the rebuild when the jar is newer than every source file. Sticks to
# POSIX find options so this works under git bash on Windows too.
if [ -f "$JAR" ] && [ -z "$(find src -name '*.java' -newer "$JAR")" ]; then
    exit 0
fi

rm -rf "$BUILD"
mkdir -p "$BUILD" "$DIST"

javac --release 11 -nowarn -d "$BUILD" $(find src -name '*.java')
jar --create --file "$JAR" --main-class putki.Compiler -C "$BUILD" .

echo "built $JAR"
