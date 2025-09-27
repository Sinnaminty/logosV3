use std::{env, path::PathBuf};

fn main() {
    // Regenerate bindings if the header changes
    println!("cargo:rerun-if-changed=vendor/dectalk/include/ttsapi.h");

    // Generate Rust FFI from the DECtalk header
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings = bindgen::Builder::default()
        .header("vendor/dectalk/include/ttsapi.h") // ← change if your header name differs
        .clang_arg("-Ivendor/dectalk/include")
        .allowlist_function("TextToSpeech.*")
        .allowlist_type("TTS_.*|LANGUAGE_.*|SPEAKER_.*")
        .allowlist_var("TTS_.*")
        .generate()
        .expect("bindgen failed");
    bindings
        .write_to_file(out.join("dectalk_bindings.rs"))
        .expect("could not write bindings");

    // Tell rustc where the shared libs live, and that we link against libtts.so
    println!("cargo:rustc-link-search=native={}", "vendor/dectalk/dist");
    println!("cargo:rustc-link-lib=tts");

    // Optional (Linux): embed an rpath so the binary can find libtts.so at runtime.
    // Comment out if you prefer exporting LD_LIBRARY_PATH instead.
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN/../vendor/dectalk/dist");
}
