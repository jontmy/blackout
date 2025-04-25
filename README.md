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
cargo run --release -- -i <input.pdf> -o <output.pdf> [-p <password>]
```
- The input path must be a single PDF file. Support for folders of PDFs is planned.
- The output path can be a directory, in which case Blackout will use the same file name as the input PDF.
- If your document is password protected, use the optional `-p` flag.

### Examples
<img width="1724" alt="Before and after of Blackout" src="https://github.com/user-attachments/assets/a199de3d-34b4-480b-9d7d-f6ab83a08f24" />

#### Limitations
Blackout internally converts PDFs into images and forces all non-white pixels to black (posterization).
Therefore, some limitations exist at the moment:

- Only A4 paper sizes are supported.
- Pages are converted to images internally at 300 dpi.
- Text in the output document cannot be selected.
- Images in the original document will likely end up as a pure black blob (see below).

<img width="1724" alt="Example of posterization on images" src="https://github.com/user-attachments/assets/8edc33ce-16d1-4899-84ee-97af4123b221" />

