// A minimal build file that picks up libagc when it exists
//
// Run with
//
//   cargo build -vv

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mut binding = cc::Build::new();
    let mut cfg = binding.cpp(true);
    if libagc_installed(&mut cfg) {
        println!("cargo::warning=\"Linking against system libagc!\"");
        println!("cargo::rustc-link-lib=agc");
        return;
    }

    println!("cargo::error=\"Should not get here\"");
}

fn libagc_installed(cfg: &mut cc::Build) -> bool {
    let guixenv =
        match env::var("GUIX_ENVIRONMENT") {
            Ok(val) => val,
            Err(_e) => "".to_string(),
        };
    let mut cmd = cfg.get_compiler().to_command();
    println!("cargo::warning=\"Looking for system libagc\"");
    cmd.arg("test/smoke.cpp")
        // .arg("-Wno-unused-parameter")
        .arg("-g0")
        .arg("-o")
        .arg("/dev/null")
        .arg(guixenv.to_string()+"/lib/mimalloc-2.1/libmimalloc.a")
        .arg("-lzstd")
        .arg(guixenv.to_string()+"/lib/libagc.a");

    println!("cargo::warning=\"running {:?}\"", cmd);
    if let Ok(status) = cmd.status() {
        if status.success() {
            return true;
        }
    }
    println!("cargo::warning=\"linking libagc.a failed!\"");
    false
}
