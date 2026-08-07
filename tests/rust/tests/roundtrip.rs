//! Builds a package from the text data and reads it back through outki.
//!
//! The polymorphic cases are the point: `Dialog::node1` is an inline `@DlgSay`
//! and `node2` points at a bare `@IDlgNode`, so a package that loses the type
//! tag, or dispatches it to the wrong variant, fails here.

use gen_test_outki::outki;
use putki_outki::outki as poutki;

struct Slice(Vec<u8>);

impl poutki::PackageRandomAccess for Slice {
    fn read_chunk(
        &self,
        begin: usize,
        end: usize,
        f: &mut dyn FnMut(poutki::OutkiResult<&[u8]>) -> poutki::OutkiResult<()>,
    ) -> poutki::OutkiResult<()> {
        (*f)(Ok(&self.0[begin..end]))
    }
}

fn load(paths: &[&str]) -> poutki::BinPackageManager {
    let bytes = putkitest::build_package(&putkitest::data_dir(), paths);
    let manifest = {
        let mut slice = bytes.as_slice();
        poutki::PackageManifest::parse(&mut slice).expect("parse manifest")
    };
    let mut mgr = poutki::BinPackageManager::new();
    mgr.insert(poutki::Package::new(manifest, Box::new(Slice(bytes))));
    mgr
}

#[test]
fn polymorphic_pointers_keep_their_type() {
    let mgr = load(&["dlg"]);
    let dlg = mgr.resolve::<outki::Dialog>("dlg").expect("resolve dlg");

    assert_eq!(dlg.id, "DIALOG HEJ");

    // Inline @DlgSay: must come back as the child variant, not the base.
    match &*dlg.node1 {
        outki::IDlgNode::DlgSay(say) => {
            assert_eq!(say.text, "hej");
            assert_eq!(say.who, 0);
            assert_eq!(say.parent.id, "dlgsay");
        }
        other => panic!("node1 should be DlgSay, got {:?}", i32::from(other)),
    }

    // A bare @IDlgNode stays the base variant.
    match &*dlg.node2 {
        outki::IDlgNode::IDlgNode(_) => {}
        other => panic!("node2 should be the base IDlgNode, got {:?}", i32::from(other)),
    }
}
