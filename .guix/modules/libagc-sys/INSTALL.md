# Building with GNU Guix

**NOTE**: All paths are relative to the repository root. If you are not working
in the repository root, please update the paths to fit your work scenario.

To build `impg` with guix without making it available, run the following
command:

```sh
guix build -L .guix/modules --file=guix.scm
```

The `-L` option adds the `.guix/modules` directory to the front of the guile
load path. The `--file` option points to the `guix.scm` at the root of the
repository.

To build and "install" `impg` with guix, run:

```sh
guix install -L .guix/modules --file=guix.scm
```
