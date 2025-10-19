use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

/// Opaque type representing an AGC file handle
#[repr(C)]
pub struct agc_t {
    _private: [u8; 0],
}

#[link(name = "agc")]
unsafe extern "C" {
    /// Open an AGC file
    ///
    /// # Arguments
    /// * `fn` - file name
    /// * `prefetching` - 1 to preload whole file into memory, 0 otherwise
    ///
    /// # Returns
    /// NULL for error
    fn agc_open(fn_: *mut c_char, prefetching: c_int) -> *mut agc_t;

    /// Close an AGC file
    ///
    /// # Arguments
    /// * `agc` - agc handle
    ///
    /// # Returns
    /// 0 for success and -1 for error
    fn agc_close(agc: *mut agc_t) -> c_int;

    /// Get the length of a contig
    ///
    /// # Arguments
    /// * `agc` - agc handle
    /// * `sample` - sample name; can be NULL
    /// * `name` - contig name
    ///
    /// # Returns
    /// contig length, or <0 for errors
    fn agc_get_ctg_len(agc: *const agc_t, sample: *const c_char, name: *const c_char) -> c_int;

    /// Get contig sequence
    ///
    /// # Arguments
    /// * `agc` - agc handle
    /// * `sample` - sample name; can be NULL
    /// * `name` - contig name
    /// * `start` - start offset
    /// * `end` - end offset
    /// * `buf` - sequence buffer; user should allocate memory
    ///
    /// # Returns
    /// contig length, or <0 for errors
    fn agc_get_ctg_seq(
        agc: *const agc_t,
        sample: *const c_char,
        name: *const c_char,
        start: c_int,
        end: c_int,
        buf: *mut c_char,
    ) -> c_int;

    /// Get the number of samples
    ///
    /// # Arguments
    /// * `agc` - agc handle
    ///
    /// # Returns
    /// the number of samples
    fn agc_n_sample(agc: *const agc_t) -> c_int;

    /// Get the number of contigs in a sample
    ///
    /// # Arguments
    /// * `agc` - agc handle
    /// * `sample` - sample name
    ///
    /// # Returns
    /// the number of contigs in sample
    fn agc_n_ctg(agc: *const agc_t, sample: *const c_char) -> c_int;

    /// Get reference sample name
    ///
    /// # Arguments
    /// * `agc` - agc handle
    ///
    /// # Returns
    /// NULL-terminated string. Use agc_string_destroy() to deallocate.
    fn agc_reference_sample(agc: *const agc_t) -> *mut c_char;

    /// List all samples
    ///
    /// # Arguments
    /// * `agc` - agc handle
    /// * `n_sample` - number of samples (returned value)
    ///
    /// # Returns
    /// array of NULL-terminated strings. Use agc_list_destroy() to deallocate.
    fn agc_list_sample(agc: *const agc_t, n_sample: *mut c_int) -> *mut *mut c_char;

    /// List all contigs in a sample
    ///
    /// # Arguments
    /// * `agc` - agc handle
    /// * `sample` - sample name; can be NULL
    /// * `n_ctg` - number of contigs (returned value)
    ///
    /// # Returns
    /// array of NULL-terminated strings. Use agc_list_destroy() to deallocate.
    fn agc_list_ctg(
        agc: *const agc_t,
        sample: *const c_char,
        n_ctg: *mut c_int,
    ) -> *mut *mut c_char;

    /// Deallocate an array of strings
    ///
    /// # Arguments
    /// * `list` - array to deallocate
    fn agc_list_destroy(list: *mut *mut c_char) -> c_int;

    // fn agc_string_destroy(sample: *mut c_char) -> c_int; FIXME
}

/// Safe wrapper for AGC file operations
pub struct AgcFile {
    handle: *mut agc_t,
}

impl AgcFile {
    /// Open an AGC file
    ///
    /// # Arguments
    /// * `filename` - path to the AGC file
    /// * `prefetching` - whether to preload the entire file into memory
    ///
    /// # Returns
    /// Result containing AgcFile or an error message
    pub fn open(filename: &str, prefetching: bool) -> Result<Self, String> {
        let c_filename = CString::new(filename).map_err(|e| e.to_string())?;
        let prefetch_flag = if prefetching { 1 } else { 0 };

        unsafe {
            // Note: We can't catch C++ exceptions with catch_unwind reliably
            // The AGC C API should not throw exceptions across the boundary
            let handle = agc_open(c_filename.into_raw(), prefetch_flag);
            if handle.is_null() {
                Err(format!("Failed to open AGC file: {}", filename))
            } else {
                Ok(AgcFile { handle })
            }
        }
    }

    /// Get the length of a contig
    ///
    /// # Arguments
    /// * `sample` - sample name (None for unspecified)
    /// * `name` - contig name
    ///
    /// # Returns
    /// Result containing contig length or an error
    pub fn get_ctg_len(&self, sample: Option<&str>, name: &str) -> Result<i32, String> {
        let c_name = CString::new(name).map_err(|e| e.to_string())?;
        let c_sample = sample.map(|s| CString::new(s).ok()).flatten();

        unsafe {
            let sample_ptr = c_sample
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null());
            let len = agc_get_ctg_len(self.handle, sample_ptr, c_name.as_ptr());

            if len < 0 {
                Err(format!("Failed to get contig length for: {}", name))
            } else {
                Ok(len)
            }
        }
    }

    /// Get contig sequence
    ///
    /// # Arguments
    /// * `sample` - sample name (None for unspecified)
    /// * `name` - contig name
    /// * `start` - start offset
    /// * `end` - end offset
    ///
    /// # Returns
    /// Result containing the sequence string or an error
    pub fn get_ctg_seq(
        &self,
        sample: Option<&str>,
        name: &str,
        start: i32,
        end: i32,
    ) -> Result<String, String> {
        let c_name = CString::new(name).map_err(|e| e.to_string())?;
        let c_sample = sample.map(|s| CString::new(s).ok()).flatten();

        // let len = self.get_ctg_len(sample, name)?;
        let buf_size = (end - start) as usize + 1;
        let mut buffer = vec![0u8; buf_size];

        unsafe {
            let sample_ptr = c_sample
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null());
            let result = agc_get_ctg_seq(
                self.handle,
                sample_ptr,
                c_name.as_ptr(),
                start,
                end,
                buffer.as_mut_ptr() as *mut c_char,
            );

            if result < 0 {
                Err(format!("Failed to get contig sequence for: {}", name))
            } else {
                String::from_utf8(buffer[..result as usize].to_vec())
                    .map_err(|e| format!("Invalid UTF-8 sequence: {}", e))
            }
        }
    }

    /// Get the number of samples
    pub fn n_sample(&self) -> i32 {
        unsafe { agc_n_sample(self.handle) }
    }

    /// Get the number of contigs in a sample
    ///
    /// # Arguments
    /// * `sample` - sample name
    pub fn n_ctg(&self, sample: &str) -> Result<i32, String> {
        let c_sample = CString::new(sample).map_err(|e| e.to_string())?;
        unsafe { Ok(agc_n_ctg(self.handle, c_sample.as_ptr())) }
    }

    /// Get reference sample name
    pub fn reference_sample(&self) -> Result<String, String> {
        unsafe {
            let ptr = agc_reference_sample(self.handle);
            if ptr.is_null() {
                return Err("Failed to get reference sample".to_string());
            }

            let c_str = CStr::from_ptr(ptr);
            let result = c_str.to_string_lossy().into_owned();
            // agc_string_destroy(ptr); FIXME
            Ok(result)
        }
    }

    /// List all samples
    pub fn list_sample(&self) -> Result<Vec<String>, String> {
        unsafe {
            let mut n_sample: c_int = 0;
            let list = agc_list_sample(self.handle, &mut n_sample);

            if list.is_null() {
                return Err("Failed to list samples".to_string());
            }

            let mut samples = Vec::new();
            for i in 0..n_sample {
                let ptr = *list.offset(i as isize);
                if !ptr.is_null() {
                    let c_str = CStr::from_ptr(ptr);
                    samples.push(c_str.to_string_lossy().into_owned());
                }
            }

            agc_list_destroy(list);
            Ok(samples)
        }
    }

    /// List all contigs in a sample
    ///
    /// # Arguments
    /// * `sample` - sample name (None for unspecified)
    pub fn list_ctg(&self, sample: Option<&str>) -> Result<Vec<String>, String> {
        let c_sample = sample.map(|s| CString::new(s).ok()).flatten();

        unsafe {
            let mut n_ctg: c_int = 0;
            let sample_ptr = c_sample
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null());
            let list = agc_list_ctg(self.handle, sample_ptr, &mut n_ctg);

            if list.is_null() {
                return Err("Failed to list contigs".to_string());
            }

            let mut contigs = Vec::new();
            for i in 0..n_ctg {
                let ptr = *list.offset(i as isize);
                if !ptr.is_null() {
                    let c_str = CStr::from_ptr(ptr);
                    contigs.push(c_str.to_string_lossy().into_owned());
                }
            }

            agc_list_destroy(list);
            Ok(contigs)
        }
    }
}

impl Drop for AgcFile {
    fn drop(&mut self) {
        unsafe {
            agc_close(self.handle);
        }
    }
}

unsafe impl Send for AgcFile {}
unsafe impl Sync for AgcFile {}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FILE: &str = "test/data/input/test.agc";

    #[test]
    fn test_open_and_close() {
        let agc = AgcFile::open(TEST_FILE, true);
        assert!(agc.is_ok(), "Failed to open AGC file");
    }

    #[test]
    fn test_open_nonexistent_file() {
        let result = AgcFile::open("nonexistent.agc", false);
        assert!(result.is_err(), "Should fail to open nonexistent file");
    }

    #[test]
    fn test_n_sample() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let n_samples = agc.n_sample();
        assert!(n_samples > 0, "Should have at least one sample");
        println!("Number of samples: {}", n_samples);
    }

    #[test]
    fn test_list_samples() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let samples = agc.list_sample().expect("Failed to list samples");
        assert!(!samples.is_empty(), "Should have at least one sample");
        println!("Samples: {:?}", samples);
    }

    #[test]
    fn test_reference_sample() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let ref_sample = agc.reference_sample().expect("Failed to get reference sample");
        assert!(!ref_sample.is_empty(), "Reference sample should not be empty");
        println!("Reference sample: {}", ref_sample);
    }

    #[test]
    fn test_list_contigs() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let samples = agc.list_sample().expect("Failed to list samples");

        if !samples.is_empty() {
            let sample = &samples[0];
            let contigs = agc.list_ctg(Some(sample)).expect("Failed to list contigs");
            assert!(!contigs.is_empty(), "Should have at least one contig");
            println!("Contigs in sample '{}': {:?}", sample, contigs);
        }
    }

    #[ignore] // FIXME
    #[test]
    fn test_list_contigs_no_sample() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let contigs = agc.list_ctg(None).expect("Failed to list contigs");
        println!("All contigs: {:?}", contigs);
    }

    #[test]
    fn test_n_ctg() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let samples = agc.list_sample().expect("Failed to list samples");

        if !samples.is_empty() {
            let sample = &samples[0];
            let n_ctg = agc.n_ctg(sample).expect("Failed to get contig count");
            assert!(n_ctg > 0, "Should have at least one contig");
            println!("Number of contigs in '{}': {}", sample, n_ctg);
        }
    }

    #[test]
    fn test_get_ctg_len() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let samples = agc.list_sample().expect("Failed to list samples");

        if !samples.is_empty() {
            let sample = &samples[0];
            let contigs = agc.list_ctg(Some(sample)).expect("Failed to list contigs");

            if !contigs.is_empty() {
                let contig = &contigs[0];
                let len = agc.get_ctg_len(Some(sample), contig)
                    .expect("Failed to get contig length");
                assert!(len > 0, "Contig length should be positive");
                println!("Length of '{}' in '{}': {}", contig, sample, len);
            }
        }
    }

    #[ignore] // FIXME
    #[test]
    fn test_get_ctg_len_no_sample() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let contigs = agc.list_ctg(None).expect("Failed to list contigs");

        if !contigs.is_empty() {
            let contig = &contigs[0];
            let len = agc.get_ctg_len(None, contig)
                .expect("Failed to get contig length");
            assert!(len > 0, "Contig length should be positive");
            println!("Length of '{}': {}", contig, len);
        }
    }

    #[test]
    fn test_get_ctg_seq() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let samples = agc.list_sample().expect("Failed to list samples");

        if !samples.is_empty() {
            let sample = &samples[0];
            let contigs = agc.list_ctg(Some(sample)).expect("Failed to list contigs");

            if !contigs.is_empty() {
                let contig = &contigs[0];
                let len = agc.get_ctg_len(Some(sample), contig)
                    .expect("Failed to get contig length");

                // Get first 100 bases or entire sequence if shorter
                let end = std::cmp::min(100, len);
                let seq = agc.get_ctg_seq(Some(sample), contig, 0, end)
                    .expect("Failed to get contig sequence");

                assert!(!seq.is_empty(), "Sequence should not be empty");
                assert!(seq.len() <= end as usize, "Sequence length should match request");
                println!("Sequence of '{}' (0-{}): {}", contig, end, seq);
            }
        }
    }

    #[test]
    fn test_get_ctg_seq_range() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let samples = agc.list_sample().expect("Failed to list samples");

        if !samples.is_empty() {
            let sample = &samples[0];
            let contigs = agc.list_ctg(Some(sample)).expect("Failed to list contigs");

            if !contigs.is_empty() {
                let contig = &contigs[0];
                let len = agc.get_ctg_len(Some(sample), contig)
                    .expect("Failed to get contig length");

                if len > 50 {
                    // Get bases 10-50
                    let seq = agc.get_ctg_seq(Some(sample), contig, 10, 50)
                        .expect("Failed to get contig sequence");

                    assert_eq!(seq.len(), 40, "Should get 40 bases (50-10)");
                    println!("Sequence of '{}' (10-50): {}", contig, seq);
                }
            }
        }
    }

    #[test]
    fn test_get_ctg_seq_invalid_contig() {
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");
        let result = agc.get_ctg_seq(None, "nonexistent_contig", 0, 100);
        assert!(result.is_err(), "Should fail for nonexistent contig");
    }

    #[test]
    fn test_multiple_files() {
        let agc1 = AgcFile::open(TEST_FILE, true);
        let agc2 = AgcFile::open(TEST_FILE, false);

        assert!(agc1.is_ok(), "First file should open");
        assert!(agc2.is_ok(), "Second file should open");

        if let (Ok(f1), Ok(f2)) = (agc1, agc2) {
            assert_eq!(f1.n_sample(), f2.n_sample(),
                "Both handles should see same number of samples");
        }
    }

    #[test]
    fn test_prefetching_modes() {
        // Test with prefetching enabled
        let agc_prefetch = AgcFile::open(TEST_FILE, true).expect("Failed to open with prefetch");
        let samples_prefetch = agc_prefetch.list_sample().expect("Failed to list samples");

        // Test without prefetching
        let agc_no_prefetch = AgcFile::open(TEST_FILE, false).expect("Failed to open without prefetch");
        let samples_no_prefetch = agc_no_prefetch.list_sample().expect("Failed to list samples");

        assert_eq!(samples_prefetch, samples_no_prefetch,
            "Results should be identical regardless of prefetching mode");
    }

    #[test]
    fn test_full_workflow() {
        // Complete workflow: open, list, query
        let agc = AgcFile::open(TEST_FILE, true).expect("Failed to open file");

        println!("\n=== Full AGC File Analysis ===");

        // Get reference sample
        if let Ok(ref_sample) = agc.reference_sample() {
            println!("Reference sample: {}", ref_sample);
        }

        // List all samples
        let samples = agc.list_sample().expect("Failed to list samples");
        println!("Total samples: {}", samples.len());

        // For each sample, list contigs
        for sample in &samples {
            let contigs = agc.list_ctg(Some(sample)).expect("Failed to list contigs");
            println!("\nSample '{}' has {} contigs:", sample, contigs.len());

            // Get details for first contig
            if let Some(contig) = contigs.first() {
                if let Ok(len) = agc.get_ctg_len(Some(sample), contig) {
                    println!("  Contig '{}': {} bp", contig, len);

                    // Get first 50 bases
                    if let Ok(seq) = agc.get_ctg_seq(Some(sample), contig, 0, std::cmp::min(50, len)) {
                        println!("  Sequence (first 50bp): {}", seq);
                    }
                }
            }
        }
    }
}
