//! End-to-end exercise of the putki pipeline: text data in `data/`, through the
//! inki build, out as a binary package, and back in through the outki reader.
//!
//! `compiler.sh tests/rust` must have run first -- the generated crates this
//! depends on are not checked in.

use std::path::Path;
use std::sync::Arc;

use gen_test_inki::inki;

/// Builds a package containing `paths` and everything they reference.
pub fn build_package(data_dir: &Path, paths: &[&str]) -> Vec<u8> {
    let la = Arc::new(putki_inki::LoadAll::new(data_dir));
    let desc = putki_inki::PipelineDesc::new(la, data_dir);
    let pipeline = Arc::new(putki_inki::Pipeline::new(desc));

    for path in paths {
        pipeline.build_as::<inki::Dialog>(path);
    }
    while pipeline.take() {}

    {
        let records = pipeline.peek_build_records().expect("build records lock");
        let failed: Vec<&str> = records
            .values()
            .filter(|r| !r.is_ok())
            .map(|r| r.get_path())
            .collect();
        assert!(failed.is_empty(), "build failed for: {}", failed.join(", "));
    }

    let mut recipe = putki_inki::PackageRecipe::new();
    for path in paths {
        recipe
            .add_object(&*pipeline, path, true)
            .unwrap_or_else(|e| panic!("add_object({path}) failed: {e:?}"));
    }
    putki_inki::write_package(&*pipeline, &recipe).expect("write_package failed")
}

/// The data directory of this test project, independent of the cwd cargo picks.
pub fn data_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}
