| :exclamation: Work in progress |
|--------------------------------|

# `headjack`

NIfTI viewer for the terminal.

(Placeholder screenshot)
![Screenshot](doc/demo.png)

## Usage

```sh
headjack image.nii.gz
```

### Controls

| Key | Action |
| --- | --- |
| <kbd>q</kbd> / <kbd>esc</kbd> / <kbd>ctrl</kbd> + <kbd>C</kbd>  | Quit |
| <kbd>&uarr;</kbd> <kbd>&darr;</kbd> / <kbd>A</kbd> <kbd>D</kbd> | Navigate X axis |
| <kbd>&larr;</kbd> <kbd>&rarr;</kbd> / <kbd>W</kbd> <kbd>S</kbd> | Navigate Y axis |
| <kbd>Z</kbd> <kbd>X</kbd> / <kbd>Y</kbd> <kbd>X</kbd> | Navigate Z axis |
| <kbd>tab</kbd> | Toggle metadata view |
| <kbd>c</kbd> | Toggle color map |

## Installation

### Precompiled binaries (recommended)

Head over to [releases](https://github.com/cmi-dair/headjack/releases) and download the latest binary for your platform.

**Optionally** add the binary to your `PATH` environment variable.

`headjack` is and will always be a single executable with 0 runtime dependencies.

### Build from source

With latest Rust compiler installed, run:

```sh
cargo build --release
```

The binary will be located at `target/release/headjack`.

## Troubleshooting

### Terminal colors look weird

Your terminal might not support 24-bit colors. Try running `headjack` with `-a`/`--ansi` for 256 ANSI color mode or `-b`/`--bw` for black and white mode.

### Unix: libc error on startup 

If you get an error like this:

```
/lib64/libm.so.6: versionGLIBC_2.29' not found
```

You are probably using an older version of glibc. Try using the `*-musl` variant from the [releases page](https://github.com/cmi-dair/headjack/releases). Or build from source with `musl` target.
