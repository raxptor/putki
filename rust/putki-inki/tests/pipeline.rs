#![allow(unused_imports)]
#![allow(unused_must_use)]
extern crate putki_inki;
extern crate putki_outki;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::any;
use std::any::TypeId;
use std::thread;
use std::path::Path;

use putki_inki::PutkiError;
use putki_inki::FieldWriter;
use putki_inki::Ptr;
use putki_inki::BinWriter;
use putki_outki::outki as outki;
use crate::outki::PackageManifest;
use crate::outki::BinReader;

// kv_to_string feeds both the text output and the hash that gives anonymous
// inline objects their identity, so it must not depend on hash iteration order.
// Two separately-lexed copies of the same object get distinct HashMap seeds,
// which is what makes an unordered walk observable here.
#[test]
fn test_kv_to_string_is_ordered() {
	let src = r#"@Thing thing { zebra: 1, alpha: 2, middle: 3, beta: 4 }"#;
	let lex_once = || {
		let objs = putki_inki::lex_file(src);
		match objs.get("thing") {
			Some(putki_inki::LexedData::Object { kv, .. }) => putki_inki::kv_to_string(kv),
			_ => panic!("expected to lex an object"),
		}
	};
	let first = lex_once();
	assert_eq!(first, "{alpha:2,beta:4,middle:3,zebra:1,}");
	for _ in 0..16 {
		assert_eq!(first, lex_once(), "kv_to_string output is not deterministic");
	}
}

#[derive(Debug, Clone, Default)]
struct TestValues {
	value1: i32,
	value2: i32	
}

#[derive(Debug, Clone, Default)]
struct Multi {
	contained: TestValues
}

#[derive(Debug, Clone, Default)]
struct Pointer {	
	contained: TestValues,
	next: putki_inki::Ptr<Pointer>
}

struct PointerOutki {
	contained: TestValues,
	next: outki::NullablePtr<PointerOutki>
}

impl putki_inki::TypeDescriptor for TestValues {
	const TAG : &'static str = "TestValues";
	const TYPE_ID : usize = 1;
}

impl putki_inki::TypeDescriptor for Multi {
	const TAG : &'static str = "Multi";
	const TYPE_ID : usize = 2;
}

impl putki_inki::TypeDescriptor for Pointer {
	const TAG : &'static str = "Pointer";
	const TYPE_ID : usize = 3;
}

impl putki_inki::TypeDescriptor for PointerOutki {
	const TAG : &'static str = "Pointer";
	const TYPE_ID : usize = 3;
}
  
impl putki_inki::BuildFields for TestValues { }

impl putki_inki::BuildFields for Multi {
	fn build_fields(&mut self, pipeline:&putki_inki::Pipeline, br:&mut putki_inki::BuildRecord) -> Result<(), putki_inki::PutkiError> {
		pipeline.build(br, &mut self.contained);
		Ok(())
	}
}

impl outki::BinLoader for Multi {
   fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
			contained: TestValues::read(stream)
		}
   }
   fn resolve(&mut self, _context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> { Ok(()) }
}

impl outki::BinLoader for TestValues {
   fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
			value1: i32::read(stream),
			value2: i32::read(stream)
		}
   }
   fn resolve(&mut self, _context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> { Ok(()) }
}

impl outki::BinLoader for PointerOutki {
   fn read(stream:&mut outki::BinDataStream) -> Self {	   
        Self {
			contained: TestValues::read(stream),
			next: outki::NullablePtr::<PointerOutki>::read(stream)
		}
	}
	fn resolve(&mut self, context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> { 
        context.resolve(&mut self.next)
    }   
}



impl outki::OutkiObj for Multi { }
impl outki::OutkiObj for TestValues { }
impl outki::OutkiObj for PointerOutki { }

impl putki_inki::BuildFields for Pointer {
	fn build_fields(&mut self, pipeline:&putki_inki::Pipeline, br:&mut putki_inki::BuildRecord) -> Result<(), putki_inki::PutkiError> {
		pipeline.build(br, &mut self.contained);
		Ok(())
	}
}

impl putki_inki::BuildCandidate for TestValues {
    fn as_any_ref(&mut self) -> &mut dyn any::Any { return self; }    
    fn build(&mut self, p:&putki_inki::Pipeline, br: &mut putki_inki::BuildRecord) -> Result<(), putki_inki::PutkiError> { p.build(br, self) }
	fn scan_deps(&self, _p:&putki_inki::Pipeline, _br: &mut putki_inki::BuildRecord) { }
}

impl putki_inki::BuildCandidate for Multi {
    fn as_any_ref(&mut self) -> &mut dyn any::Any { return self; }    
    fn build(&mut self, p:&putki_inki::Pipeline, br: &mut putki_inki::BuildRecord) -> Result<(), putki_inki::PutkiError> { p.build(br, self) }
	fn scan_deps(&self, _p:&putki_inki::Pipeline, _br: &mut putki_inki::BuildRecord) { }
}

impl putki_inki::BuildCandidate for Pointer {
    fn as_any_ref(&mut self) -> &mut dyn any::Any { return self; }    
    fn build(&mut self, p:&putki_inki::Pipeline, br: &mut putki_inki::BuildRecord) -> Result<(), putki_inki::PutkiError> { p.build(br, self) }
	fn scan_deps(&self, p:&putki_inki::Pipeline, br: &mut putki_inki::BuildRecord) { 
		p.add_output_dependency(br, &self.next);
	}
}

struct TestValueBuilder { }
struct PointerBuilder { }

impl putki_inki::Builder<TestValues> for TestValueBuilder {
	fn desc(&self) -> putki_inki::BuilderDesc {
		putki_inki::BuilderDesc {
			description: "testit"
		}
	}
	fn build(&self, _br:&mut putki_inki::BuildRecord, input:&mut TestValues) -> Result<(), putki_inki::PutkiError> {		
		println!("building input v1={} v2={}", input.value1, input.value2);
		input.value1 = input.value1 + 1000;
		input.value2 = input.value2 + 2000;
		return Ok(());
	}
}

impl putki_inki::Builder<Pointer> for PointerBuilder {
	fn desc(&self) -> putki_inki::BuilderDesc {
		putki_inki::BuilderDesc {
			description: "pointer"
		}
	}
	fn build(&self, br:&mut putki_inki::BuildRecord, input:&mut Pointer) -> Result<(), putki_inki::PutkiError> {		
		let ptr = br.create_object("n", Pointer {
			next: putki_inki::Ptr::null(),
			contained: TestValues {
				value1 : 222,
				value2 : 333
			}
		});
		if /*ptr.get_target_path().is_none() && */br.get_path().len() < 30 {
			input.next = ptr;
		}
		return Ok(());
	}
}

impl putki_inki::ParseFromKV for TestValues {
	fn parse(kv : &putki_inki::lexer::LexedKv, _pctx: &Arc<putki_inki::InkiResolver>) -> Self {
		return Self {
			value1 : putki_inki::lexer::get_int(kv.get("Value1"), 0),
			value2 : putki_inki::lexer::get_int(kv.get("Value2"), 0)
		}
	}
}

impl putki_inki::WriteAsText for TestValues {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Value1", &self.value1, false)?;
		output.write_field("Value2", &self.value2, true)
	}
}

impl putki_inki::BinSaver for TestValues {
	fn write(&self, data: &mut Vec<u8>, _refwriter: &putki_inki::PackageRefs) -> Result<(), PutkiError> {
		self.value1.write(data);
		self.value2.write(data);
		Ok(())
	}	
}

impl putki_inki::ParseFromKV for Multi {
	fn parse(kv : &putki_inki::lexer::LexedKv, _pctx: &Arc<putki_inki::InkiResolver>) -> Self {
		return Self {
			contained : putki_inki::lexer::get_object(kv.get("Contained")).map(|v| { putki_inki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default()
		}
	}
}

impl putki_inki::BinSaver for Multi {
	fn write(&self, data: &mut Vec<u8>, refwriter: &putki_inki::PackageRefs) -> Result<(), PutkiError> {
		self.contained.write(data, refwriter);
		Ok(())
	}	
}

impl putki_inki::WriteAsText for Multi {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Contained", &self.contained, false)
	}
}

impl putki_inki::ParseFromKV for Pointer {
	fn parse(kv : &putki_inki::lexer::LexedKv, _pctx: &Arc<putki_inki::InkiResolver>) -> Self {
		return Self {
			contained : putki_inki::lexer::get_object(kv.get("Contained")).map(|v| { putki_inki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default(),
			next: kv.get("Next").map(|v| { putki_inki::ptr_from_data(_pctx, v) }).unwrap_or_default()
		}
	}
}

impl putki_inki::BinSaver for Pointer {
	fn write(&self, data: &mut Vec<u8>, refwriter: &putki_inki::PackageRefs) -> Result<(), PutkiError> {
		self.contained.write(data, refwriter)?;
		self.next.write(data, refwriter)?;
		Ok(())
	}	
}

impl putki_inki::WriteAsText for Pointer {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Contained", &self.contained, false)?;
		output.write_field("Next", &self.next, true)
	}
}

impl putki_inki::InkiObj for TestValues { }
impl putki_inki::InkiObj for Multi { }
impl putki_inki::InkiObj for Pointer { }

struct ReadFromVec {
	data: Vec<u8>
}

impl outki::PackageRandomAccess for ReadFromVec
{
   fn read_chunk(&self, begin:usize, end:usize, f:&mut dyn FnMut(outki::OutkiResult<&[u8]>) -> outki::OutkiResult<()>) -> outki::OutkiResult<()> {
        (*f)(Ok(&self.data[begin..end]))
    }
}

#[test]
fn test_pipeline() {
	let la = Arc::new(putki_inki::LoadAll::from_txty_data(r#"
		@TestValues tv0 {
			Value1: 123,
			Value2: 456,
		}
		@Multi multi {
			Contained: {
				Value1: 321
				Value2: 654
			}
		}
		@Pointer ptr {
			Contained: {
				Value1: 1
				Value2: 2
			}
			Next: ptr2
		}
		@Pointer ptr2 {
			Contained: {
				Value1: 2
				Value2: 3
			}			
		}		
	"#));	

	let desc = putki_inki::PipelineDesc::new(la.clone(), Path::new("."))
			.add_builder(TestValueBuilder{ })
			.add_builder(PointerBuilder{ });

	let pipeline = Arc::new(putki_inki::Pipeline::new(desc));

	pipeline.build_as::<Multi>("multi");
	pipeline.build_as::<Pointer>("ptr");

	let mut thr = Vec::new();
	for _i in 0..4  {
		let pl = pipeline.clone();		
		thr.push(thread::spawn(move || {
			let mut k = 0;
			while pl.take() { k = k + 1; if k > 100 { panic!("Pipeline never finished!") } }
		}));
	}

	for x in thr {
		x.join().ok();
	}

	let p2 = pipeline.clone();
	thread::spawn(move || {
		let recs = p2.peek_build_records().unwrap();
		for (ref k, ref v) in recs.iter() {						
			let mut r = String::new();
			if let Some(bo) = v.built_object() {
				bo.write_object(&mut r);
			} else {
				panic!("Failed somehow with {}", k);
			}
		}
	}).join().ok();

	println!("Building package");
	let mut rcp = putki_inki::PackageRecipe::new();
	rcp.add_object(&(*pipeline), "ptr", true);
	rcp.add_object(&(*pipeline), "multi", true);

	let data = putki_inki::write_package(&(*pipeline), &rcp).expect("It should have worked");

	// Golden fixture: the same bytes are parsed by the C# test in
	// tests/csharp/PackageVectors.cs, so the two implementations cannot drift.
	// Regenerate with PUTKI_REGEN_FIXTURES=1 cargo test after a format change,
	// and re-run the C# test against the new file.
	{
		let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
			.join("../../tests/fixtures/sample.pkg");
		if std::env::var("PUTKI_REGEN_FIXTURES").is_ok() {
			std::fs::create_dir_all(fixture.parent().unwrap()).unwrap();
			std::fs::write(&fixture, &data).unwrap();
		} else {
			let want = std::fs::read(&fixture).expect("missing tests/fixtures/sample.pkg");
			assert_eq!(data, want, "package format changed; regenerate the fixture and update the C# reader");
		}
	}

	// Slot and type indices are assigned by iteration order, so the package
	// layout must not depend on hash seeding: rebuilding the same recipe has to
	// produce identical bytes or content hashing and incremental builds break.
	// Each new HashSet in a thread gets a distinct RandomState, so an unordered
	// container shows up within a single process.
	for _ in 0..16 {
		let mut again_rcp = putki_inki::PackageRecipe::new();
		again_rcp.add_object(&(*pipeline), "ptr", true);
		again_rcp.add_object(&(*pipeline), "multi", true);
		let again = putki_inki::write_package(&(*pipeline), &again_rcp).expect("It should have worked");
		assert_eq!(data, again, "write_package output is not deterministic");
	}

	let mfest;
	{
		let mut slice = data.as_slice();
		mfest = outki::PackageManifest::parse(&mut slice).expect("Could not parse manifest");
	}
	
    let mut mgr = outki::BinPackageManager::new();
	let rfv = ReadFromVec { data: data };
    mgr.insert(outki::Package::new(mfest, Box::new(rfv)));

	{
		let obj_maybe = mgr.resolve::<Multi>("multi");
		assert_eq!(obj_maybe.is_ok(), true);  	
		let obj = obj_maybe.unwrap();
		assert_eq!(obj.contained.value1, 321 + 1000);
		assert_eq!(obj.contained.value2, 654 + 2000);
	}

	{
		// Asking for the wrong type must be refused, not reinterpreted.
		match mgr.resolve::<Multi>("ptr") {
			Err(outki::OutkiError::TypeMismatch { wanted, found, .. }) => {
				assert_eq!(wanted, "Multi");
				assert_eq!(found, "Pointer");
			}
			Err(e) => panic!("expected TypeMismatch, got {:?}", e),
			Ok(_) => panic!("resolving a Pointer slot as Multi should have failed"),
		}
	}

	{
		let obj_maybe = mgr.resolve::<PointerOutki>("ptr");
		assert_eq!(obj_maybe.is_ok(), true);  	
		let obj = obj_maybe.unwrap();
		assert_eq!(obj.contained.value1, 1 + 1000);
		assert_eq!(obj.contained.value2, 2 + 2000);
		assert_eq!(obj.next.unwrap().contained.value1, 222 + 1000);
		assert_eq!(obj.next.unwrap().contained.value2, 333 + 2000);
	}	
	
}

// Normative escaping vectors, mirroring the table in doc/text-format.md.
// (raw value, encoded form). Keep the two in sync; C# and JS should be checked
// against the same table when they are brought in line.
const ESCAPE_VECTORS: &[(&str, &str)] = &[
	("plain",            "\"plain\""),
	("with \"quote\"",   "\"with \\\"quote\\\"\""),
	("back\\slash",      "\"back\\\\slash\""),
	("trailing\\",       "\"trailing\\\\\""),
	("two\\\\slashes",   "\"two\\\\\\\\slashes\""),
	("line\nbreak",      "\"line\\nbreak\""),
	("tab\there",        "\"tab\\there\""),
	("quote\"then\\",    "\"quote\\\"then\\\\\""),
	("\\\"",             "\"\\\\\\\"\""),
	("unicode: \u{e5}\u{e4}\u{f6}", "\"unicode: \u{e5}\u{e4}\u{f6}\""),
	("",                 "\"\""),
];

fn lex_one_string(encoded: &str) -> Option<String> {
	let src = format!("@T o {{ f: {} }}", encoded);
	match putki_inki::lex_file(&src).get("o") {
		Some(putki_inki::LexedData::Object { kv, .. }) => match kv.get("f") {
			Some(putki_inki::LexedData::StringLiteral(s)) => Some(s.clone()),
			_ => None,
		},
		_ => None,
	}
}

#[test]
fn test_string_escapes_encode() {
	for (raw, encoded) in ESCAPE_VECTORS {
		assert_eq!(&putki_inki::escape_string(raw), encoded, "encoding {:?}", raw);
	}
}

#[test]
fn test_string_escapes_decode() {
	for (raw, encoded) in ESCAPE_VECTORS {
		assert_eq!(lex_one_string(encoded).as_deref(), Some(*raw), "decoding {:?}", encoded);
	}
}

#[test]
fn test_string_escapes_roundtrip() {
	for (raw, _) in ESCAPE_VECTORS {
		let once = putki_inki::escape_string(raw);
		assert_eq!(lex_one_string(&once).as_deref(), Some(*raw), "roundtrip {:?}", raw);
	}
}

#[test]
fn test_legacy_escapes_accepted_but_never_emitted() {
	// \uXXXX carries bytes of the original UTF-8, so a run reassembles into one
	// char: U+00E5 is C3 A5 in UTF-8.
	assert_eq!(lex_one_string("\"\\u00c3\\u00a5\"").as_deref(), Some("\u{e5}"));
	assert_eq!(lex_one_string("\"pre \\u00c3\\u00a5 post\"").as_deref(), Some("pre \u{e5} post"));
	// A lone \u00e5 is the Latin-1 byte, not UTF-8, so it is malformed legacy
	// data. Reject it rather than emit a replacement character.
	assert_eq!(lex_one_string("\"\\u00e5\""), None);
	// Newline is newline: a legacy \r escape folds into one.
	assert_eq!(lex_one_string("\"a\\rb\"").as_deref(), Some("a\nb"));
	// ...and neither form is ever produced again.
	assert!(!putki_inki::escape_string("\u{e5}\r\n\r").contains("\\u"));
	assert!(!putki_inki::escape_string("\u{e5}\r\n\r").contains("\\r"));
	// CRLF and lone CR both normalise to a single \n.
	assert_eq!(putki_inki::escape_string("a\r\nb\rc"), "\"a\\nb\\nc\"");
}

#[test]
fn test_invalid_escape_is_rejected_not_guessed() {
	assert_eq!(lex_one_string("\"bad \\q escape\""), None);
	assert_eq!(lex_one_string("\"short \\u00\""), None);
	assert_eq!(lex_one_string("\"unterminated"), None);
}
