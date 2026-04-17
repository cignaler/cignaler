fn main() {
    // When pre-building cignaler-native-host via beforeBuildCommand, skip the Tauri
    // build setup to avoid the circular dependency: tauri_build checks for the
    // externalBin binary before it has been built.
    #[cfg(not(feature = "skip-tauri-build"))]
    tauri_build::build()
}
