#!/usr/bin/env sh

if [ -n "$JAVA_HOME" ] ; then
    if [ -x "$JAVA_HOME/jre/sh/javac" ] ; then
        JAVAC_CMD="$JAVA_HOME/jre/sh/javac"
        JAR_CMD="$JAVA_HOME/jre/sh/jar"
    else
        JAVAC_CMD="$JAVA_HOME/bin/javac"
        JAR_CMD="$JAVA_HOME/bin/jar"
    fi
    if [ ! -x "$JAVAC_CMD" ] ; then
        die "ERROR: JAVA_HOME is set to an invalid directory: $JAVA_HOME

Please set the JAVA_HOME variable in your environment to match the
location of your Java installation."
    fi
else
    JAVAC_CMD="javac"
    JAR_CMD="jar"
    which javac >/dev/null 2>&1 || die "ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH.

Please set the JAVA_HOME variable in your environment to match the
location of your Java installation."
fi

cargo build --release --target=wasm32-unknown-unknown -p cdylib
if [ -x "$(command -v wasm-gc)" ]; then
    echo "Using wasm-gc"
    wasm-gc ../../target/wasm32-unknown-unknown/release/livesplit_core.wasm
fi
rm -rf src
mkdir src
(cd src && mkdir org && cd org && mkdir livesplit)
rm -rf classes
mkdir classes
(cd classes && mkdir org && cd org && mkdir livesplit && cd livesplit && ../../../asmble/bin/asmble compile ../../../../../target/wasm32-unknown-unknown/release/livesplit_core.wasm org.livesplit.LiveSplitCore)
(cd ../bind_gen && cargo run)
cp ../bindings/java/wasm/*.java src/org/livesplit
"$JAVAC_CMD" -g:source,lines,vars -classpath classes src/org/livesplit/*.java
mv src/org/livesplit/*.class classes/org/livesplit
"$JAR_CMD" cvf livesplit-core.jar -C classes .

# JAVA_HOME="C:\Program Files\Java\jdk1.8.0_144" ./make.sh
# "C:\Program Files\Java\jdk1.8.0_144\bin\javap" -p -c classes.org.livesplit.LiveSplitCore > new-code2.asm
