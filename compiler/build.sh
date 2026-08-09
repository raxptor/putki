#!/bin/sh
# Compiles the putki compiler into build/classes. No Ant, no Gradle -- it is a
# handful of source files with no dependencies outside the JDK.
#
#   ./build.sh          compile if sources changed
#   ./build.sh clean    remove build output
#
# There is no jar. Packaging needs the `jar` tool, which a perfectly working JDK
# on Windows routinely leaves off PATH -- java and javac are exposed through a
# launcher directory that holds nothing else -- and compiler.sh runs the classes
# with `java -cp` regardless, so building one only added a way to fail.
set -e

cd "$(dirname "$0")"

BUILD=build
CLASSES=$BUILD/classes
STAMP=$BUILD/.compiled

if [ "$1" = "clean" ]; then
    rm -rf "$BUILD" dist
    exit 0
fi

# Skip the rebuild when the classes are newer than every source file. Sticks to
# POSIX find options so this works under git bash on Windows too.
if [ -f "$STAMP" ] && [ -z "$(find src -name '*.java' -newer "$STAMP")" ]; then
    exit 0
fi

# javac is usually on PATH; fall back to JAVA_HOME and to the JDK the running
# java reports as its own home, so a machine that can run java can build this.
find_javac() {
    if command -v javac >/dev/null 2>&1; then
        echo javac
        return 0
    fi
    runtime_home=$(java -XshowSettings:properties -version 2>&1 | sed -n 's/^ *java\.home = *//p' | head -1)
    if [ -n "$runtime_home" ] && command -v cygpath >/dev/null 2>&1; then
        runtime_home=$(cygpath -u "$runtime_home")
    fi
    for dir in "$JAVA_HOME/bin" "$runtime_home/bin"; do
        for exe in "$dir/javac" "$dir/javac.exe"; do
            if [ -x "$exe" ]; then
                echo "$exe"
                return 0
            fi
        done
    done
    return 1
}

JAVAC=$(find_javac) || {
    echo "$0: cannot find 'javac'. Install a JDK (not just a JRE), or set JAVA_HOME to one." >&2
    exit 1
}

rm -rf "$CLASSES"
mkdir -p "$CLASSES"
"$JAVAC" --release 11 -nowarn -d "$CLASSES" $(find src -name '*.java')
touch "$STAMP"

echo "built $CLASSES"
