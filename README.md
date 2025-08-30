# rw-parser-rs

[![Crates.io](https://img.shields.io/crates/v/rw-parser-rs.svg)](https://crates.io/crates/rw-parser-rs)
[![Docs.rs](https://docs.rs/rw-parser-rs/badge.svg)](https://docs.rs/rw-parser-rs)
[![License](https://img.shields.io/crates/l/rw-parser-rs.svg)](https://github.com/DepsCian/rw-parser-rs/blob/main/LICENSE)

A high-performance, native Rust parser for RenderWare files (`.dff`, `.txd`, `.ifp`).

This project is a Rust port of the excellent [rw-parser-ng](https://github.com/DepsCian/rw-parser-ng), rewritten from the ground up to leverage Rust's performance and safety. The goal is to provide a significantly faster and more memory-efficient alternative for server-side processing and tooling.

Our benchmarks show that `rw-parser-rs` is approximately **100x faster** than its Node.js counterpart for typical DFF parsing tasks.

## Features

*   **Blazing Fast:** Native Rust performance for maximum throughput.
*   **DFF (Model) Parsing:** Extracts geometry, materials, frames, and skinning data.
*   **TXD (Texture Dictionary) Parsing:** Extracts texture information and decompresses DXT formats.
*   **IFP (Animation) Parsing:** Extracts animation data for `ANP3` and `ANPK` formats.
*   **Safe & Robust:** Built with Rust's safety guarantees to prevent common parsing vulnerabilities.
*   **Strongly Typed:** Ensures data integrity and a great developer experience.

## Installation

Add `rw-parser-rs` to your `Cargo.toml`:

```toml
[dependencies]
rw-parser-rs = "1.0.0"
```

## Usage

```rust
use rw_parser_rs::renderware::dff::dff_parser::DffParser;
use std::fs;

fn main() -> std::io::Result<()> {
    // DFF
    let dff_buffer = fs::read("path/to/your/model.dff")?;
    let mut dff_parser = DffParser::new(&dff_buffer);
    let dff_data = dff_parser.parse()?;
    println!("Successfully parsed DFF model: {}", dff_data.version);

    // Similar usage for TxdParser and IfpParser

    Ok(())
}
```

## Development

1.  Clone the repository: `git clone https://github.com/DepsCian/rw-parser-rs.git`
2.  Build the project: `cargo build --release`
3.  Run benchmarks: `cargo bench`
4.  Generate documentation: `cargo doc --open`

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the GPL-3.0 License.