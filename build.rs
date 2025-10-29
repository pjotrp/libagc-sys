use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Get the output directory
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=build.rs");

    // AGC is written in C++, so we need to link the C++ standard library
    // This applies to both the main library and test binaries
    link_cpp_stdlib();
    find_and_link_static_libstdcpp();

    // Link AGC's dependencies (zstd, etc.)
    link_agc_dependencies();

    // Also ensure test binaries get the same link flags
    ensure_test_linking();

    // Determine the AGC library location
    // Try multiple approaches to find/build the AGC library

    // Approach 1: Check if AGC_LIB_DIR is set (user-provided library)
    if let Ok(lib_dir) = env::var("AGC_LIB_DIR") {
        println!("cargo:rustc-link-search=native={}", lib_dir);
        println!("cargo:rustc-link-lib=agc");
        println!("cargo:rerun-if-env-changed=AGC_LIB_DIR");
        return;
    }

    // Approach 2: Check if AGC library is in system library paths
    if library_exists_in_system() {
        println!("cargo:rustc-link-lib=agc");
        println!("cargo:warning=Using system AGC library");
        return;
    }

    // Approach 3: Build AGC from source if available
    if let Ok(agc_source) = env::var("AGC_SOURCE_DIR") {
        build_agc_from_source(&agc_source, &out_dir);
        return;
    }

    // Approach 4: Try to find AGC in common locations
    let common_paths = vec![
        "/usr/lib",
        "/usr/local/lib",
        "/opt/homebrew/lib",  // macOS with Homebrew
        "/opt/local/lib",     // MacPorts
        "C:\\Program Files\\agc\\lib",  // Windows
    ];

    for path in common_paths {
        let lib_path = PathBuf::from(path);
        if lib_path.exists() && check_lib_in_path(&lib_path) {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=agc");
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
         Example: AGC_SOURCE_DIR=/path/to/agc/source cargo build\n\
         \n\
         Note: AGC is a C++ library, ensure you have a C++ compiler and standard library installed."
    );
}

fn find_and_link_static_libstdcpp() {
    // Method 1: Use g++ to find libstdc++.a
    if let Ok(output) = Command::new("g++")
        .arg("-print-file-name=libstdc++.a")
        .output()
    {
        if output.status.success() {
            if let Ok(path_str) = String::from_utf8(output.stdout) {
                let path_str = path_str.trim();

                if !path_str.is_empty() && path_str != "libstdc++.a" {
                    // Found the actual path
                    if let Some(parent) = PathBuf::from(path_str).parent() {
                        println!("cargo:rustc-link-search=native={}", parent.display());
                        println!("cargo:rustc-link-lib=static=stdc++");
                        println!("cargo:warning=Using static libstdc++ from: {}", path_str);
                        return;
                    }
                }
            }
        }
    }

    println!("cargo:rustc-link-search=native={}", "/gnu/store/x82y1af67l0kk6z95rk0m7pf216drh29-profile/lib");
}

/// Link AGC's dependencies (compression libraries, etc.)
fn link_agc_dependencies() {
    let target = env::var("TARGET").unwrap();

    // AGC uses zstd for compression
    println!("cargo:rustc-link-lib=zstd");

    // AGC may also use other compression libraries
    // Try linking them, but don't fail if they're not available
    // as they might be statically linked into libagc

    // Check if we can find zstd with pkg-config
    if Command::new("pkg-config")
        .args(&["--exists", "libzstd"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        if let Ok(output) = Command::new("pkg-config")
            .args(&["--libs", "libzstd"])
            .output()
        {
            if output.status.success() {
                let libs = String::from_utf8_lossy(&output.stdout);
                println!("cargo:warning=Found zstd via pkg-config: {}", libs.trim());
            }
        }
    } else {
        println!("cargo:warning=zstd library will be linked (ensure libzstd is installed)");
    }

    // On some systems, AGC might need additional libraries
    if target.contains("linux") {
        // pthread is often needed
        println!("cargo:rustc-link-lib=pthread");
    }

    println!("cargo:warning=Linking AGC dependencies: zstd");
}

/// Link the C++ standard library based on platform and compiler
fn link_cpp_stdlib() {
    let target = env::var("TARGET").unwrap();

    if target.contains("apple") || target.contains("darwin") {
        // macOS - link libc++
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") {
        // Linux - link libstdc++ (most common)
        // Using static linking for C++ stdlib to avoid runtime issues?
        // println!("cargo:rustc-link-lib=static=stdc++");
        println!("cargo:rustc-link-lib=stdc++");

        // Also link gcc_s for exception handling
        // println!("cargo:rustc-link-lib=gcc_s");
    } else if target.contains("windows") {
        // Windows with MSVC
        if target.contains("msvc") {
            // MSVC - link the C++ runtime
            println!("cargo:rustc-link-lib=msvcprt");
        } else {
            // MinGW - link libstdc++ and required dependencies
            println!("cargo:rustc-link-lib=stdc++");
            println!("cargo:rustc-link-lib=gcc_s");
            println!("cargo:rustc-link-lib=gcc");
        }
    } else if target.contains("freebsd") {
        println!("cargo:rustc-link-lib=c++");
    } else {
        // Default fallback for other Unix-like systems
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Print diagnostic information
    println!("cargo:warning=Linking C++ standard library for target: {}", target);
}

/// Ensure test binaries also get proper C++ linking
fn ensure_test_linking() {
    let target = env::var("TARGET").unwrap();

    // Set rustc-link-lib for all build types including tests
    if target.contains("apple") || target.contains("darwin") {
        println!("cargo:rustc-cdylib-link-arg=-lc++");
        println!("cargo:rustc-cdylib-link-arg=-lzstd");
    } else if target.contains("linux") {
        println!("cargo:rustc-cdylib-link-arg=-lstdc++");
        println!("cargo:rustc-cdylib-link-arg=-lgcc_s");
        println!("cargo:rustc-cdylib-link-arg=-lzstd");
        println!("cargo:rustc-cdylib-link-arg=-lpthread");
    } else if target.contains("windows") && !target.contains("msvc") {
        println!("cargo:rustc-cdylib-link-arg=-lstdc++");
        println!("cargo:rustc-cdylib-link-arg=-lzstd");
    }
}

/// Check if AGC library exists in system library paths
fn library_exists_in_system() -> bool {
    // Create a test C++ file that uses the AGC C API
    let test_code = r#"
        extern "C" {
            void* agc_open(char* fn, int prefetching);
        }
        int main() {
            return 0;
        }
    "#;

    let tmp_dir = env::temp_dir();
    let test_file = tmp_dir.join("agc_link_test.cpp");
    let test_bin = tmp_dir.join("agc_link_test");

    if std::fs::write(&test_file, test_code).is_err() {
        return false;
    }

    // Try to compile and link
    let compilers = vec!["g++", "clang++", "c++"];

    for compiler in compilers {
        let output = Command::new(compiler)
            .arg(&test_file)
            .arg("-lagc")
            .arg("-o")
            .arg(&test_bin)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let _ = std::fs::remove_file(&test_file);
                let _ = std::fs::remove_file(&test_bin);
                return true;
            }
        }
    }

    let _ = std::fs::remove_file(&test_file);
    false
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

/// Build AGC library from source
fn build_agc_from_source(source_dir: &str, out_dir: &PathBuf) {
    let source_path = PathBuf::from(source_dir);

    if !source_path.exists() {
        panic!("AGC source directory does not exist: {}", source_dir);
    }

    println!("cargo:warning=Building AGC from source at {}", source_dir);
    println!("cargo:rerun-if-changed={}", source_dir);

    // Create build directory
    let build_dir = out_dir.join("agc_build");
    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    // Build using make if Makefile exists
    let makefile = source_path.join("Makefile");
    if makefile.exists() {
        build_with_make(&source_path, &build_dir, out_dir);
        return;
    }

    // Build using CMake if CMakeLists.txt exists
    let cmake_file = source_path.join("CMakeLists.txt");
    if cmake_file.exists() {
        build_with_cmake(&source_path, &build_dir, out_dir);
        return;
    }

    // Try to build using cc crate for simple C/C++ projects
    build_with_cc(&source_path, out_dir);
}

/// Build AGC using Make
fn build_with_make(source_path: &PathBuf, _build_dir: &PathBuf, out_dir: &PathBuf) {
    println!("cargo:warning=Building AGC with Make");

    // Check if we need to run make clean first
    let _ = Command::new("make")
        .current_dir(source_path)
        .arg("clean")
        .status();

    let status = Command::new("make")
        .current_dir(source_path)
        .arg("-j")
        .arg(num_cpus::get().to_string())
        .env("CXX", env::var("CXX").unwrap_or_else(|_| "g++".to_string()))
        .status()
        .expect("Failed to run make");

    if !status.success() {
        panic!("Make build failed");
    }

    // Find and copy the built library
    let lib_patterns = if cfg!(target_os = "windows") {
        vec!["agc.dll", "libagc.dll", "agc.lib", "libagc.lib"]
    } else if cfg!(target_os = "macos") {
        vec!["libagc.dylib", "libagc.a"]
    } else {
        vec!["libagc.so", "libagc.a", "libagc.so.1"]
    };

    for entry in walkdir::WalkDir::new(source_path)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_name = entry.file_name().to_string_lossy().to_string();
        for pattern in &lib_patterns {
            if file_name == *pattern || file_name.starts_with("libagc.so") {
                let dest = out_dir.join(pattern);
                std::fs::copy(entry.path(), &dest)
                    .expect(&format!("Failed to copy library to {}", dest.display()));

                println!("cargo:rustc-link-search=native={}", out_dir.display());
                println!("cargo:rustc-link-lib=agc");
                println!("cargo:warning=Built and copied AGC library to {}", dest.display());
                return;
            }
        }
    }

    panic!("Could not find built AGC library after make");
}

/// Build AGC using CMake
fn build_with_cmake(source_path: &PathBuf, build_dir: &PathBuf, out_dir: &PathBuf) {
    println!("cargo:warning=Building AGC with CMake");

    // Configure
    let mut cmake_config = Command::new("cmake");
    cmake_config
        .current_dir(&build_dir)
        .arg(source_path)
        .arg(format!("-DCMAKE_INSTALL_PREFIX={}", out_dir.display()))
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg("-DBUILD_SHARED_LIBS=ON");

    // Set C++ compiler if specified
    if let Ok(cxx) = env::var("CXX") {
        cmake_config.arg(format!("-DCMAKE_CXX_COMPILER={}", cxx));
    }

    let status = cmake_config.status().expect("Failed to run cmake configure");
    if !status.success() {
        panic!("CMake configuration failed");
    }

    // Build
    let status = Command::new("cmake")
        .current_dir(&build_dir)
        .arg("--build")
        .arg(".")
        .arg("--config")
        .arg("Release")
        .arg("--parallel")
        .arg(num_cpus::get().to_string())
        .status()
        .expect("Failed to run cmake build");

    if !status.success() {
        panic!("CMake build failed");
    }

    // Install
    let status = Command::new("cmake")
        .current_dir(&build_dir)
        .arg("--install")
        .arg(".")
        .status()
        .expect("Failed to run cmake install");

    if !status.success() {
        panic!("CMake install failed");
    }

    println!("cargo:rustc-link-search=native={}/lib", out_dir.display());
    println!("cargo:rustc-link-lib=agc");
}

/// Build AGC using cc crate (for simple projects without build system)
fn build_with_cc(source_path: &PathBuf, _out_dir: &PathBuf) {
    println!("cargo:warning=Building AGC with cc crate");

    let mut build = cc::Build::new();

    // Set C++ compiler
    build.cpp(true);
    build.flag_if_supported("-std=c++17");
    build.flag_if_supported("/std:c++17"); // MSVC

    // Find all C++ source files
    let extensions = vec!["cpp", "cc", "cxx", "c++"];
    let mut found_files = false;

    for entry in walkdir::WalkDir::new(source_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(ext) = entry.path().extension() {
            if extensions.contains(&ext.to_string_lossy().as_ref()) {
                let file_name = entry.file_name().to_string_lossy().to_string();

                // Skip test files, main files, and example files
                if !file_name.contains("test")
                    && !file_name.contains("main")
                    && !file_name.contains("example")
                    && !entry.path().to_string_lossy().contains("/test")
                    && !entry.path().to_string_lossy().contains("/examples")
                {
                    build.file(entry.path());
                    found_files = true;
                }
            }
        }
    }

    if !found_files {
        panic!("No C++ source files found in {}", source_path.display());
    }

    // Add include directories
    build.include(source_path);

    // Common include subdirectories
    for subdir in &["include", "src", "lib"] {
        let inc_path = source_path.join(subdir);
        if inc_path.exists() {
            build.include(&inc_path);
        }
    }

    // Optimization flags
    build.opt_level(3);
    build.flag_if_supported("-O3");
    build.flag_if_supported("-march=native");

    // Warning flags
    build.warnings(false); // Disable warnings from AGC source

    // Compile
    build.compile("agc");

    println!("cargo:rustc-link-lib=static=agc");
}
