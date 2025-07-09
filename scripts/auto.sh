export PATH="$PATH:$HOME/.cargo/bin"
export JAVA_HOME="/Applications/Android Studio.app/Contents/jbr/Contents/Home";
export ANDROID_HOME="$HOME/Library/Android/sdk";
export NDK_HOME="$ANDROID_HOME/ndk/$(ls -1 $ANDROID_HOME/ndk)";

# build a universal apk with support for aarch64 and armv7
function build-android-apk() {
    cargo tauri android build --apk --target aarch64 --target armv7
}

# build a macos .app that can be copied to the Applications folder
# FIXME: don't know if it works for intel macs
function build-macos-app() {
    cargo tauri build --bundles app
}
