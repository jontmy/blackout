# Blackout
A tool to convert color/grayscale PDF files to black and white.
Yes, literally only black pixels and white pixels, no shades of gray.

### Pre-requisites
- [`pdfium`](https://github.com/bblanchon/pdfium-binaries)
- Rust 1.86.0 (stable) or later (if building from source)

>[!NOTE]
> If you don't have `pdfium` installed, you can download the precompiled binary (`.dylib` for macOS or `.so` for Linux) from the [pdfium-binaries](https://github.com/bblanchon/pdfium-binaries) repository. Place the binary in the same directory as the `blackout` binary.

>[!TIP]
> If you are running Blackout using Cargo, you can also place the `pdfium` binary in the root of the repository.

### Usage
```bash
cargo run --release -- -i <input.pdf> -o <output.pdf>
```
