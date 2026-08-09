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

# Find a JDK tool. Not every JDK puts all of them on PATH -- a Windows install
# commonly leaves java and javac reachable through shims while jar, which lives
# beside them in the real JDK, is not -- so fall back to JAVA_HOME and then to
# whatever directory javac itself was found in.
find_jdk_tool() {
    if command -v "$1" >/dev/null 2>&1; then
        echo "$1"
        return 0
    fi
    javac_path=$(command -v javac 2>/dev/null || echo /nonexistent)
    # Both the directory javac was found in and the one it resolves to: when javac
    # is a symlink or a shim, its siblings are not the JDK's, but its target's are.
    javac_real=$(readlink -f "$javac_path" 2>/dev/null || echo "$javac_path")
    for dir in "$JAVA_HOME/bin" "$(dirname "$javac_path")" "$(dirname "$javac_real")"; do
        for exe in "$dir/$1" "$dir/$1.exe"; do
            if [ -x "$exe" ]; then
                echo "$exe"
                return 0
            fi
        done
    done
    echo "$0: cannot find '$1'. Install a JDK (not just a JRE), or set JAVA_HOME to one." >&2
    return 1
}


if [ "$1" = "clean" ]; then
    rm -rf "$BUILD" "$DIST"
    exit 0
fi

# Skip the rebuild when the jar is newer than every source file. Sticks to
# POSIX find options so this works under git bash on Windows too.
if [ -f "$JAR" ] && [ -z "$(find src -name '*.java' -newer "$JAR")" ]; then
    exit 0
fi

# Resolved here rather than up top so `clean`, and the common case of an already
# current jar, do not need a JDK present at all.
JAVAC=$(find_jdk_tool javac)
JAR_TOOL=$(find_jdk_tool jar)

rm -rf "$BUILD"
mkdir -p "$BUILD" "$DIST"

"$JAVAC" --release 11 -nowarn -d "$BUILD" $(find src -name '*.java')
"$JAR_TOOL" --create --file "$JAR" --main-class putki.Compiler -C "$BUILD" .

echo "built $JAR"
