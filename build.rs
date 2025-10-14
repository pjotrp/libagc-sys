// A minimal build file that picks up libagc when it exists
//
// Run with
//
//   cargo build -vv

use std::env;
use std::path::PathBuf;
use std::process::Command;

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

    // Approach 1: Check if AGC_LIB_DIR is set (user-provided library)
    if let Ok(lib_dir) = env::var("AGC_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=dylib=agc");
        println!("cargo:rerun-if-env-changed=AGC_LIB_DIR");
        return;
    }

    // Approach 2: Check if AGC library is in system library paths
    if library_exists_in_system() {
        println!("cargo:rustc-link-lib=dylib=agc");
        println!("cargo:warning=Using system AGC library");
        return;
    }

    // Approach 3: Build AGC from source if available
    // if let Ok(agc_source) = env::var("AGC_SOURCE_DIR") {
    //    return;
    // }

    // Approach 4: Try to find AGC in common locations
    let common_paths = vec![
        "/usr/lib",
        "/usr/local/lib",
        "/lib",
        "/opt/homebrew/lib",  // macOS with Homebrew
        "C:\\Program Files\\agc\\lib",  // Windows
    ];

    for path in common_paths {
        let lib_path = PathBuf::from(path);
        if lib_path.exists() && check_lib_in_path(&lib_path) {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=dylib=agc");
            println!("cargo:warning=Found AGC library in {}", path);
            return;
        }
    }

    // If we get here, we couldn't find or build the library
    panic!(
        "AGC library not found. Please either:\n\
         1. Install AGC system-wide, or\n\
         2. Set AGC_LIB_DIR to the directory containing libagc.so/dylib/dll, or\n\
         3. Set AGC_SOURCE_DIR to the AGC source directory to build from source\n\
         \n\
         Example: AGC_LIB_DIR=/path/to/agc/lib cargo build\n\
         Example: AGC_SOURCE_DIR=/path/to/agc/source cargo build"
    );
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


/// Check if AGC library exists in system library paths
fn library_exists_in_system() -> bool {
    // Try to compile a simple test program that links against agc
    let test_code = r#"
        extern "C" {
            fn agc_open(fn_: *mut i8, prefetching: i32) -> *mut std::ffi::c_void;
        }
        fn main() {}
    "#;

    let tmp_dir = env::temp_dir();
    let test_file = tmp_dir.join("agc_link_test.rs");

    if std::fs::write(&test_file, test_code).is_err() {
        return false;
    }

    let output = Command::new("rustc")
        .arg(&test_file)
        .arg("-l")
        .arg("agc")
        .arg("--crate-type")
        .arg("bin")
        .arg("-o")
        .arg(tmp_dir.join("agc_link_test"))
        .output();

    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(tmp_dir.join("agc_link_test"));

    output.map(|o| o.status.success()).unwrap_or(false)
}

/// Check if AGC library exists in a specific path
fn check_lib_in_path(path: &PathBuf) -> bool {
    let extensions = if cfg!(target_os = "windows") {
        vec!["dll", "lib"]
    } else if cfg!(target_os = "macos") {
        vec!["dylib", "a"]
    } else {
        vec!["so", "a"]
    };

    for ext in extensions {
        let lib_name = format!("libagc.{}", ext);
        let agc_name = format!("agc.{}", ext);

        if path.join(&lib_name).exists() || path.join(&agc_name).exists() {
            return true;
        }
    }

    false
}
