;; guix build -L . -L .guix/modules -f guix.scm
;; guix build -L . -L .guix/modules bio-agclib
;; guix shell -C -D -N -F -L . -L .guix/modules crusco-shell
;;    cargo build


(define-module (guix)
  #:use-module (guix gexp)
  #:use-module (guix utils)
  #:use-module (guix packages)
  #:use-module (guix git-download)
  #:use-module (guix build-system cargo)
  #:use-module (guix build-system gnu)
  #:use-module (guix import crate)
  #:use-module ((guix licenses) #:prefix license:)

  #:use-module (gnu packages base)
  #:use-module (gnu packages c)
  #:use-module (gnu packages cmake)
  #:use-module (gnu packages commencement)
  #:use-module (gnu packages compression)
  #:use-module (gnu packages curl)
  #:use-module (gnu packages llvm)
  #:use-module (gnu packages nss)
  #:use-module (gnu packages perl)
  #:use-module (gnu packages pkg-config)
  #:use-module (gnu packages rust)
  #:use-module (gnu packages rust-crates)
  #:use-module (gnu packages tls)

  #:use-module (libagc-sys bioinformatics)
  )


(define-public bio-agclib
  (let ((commit "0eca9c6851c6490180a6a9683b752e16bc37d6df"))
  (package
    (name "bio-agclib")
    (version (string-append "3.2.1-" (string-take commit 7)))
    (source (origin
             (method git-fetch)
             (uri (git-reference
                   (url "https://github.com/pjotrp/agc")
                   (commit commit)
                   (recursive? #t)))
             (file-name (string-append name "-" version "-checkout"))
             (sha256
              (base32
               "0523pdgrbhgnaykd6nxvzncs4z91kpv5aq842zdb3pbkf8qciqv2"
               ))))
    (inputs (list
      libdeflate
      mimalloc
      ;; coreutils
      ;; sed
      ;; minizip-ng
      `(,zstd "lib")
      libdeflate
      lzlib
      ;; zstd
      zlib
      ))

    (build-system gnu-build-system)
    (arguments
     (list
      #:tests? #f ;; no tests
      #:make-flags
        #~(list (string-append "PREFIX=" #$output)
                (string-append "CC=" #$(cc-for-target))
                (string-append "CXX=" #$(cxx-for-target))
                (string-append "AR=" #$(ar-for-target))
                (string-append "PLATFORM=avx2")
                ;; Override "/sbin/ldconfig" with simply "echo" since
                ;; we don't need ldconfig(8).
                "LDCONF=echo")
      #:phases
        #~(modify-phases %standard-phases
            (add-before 'build 'remove-building-vendored-dependencies
                        (lambda _
                          (substitute* "makefile"
                                       (("^[$].call ADD_MIMALLOC") "# ")
                                       (("^[$].call ADD_LIBDEFLATE") "# ")
                                       (("^[$].call ADD_LIBZSTD") "# ")
                                       (("^[$].call PROPOSE_ISAL") "# ")
                                       (("^[$].call PROPOSE_ZLIB_NG") "# ")
                                       (("^[$].call CHOOSE_GZIP_DECOMPRESSION") "# ")
                                       (("^[$].call ADD_PYBIND11") "# ")
                                       (("^[$].call SET_STATIC") "# ")
                                       )))
            (delete 'configure)
            (replace 'build
               ;; By default, only the static library is built.
               (lambda* (#:key make-flags parallel-build?
                         #:allow-other-keys)
                        (let* ((job-count (if parallel-build?
                                              (number->string (parallel-job-count))
                                              1))
                               (jobs (string-append "-j" job-count))
                               (target #$(if (target-mingw?)
                                             "static"
                                             "shared")))
                          (apply invoke "make" "VERBOSE=1" "libagc_so" jobs make-flags)
                          )))
            (replace 'install
               ;; Upstream provides no install phase.
               (lambda _
                 (let* ((lib (string-append #$output "/lib"))
                        (inc (string-append #$output "/include/agc"))
                        )
                (install-file "src/lib-cxx/agc-api.h" inc)
                (install-file "bin/libagc.so" lib)
                ;; (install-file "bin/libagc.a" lib)
                )))
            )))

    (properties '((tunable? . #t)))

    (home-page "https://github.com/refresh-bio/agc")
    (synopsis "High levels of population-based compression for sequence data")
    (description "
Assembled Genomes Compressor (AGC) is a tool designed to compress collections of de-novo assembled genomes. It can be used for various types of datasets: short genomes (viruses) as well as long (humans).")

    (license license:expat))))


(define %source-dir (dirname (dirname (dirname (current-source-directory)))))

(define vcs-file?
  (or (git-predicate %source-dir)
      (const #t)))

(define-public libagc-sys
  (package
    (name "libagc-sys")
    (version "0.1.0")
    (source
     (local-file "../../.." ; find root from .guix/modules/agc in repo
                 "agc-checkout"
                 #:recursive? #t
                 #:select? vcs-file?))
    (build-system cargo-build-system)
    (inputs (cons*
             (cargo-inputs 'libagc-sys #:module '(libagc-sys rust-crates))
             ))
    (propagated-inputs (list
                        bio-agclib
                        pkg-config
                        zstd))
    (arguments
     (list
      #:phases
      #~(modify-phases %standard-phases
                       (replace 'build
                                (lambda _
                                  (invoke "cargo" "build" "-vv")
                                )
                       ))))
    (synopsis "High levels of population-based compression for sequence data")
    (description "
Assembled Genomes Compressor (AGC) is a tool designed to compress collections of de-novo assembled genomes. It can be used for various types of datasets: short genomes (viruses) as well as long (humans).")

    (license license:expat)
    (home-page "https://github.com/pangenome/libagc-sys ")))

(define-public libagc-sys-dont-build
  "Does not run cargo build before shell invocation"
  (package
    (inherit libagc-sys)
    (name "libagc-sys-dont-build")
    (arguments
     (list
      #:tests? #f
      #:phases
      #~(modify-phases %standard-phases
                       (delete 'build)
                       (delete 'install)
                       )))))

(define-public crusco-shell
  "Shell version to use 'cargo build' against guix rust"
  (package
    (inherit libagc-sys-dont-build)
    (name "crusco-shell")
    (build-system cargo-build-system)
    (propagated-inputs (list cmake rust nss-certs openssl perl gnu-make-4.2
                             findutils
                             bio-agclib
                             mimalloc
                             coreutils which perl binutils gcc-toolchain pkg-config zlib
                             sed curl clang
                             `(,zstd "lib")
                             `(,rust "cargo")
                             )) ;; to run cargo build in the shell
    (arguments
     (list
      #:tests? #f
      #:phases
      #~(modify-phases %standard-phases
                       (delete 'build)
                       (delete 'package)
                       (delete 'install))))
    ))

libagc-sys
