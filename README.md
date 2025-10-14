# libagc-sys crate: AGC Rust Bindings

Rust FFI bindings for the AGC (Assembled Genomes Compressor) C library.

## Overview

AGC is a library for compressed storage and efficient random access to assembled genomes. These Rust bindings provide a safe, idiomatic interface to the AGC C API.

Ensure the AGC library is available for linking. One of:

- Provide a path to the static libagc.a
- The AGC shared library must be in your library path
- Or use a build script to compile and link the C library

## Quick Start

```rust
use agc_bindings::AgcFile;

fn main() -> Result<(), String> {
    // Open AGC file with prefetching enabled
    let agc = AgcFile::open("genome_data.agc", true)?;

    // List all samples
    let samples = agc.list_sample()?;
    println!("Found {} samples", samples.len());

    // Get contigs for the first sample
    let contigs = agc.list_ctg(Some(&samples[0]))?;

    // Get contig length
    let length = agc.get_ctg_len(Some(&samples[0]), &contigs[0])?;
    println!("Contig {} has length: {}", contigs[0], length);

    // Retrieve sequence data (first 1000 bases)
    let sequence = agc.get_ctg_seq(Some(&samples[0]), &contigs[0], 0, 1000)?;
    println!("Sequence: {}", sequence);

    Ok(())
}
```

## API Reference

### `AgcFile`

The main interface for interacting with AGC files.

#### Opening Files

##### `AgcFile::open(filename: &str, prefetching: bool) -> Result<Self, String>`

Opens an AGC file for reading.

**Parameters:**
- `filename` - Path to the AGC file
- `prefetching` - If `true`, preloads the entire file into memory for faster queries. Use `true` for multiple sequence queries, `false` for single queries or large files.

**Returns:**
- `Ok(AgcFile)` on success
- `Err(String)` with error message on failure

**Example:**
```rust
// Open with prefetching for multiple queries
let agc = AgcFile::open("data.agc", true)?;

// Open without prefetching for large files
let agc_large = AgcFile::open("large_data.agc", false)?;
```

**Test Coverage:** `test_open_and_close`, `test_open_nonexistent_file`, `test_prefetching_modes`

---

#### Sample Operations

##### `fn n_sample(&self) -> i32`

Returns the total number of samples in the AGC file.

**Returns:**
- Number of samples

**Example:**
```rust
let agc = AgcFile::open("data.agc", true)?;
let count = agc.n_sample();
println!("File contains {} samples", count);
```

**Test Coverage:** `test_n_sample`

---

##### `fn list_sample(&self) -> Result<Vec<String>, String>`

Lists all sample names in the AGC file.

**Returns:**
- `Ok(Vec<String>)` containing sample names
- `Err(String)` on failure

**Example:**
```rust
let agc = AgcFile::open("data.agc", true)?;
let samples = agc.list_sample()?;
for sample in samples {
    println!("Sample: {}", sample);
}
```

**Test Coverage:** `test_list_samples`, `test_full_workflow`

---

##### `fn reference_sample(&self) -> Result<String, String>`

Gets the name of the reference sample.

**Returns:**
- `Ok(String)` containing the reference sample name
- `Err(String)` on failure

**Example:**
```rust
let agc = AgcFile::open("data.agc", true)?;
let reference = agc.reference_sample()?;
println!("Reference sample: {}", reference);
```

**Test Coverage:** `test_reference_sample`

---

#### Contig Operations

##### `fn n_ctg(&self, sample: &str) -> Result<i32, String>`

Returns the number of contigs in a specific sample.

**Parameters:**
- `sample` - Name of the sample

**Returns:**
- `Ok(i32)` with contig count
- `Err(String)` on failure

**Example:**
```rust
let agc = AgcFile::open("data.agc", true)?;
let count = agc.n_ctg("sample1")?;
println!("Sample 'sample1' has {} contigs", count);
```

**Test Coverage:** `test_n_ctg`

---

##### `fn list_ctg(&self, sample: Option<&str>) -> Result<Vec<String>, String>`

Lists contig names in a sample.

**Parameters:**
- `sample` - Sample name, or `None` to list all contigs

**Returns:**
- `Ok(Vec<String>)` containing contig names
- `Err(String)` on failure

**Examples:**
```rust
let agc = AgcFile::open("data.agc", true)?;

// List contigs for a specific sample
let contigs = agc.list_ctg(Some("sample1"))?;
for contig in contigs {
    println!("Contig: {}", contig);
}

// List all contigs (no sample filter)
let all_contigs = agc.list_ctg(None)?;
```

**Test Coverage:** `test_list_contigs`, `test_list_contigs_no_sample`

---

##### `fn get_ctg_len(&self, sample: Option<&str>, name: &str) -> Result<i32, String>`

Gets the length of a contig.

**Parameters:**
- `sample` - Sample name, or `None` if contig name is unique
- `name` - Contig name

**Returns:**
- `Ok(i32)` with contig length in base pairs
- `Err(String)` if contig not found or name is ambiguous without sample

**Examples:**
```rust
let agc = AgcFile::open("data.agc", true)?;

// Get length with sample specified
let len = agc.get_ctg_len(Some("sample1"), "chr1")?;
println!("chr1 length: {} bp", len);

// Get length for unique contig name
let len = agc.get_ctg_len(None, "unique_contig")?;
```

**Test Coverage:** `test_get_ctg_len`, `test_get_ctg_len_no_sample`

---

##### `fn get_ctg_seq(&self, sample: Option<&str>, name: &str, start: i32, end: i32) -> Result<String, String>`

Retrieves a sequence range from a contig.

**Parameters:**
- `sample` - Sample name, or `None` if contig name is unique
- `name` - Contig name
- `start` - Start position (0-based, inclusive)
- `end` - End position (0-based, exclusive)

**Returns:**
- `Ok(String)` containing the DNA sequence
- `Err(String)` on failure

**Examples:**
```rust
let agc = AgcFile::open("data.agc", true)?;

// Get first 100 bases
let seq = agc.get_ctg_seq(Some("sample1"), "chr1", 0, 100)?;
println!("First 100 bases: {}", seq);

// Get bases 1000-2000
let seq = agc.get_ctg_seq(Some("sample1"), "chr1", 1000, 2000)?;
println!("Region 1000-2000: {}", seq);

// Get entire contig
let len = agc.get_ctg_len(Some("sample1"), "chr1")?;
let full_seq = agc.get_ctg_seq(Some("sample1"), "chr1", 0, len)?;
```

**Test Coverage:** `test_get_ctg_seq`, `test_get_ctg_seq_range`, `test_get_ctg_seq_invalid_contig`

---

## Complete Workflow Example

This example demonstrates a complete workflow for analyzing an AGC file:

```rust
use agc_bindings::AgcFile;

fn analyze_agc_file(filename: &str) -> Result<(), String> {
    // Open the file with prefetching for multiple queries
    let agc = AgcFile::open(filename, true)?;

    println!("=== AGC File Analysis ===\n");

    // Get reference sample
    if let Ok(ref_sample) = agc.reference_sample() {
        println!("Reference sample: {}\n", ref_sample);
    }

    // List all samples
    let samples = agc.list_sample()?;
    println!("Total samples: {}\n", samples.len());

    // Analyze each sample
    for sample in &samples {
        println!("Sample: {}", sample);

        // Get contigs for this sample
        let contigs = agc.list_ctg(Some(sample))?;
        println!("  Contigs: {}", contigs.len());

        // Analyze first contig
        if let Some(contig) = contigs.first() {
            let len = agc.get_ctg_len(Some(sample), contig)?;
            println!("  First contig '{}': {} bp", contig, len);

            // Get first 50 bases
            let preview_len = std::cmp::min(50, len);
            let seq = agc.get_ctg_seq(Some(sample), contig, 0, preview_len)?;
            println!("  Sequence preview: {}", seq);
        }
        println!();
    }

    Ok(())
}

fn main() {
    match analyze_agc_file("genome_data.agc") {
        Ok(()) => println!("Analysis complete"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

**Test Coverage:** `test_full_workflow`

---

## Multiple File Handles

You can open multiple AGC files simultaneously or open the same file multiple times:

```rust
// Open different files
let agc1 = AgcFile::open("data1.agc", true)?;
let agc2 = AgcFile::open("data2.agc", true)?;

// Open same file with different settings
let agc_prefetch = AgcFile::open("data.agc", true)?;
let agc_no_prefetch = AgcFile::open("data.agc", false)?;
```

Each `AgcFile` instance maintains its own handle and is automatically closed when dropped.

**Test Coverage:** `test_multiple_files`

---

## Performance Considerations

### Prefetching

- **Enable prefetching (`true`)** when:
  - Making multiple sequence queries
  - File size is manageable for your available memory
  - You need consistently fast query performance

- **Disable prefetching (`false`)** when:
  - Making only a few queries
  - Working with very large files
  - Memory is limited

### Example Performance Comparison

```rust
use std::time::Instant;

// With prefetching (faster for multiple queries)
let start = Instant::now();
let agc = AgcFile::open("data.agc", true)?;
for i in 0..1000 {
    let seq = agc.get_ctg_seq(Some("sample1"), "chr1", i * 100, (i + 1) * 100)?;
}
println!("With prefetch: {:?}", start.elapsed());

// Without prefetching (slower for multiple queries)
let start = Instant::now();
let agc = AgcFile::open("data.agc", false)?;
for i in 0..1000 {
    let seq = agc.get_ctg_seq(Some("sample1"), "chr1", i * 100, (i + 1) * 100)?;
}
println!("Without prefetch: {:?}", start.elapsed());
```

**Test Coverage:** `test_prefetching_modes`

---

## Error Handling

All fallible operations return `Result<T, String>` where the error string describes what went wrong:

```rust
let agc = match AgcFile::open("data.agc", true) {
    Ok(file) => file,
    Err(e) => {
        eprintln!("Failed to open file: {}", e);
        return;
    }
};

let sequence = match agc.get_ctg_seq(None, "chr1", 0, 100) {
    Ok(seq) => seq,
    Err(e) => {
        eprintln!("Failed to get sequence: {}", e);
        return;
    }
};
```

Using the `?` operator for cleaner error propagation:

```rust
fn process_genome() -> Result<(), String> {
    let agc = AgcFile::open("data.agc", true)?;
    let samples = agc.list_sample()?;
    let contigs = agc.list_ctg(Some(&samples[0]))?;
    let seq = agc.get_ctg_seq(Some(&samples[0]), &contigs[0], 0, 100)?;

    println!("Sequence: {}", seq);
    Ok(())
}
```

**Test Coverage:** `test_open_nonexistent_file`, `test_get_ctg_seq_invalid_contig`

---

## Thread Safety

`AgcFile` implements `Send` and `Sync`, making it safe to share across threads:

```rust
use std::thread;
use std::sync::Arc;

let agc = Arc::new(AgcFile::open("data.agc", true)?);

let handles: Vec<_> = (0..4).map(|i| {
    let agc = Arc::clone(&agc);
    thread::spawn(move || {
        let samples = agc.list_sample().unwrap();
        println!("Thread {} found {} samples", i, samples.len());
    })
}).collect();

for handle in handles {
    handle.join().unwrap();
}
```

---

## Memory Management

The bindings automatically handle memory management:

- C strings are properly allocated and deallocated
- String arrays returned by C functions are freed after conversion to Rust `Vec<String>`
- File handles are automatically closed when `AgcFile` is dropped

You don't need to manually call any cleanup functions.

---

## Running Tests

To run the test suite, you need a test AGC file:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_full_workflow -- --nocapture

# Run with specific test file
TEST_FILE=my_data.agc cargo test
```

All tests require a valid AGC file at `test_data.agc` (or set via `TEST_FILE` environment variable).

---

## Low-Level FFI

If you need direct access to the C API, the raw FFI functions are available:

```rust
use agc_bindings::{agc_open, agc_close, agc_get_ctg_len};
use std::ffi::CString;

unsafe {
    let filename = CString::new("data.agc").unwrap();
    let handle = agc_open(filename.into_raw(), 1);
    if !handle.is_null() {
        // Use handle...
        agc_close(handle);
    }
}
```

However, using the safe `AgcFile` wrapper is strongly recommended.

---

## License

These bindings are provided for the AGC library which is distributed under the MIT license.
See the [AGC project homepage](https://github.com/refresh-bio/agc) for more information.

---

## Version

Compatible with AGC library version 3.2 (2024-11-21)
# AGC Rust Bindings

Rust FFI bindings for the [AGC (Assembled Genomes Compressor)](https://github.com/refresh-bio/agc) library.

## Prerequisites

Before building, you need either:

1. **AGC library installed system-wide**, or
2. **AGC source code** to build from source, or
3. **Pre-built AGC library** in a custom location

### Installing AGC System-Wide

#### From Source (Recommended)

```bash
# Clone AGC repository
git clone https://github.com/refresh-bio/agc.git
cd agc

# Build and install
make
sudo make install

# Or with CMake
mkdir build && cd build
cmake ..
make
sudo make install
```

#### Via Package Manager

```bash
# Homebrew (macOS)
brew install agc

# Apt (Ubuntu/Debian) - if available
sudo apt-get install libagc-dev
```

## Building the Rust Bindings

### Option 1: System-Installed AGC Library

If AGC is installed system-wide:

```bash
cargo build
cargo test
```

### Option 2: Custom AGC Library Location

If you have AGC installed in a custom location:

```bash
# Set the library directory
export AGC_LIB_DIR=/path/to/agc/lib
cargo build
cargo test
```

Or for a single build:

```bash
AGC_LIB_DIR=/path/to/agc/lib cargo build
AGC_LIB_DIR=/path/to/agc/lib cargo test
```

### Option 3: Build AGC From Source

If you have AGC source code:

```bash
# Set the source directory
export AGC_SOURCE_DIR=/path/to/agc/source
cargo build
cargo test
```

The build script will automatically compile AGC from source.

### Option 4: Using Vendored AGC (Future Feature)

```bash
cargo build --features vendored
```

## Running Tests

### Basic Test Run

```bash
cargo test
```

### Test with Output

```bash
cargo test -- --nocapture
```

### Test with Custom AGC File

By default, tests look for `test_data.agc` in the project root. To use a different file:

```bash
# Set test file location
export TEST_FILE=/path/to/your/test.agc
cargo test -- --nocapture
```

### Run Specific Tests

```bash
# Run a specific test
cargo test test_open_and_close

# Run tests matching a pattern
cargo test list_sample

# Run the full workflow test with output
cargo test test_full_workflow -- --nocapture
```

### Creating Test Data

If you don't have an AGC file for testing, you can create one using the AGC command-line tool:

```bash
# Install AGC CLI
git clone https://github.com/refresh-bio/agc.git
cd agc
make

# Create test AGC file from FASTA files
./agc create test_data.agc sample1:genome1.fa sample2:genome2.fa

# Or append to existing AGC
./agc append test_data.agc sample3:genome3.fa
```

## Project Structure

```
agc-bindings/
├── Cargo.toml          # Package configuration
├── build.rs            # Build script for linking AGC library
├── README.md           # This file
├── src/
│   └── lib.rs         # Main library with bindings and tests
├── examples/
│   ├── basic_usage.rs
│   └── analyze_genome.rs
└── test_data.agc      # Test AGC file (not included, must be provided)
```

## Examples

### Basic Usage

Create `examples/basic_usage.rs`:

```rust
use agc_bindings::AgcFile;

fn main() -> Result<(), String> {
    let agc = AgcFile::open("test_data.agc", true)?;

    let samples = agc.list_sample()?;
    println!("Found {} samples", samples.len());

    for sample in &samples {
        let contigs = agc.list_ctg(Some(sample))?;
        println!("Sample '{}' has {} contigs", sample, contigs.len());
    }

    Ok(())
}
```

Run with:

```bash
cargo run --example basic_usage
```

### Genome Analysis

Create `examples/analyze_genome.rs`:

```rust
use agc_bindings::AgcFile;

fn main() -> Result<(), String> {
    let agc = AgcFile::open("test_data.agc", true)?;

    let samples = agc.list_sample()?;

    for sample in &samples {
        println!("\nAnalyzing sample: {}", sample);

        let contigs = agc.list_ctg(Some(sample))?;

        for contig in &contigs {
            let len = agc.get_ctg_len(Some(sample), contig)?;
            let preview_len = std::cmp::min(50, len);
            let seq = agc.get_ctg_seq(Some(sample), contig, 0, preview_len)?;

            println!("  {}: {} bp", contig, len);
            println!("    Preview: {}", seq);
        }
    }

    Ok(())
}
```

Run with:

```bash
AGC_FILE=test_data.agc cargo run --example analyze_genome
```

## Troubleshooting

### "AGC library not found" Error

This means the build script couldn't locate the AGC library. Solutions:

1. **Install AGC system-wide** and ensure it's in your library path
2. **Set AGC_LIB_DIR**:
   ```bash
   export AGC_LIB_DIR=/usr/local/lib  # or wherever libagc is located
   ```
3. **Set AGC_SOURCE_DIR** to build from source:
   ```bash
   export AGC_SOURCE_DIR=/path/to/agc/source
   ```

### Library Path Issues on Linux

If you get runtime linking errors:

```bash
# Add AGC library to LD_LIBRARY_PATH
export LD_LIBRARY_PATH=/path/to/agc/lib:$LD_LIBRARY_PATH

# Or update ldconfig
sudo ldconfig /path/to/agc/lib
```

### Library Path Issues on macOS

```bash
# Add AGC library to DYLD_LIBRARY_PATH
export DYLD_LIBRARY_PATH=/path/to/agc/lib:$DYLD_LIBRARY_PATH

# Or use install_name_tool to fix the library path
install_name_tool -change libagc.dylib /path/to/libagc.dylib target/debug/your_binary
```

### Test File Not Found

If tests fail with file not found errors:

```bash
# Create a symlink to your AGC file
ln -s /path/to/your/file.agc test_data.agc

# Or set TEST_FILE environment variable
export TEST_FILE=/path/to/your/file.agc
cargo test
```

### Build Failures

If the build fails:

1. **Check AGC installation**:
   ```bash
   # Verify library exists
   find /usr -name "libagc.*" 2>/dev/null

   # Check if pkg-config knows about it
   pkg-config --libs --cflags agc
   ```

2. **Check compiler requirements**:
   - Ensure you have a C++17 compatible compiler
   - GCC 7+ or Clang 5+ recommended

3. **Enable verbose build**:
   ```bash
   cargo build -vv
   ```

## Development

### Running All Tests with Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Run tests with coverage
cargo tarpaulin --out Html
```

### Benchmarking

```bash
cargo bench
```

### Documentation

Generate and view documentation:

```bash
cargo doc --open
```

### Formatting and Linting

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings
```

## CI/CD Setup

Example GitHub Actions workflow (`.github/workflows/ci.yml`):

```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install AGC
        run: |
          git clone https://github.com/refresh-bio/agc.git
          cd agc
          make
          sudo make install

      - name: Build
        run: cargo build --verbose

      - name: Run tests
        run: |
          # Create test data
          ./agc/agc create test_data.agc test:test.fa
          cargo test --verbose
```

## Environment Variables Reference

| Variable | Purpose | Example |
|----------|---------|---------|
| `AGC_LIB_DIR` | Path to AGC library directory | `/usr/local/lib` |
| `AGC_SOURCE_DIR` | Path to AGC source for building | `/home/user/agc` |
| `TEST_FILE` | Path to test AGC file | `my_test.agc` |
| `LD_LIBRARY_PATH` | Runtime library path (Linux) | `/opt/agc/lib` |
| `DYLD_LIBRARY_PATH` | Runtime library path (macOS) | `/opt/agc/lib` |

## Performance Tips

1. **Use prefetching for multiple queries**:
   ```rust
   let agc = AgcFile::open("data.agc", true)?;  // Enable prefetching
   ```

2. **Disable prefetching for large files or single queries**:
   ```rust
   let agc = AgcFile::open("large.agc", false)?;  // Disable prefetching
   ```

3. **Reuse AgcFile handles** instead of reopening files

4. **Use parallel processing** for analyzing multiple samples:
   ```rust
   use rayon::prelude::*;

   samples.par_iter().for_each(|sample| {
       // Process each sample in parallel
   });
   ```

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see LICENSE file for details

## Links

- [AGC Library](https://github.com/refresh-bio/agc)
- [API Documentation](https://docs.rs/agc-bindings)
- [Report Issues](https://github.com/yourusername/agc-bindings/issues)

## Version History

### 0.1.0 (Initial Release)
- Complete FFI bindings for AGC C API
- Safe Rust wrapper with `AgcFile`
- Comprehensive test suite
- Build script with multiple library discovery methods
- Documentation and examples
