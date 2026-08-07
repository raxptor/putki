//! Writes the polymorphic package fixture the C# tests read back.
//!
//!     compiler.sh tests/rust
//!     cd tests/rust && cargo run --example write-fixture -- ../fixtures/polymorphic.pkg
//!
//! Needs a checkout that is not inside another cargo workspace.

use std::path::Path;

fn main() {
    let out = match std::env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: write-fixture <output.pkg>");
            std::process::exit(1);
        }
    };
    let bytes = putkitest::build_package(&putkitest::data_dir(), &["dlg"]);
    std::fs::write(Path::new(&out), &bytes).unwrap_or_else(|e| panic!("write {out}: {e}"));
    println!("wrote {} ({} bytes)", out, bytes.len());
}
