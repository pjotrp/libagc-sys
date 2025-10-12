;; Import a package with:
;; guix import crate --lockfile=Cargo.lock cc

(define-module (libagc-sys rust-crates)
  #:use-module (guix packages)
  #:use-module (guix git-download)
  #:use-module (guix build-system cargo)

  #:use-module (gnu packages)

  #:export (lookup-cargo-inputs))

(define rust-cc-1.2.41
  (crate-source "cc" "1.2.41"
                "1dvwli6fljqc7kgmihb249rmdfs5irla1h0n6vkavdi4pg6yd7xc"))

(define rust-find-msvc-tools-0.1.4
  (crate-source "find-msvc-tools" "0.1.4"
                "09x1sfinrz86bkm6i2d85lpsfnxn0w797g5zisv1nwhaz1w1h1aj"))

(define rust-pkg-config-0.3.32
  (crate-source "pkg-config" "0.3.32"
                "0k4h3gnzs94sjb2ix6jyksacs52cf1fanpwsmlhjnwrdnp8dppby"))

(define rust-shlex-1.3.0
  (crate-source "shlex" "1.3.0"
                "0r1y6bv26c1scpxvhg2cabimrmwgbp4p3wy6syj9n0c4s3q2znhg"))

(define rust-vcpkg-0.2.15
  (crate-source "vcpkg" "0.2.15"
                "09i4nf5y8lig6xgj3f7fyrvzd3nlaw4znrihw8psidvv5yk4xkdc"))

(define-cargo-inputs lookup-cargo-inputs
                     (libagc-sys => (list
                                     rust-cc-1.2.41
                                     rust-pkg-config-0.3.32
                                     rust-find-msvc-tools-0.1.4
                                     rust-shlex-1.3.0
                                     rust-vcpkg-0.2.15
)))
