#![allow(unused_imports)]
#![allow(unused_must_use)]
extern crate putki;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::any;
use std::any::TypeId;
use std::thread;
use std::path::Path;

use putki::PutkiError;
use putki::FieldWriter;
use putki::Ptr;
use putki::BinWriter;
use putki::outki as outki;
use outki::PackageManifest;
use outki::BinReader;

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
	next: putki::Ptr<Pointer>
}

struct PointerOutki {
	contained: TestValues,
	next: outki::NullablePtr<PointerOutki>
}

impl putki::TypeDescriptor for TestValues {
	const TAG : &'static str = "TestValues";
}

impl putki::TypeDescriptor for Multi {
	const TAG : &'static str = "Multi";
}

impl putki::TypeDescriptor for Pointer {
	const TAG : &'static str = "Pointer";
}

impl putki::TypeDescriptor for PointerOutki {
	const TAG : &'static str = "Pointer";
}
  
impl putki::BuildFields for TestValues { }

impl putki::BuildFields for Multi {
	fn build_fields(&mut self, pipeline:&putki::Pipeline, br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {
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

impl putki::BuildFields for Pointer {
	fn build_fields(&mut self, pipeline:&putki::Pipeline, br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {
		pipeline.build(br, &mut self.contained);
		Ok(())
	}
}

impl putki::BuildCandidate for TestValues {
    fn as_any_ref(&mut self) -> &mut any::Any { return self; }    
    fn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) -> Result<(), putki::PutkiError> { p.build(br, self) }
	fn scan_deps(&self, _p:&putki::Pipeline, _br: &mut putki::BuildRecord) { }
}

impl putki::BuildCandidate for Multi {
    fn as_any_ref(&mut self) -> &mut any::Any { return self; }    
    fn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) -> Result<(), putki::PutkiError> { p.build(br, self) }
	fn scan_deps(&self, _p:&putki::Pipeline, _br: &mut putki::BuildRecord) { }
}

impl putki::BuildCandidate for Pointer {
    fn as_any_ref(&mut self) -> &mut any::Any { return self; }    
    fn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) -> Result<(), putki::PutkiError> { p.build(br, self) }
	fn scan_deps(&self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { 
		p.add_output_dependency(br, &self.next);
	}
}

struct TestValueBuilder { }
struct PointerBuilder { }

impl putki::Builder<TestValues> for TestValueBuilder {
	fn desc(&self) -> putki::BuilderDesc {
		putki::BuilderDesc {
			description: "testit"
		}
	}
	fn build(&self, _br:&mut putki::BuildRecord, input:&mut TestValues) -> Result<(), putki::PutkiError> {		
		println!("building input v1={} v2={}", input.value1, input.value2);
		input.value1 = input.value1 + 1000;
		input.value2 = input.value2 + 2000;
		return Ok(());
	}
}

impl putki::Builder<Pointer> for PointerBuilder {
	fn desc(&self) -> putki::BuilderDesc {
		putki::BuilderDesc {
			description: "pointer"
		}
	}
	fn build(&self, br:&mut putki::BuildRecord, input:&mut Pointer) -> Result<(), putki::PutkiError> {		
		let ptr = br.create_object("n", Pointer {
			next: putki::Ptr::null(),
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

impl putki::ParseFromKV for TestValues {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &Arc<putki::InkiResolver>) -> Self {
		return Self {
			value1 : putki::lexer::get_int(kv.get("Value1"), 0),
			value2 : putki::lexer::get_int(kv.get("Value2"), 0)
		}
	}
}

impl putki::WriteAsText for TestValues {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Value1", &self.value1, false)?;
		output.write_field("Value2", &self.value2, true)
	}
}

impl putki::BinSaver for TestValues {
	fn write(&self, data: &mut Vec<u8>, _refwriter: &putki::PackageRefs) -> Result<(), PutkiError> {
		self.value1.write(data);
		self.value2.write(data);
		Ok(())
	}	
}

impl putki::ParseFromKV for Multi {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &Arc<putki::InkiResolver>) -> Self {
		return Self {
			contained : putki::lexer::get_object(kv.get("Contained")).map(|v| { putki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default()
		}
	}
}

impl putki::BinSaver for Multi {
	fn write(&self, data: &mut Vec<u8>, refwriter: &putki::PackageRefs) -> Result<(), PutkiError> {
		self.contained.write(data, refwriter);
		Ok(())
	}	
}

impl putki::WriteAsText for Multi {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Contained", &self.contained, false)
	}
}

impl putki::ParseFromKV for Pointer {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &Arc<putki::InkiResolver>) -> Self {
		return Self {
			contained : putki::lexer::get_object(kv.get("Contained")).map(|v| { putki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default(),
			next: kv.get("Next").map(|v| { putki::ptr_from_data(_pctx, v) }).unwrap_or_default()
		}
	}
}

impl putki::BinSaver for Pointer {
	fn write(&self, data: &mut Vec<u8>, refwriter: &putki::PackageRefs) -> Result<(), PutkiError> {
		self.contained.write(data, refwriter)?;
		self.next.write(data, refwriter)?;
		Ok(())
	}	
}

impl putki::WriteAsText for Pointer {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Contained", &self.contained, false)?;
		output.write_field("Next", &self.next, true)
	}
}

impl putki::InkiObj for TestValues { }
impl putki::InkiObj for Multi { }
impl putki::InkiObj for Pointer { }

struct ReadFromVec {
	data: Vec<u8>
}

impl outki::PackageRandomAccess for ReadFromVec
{
    fn read_chunk(&self, begin:usize, into:&mut [u8]) -> outki::OutkiResult<()> {
        for i in 0..into.len() {
            into[i] = self.data[begin + i];
        }
        Ok(())
    }
}

#[test]
fn test_pipeline() {
	let la = Arc::new(putki::LoadAll::from_txty_data(r#"
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

	let desc = putki::PipelineDesc::new(la.clone(), Path::new("."))
			.add_builder(TestValueBuilder{ })
			.add_builder(PointerBuilder{ });

	let pipeline = Arc::new(putki::Pipeline::new(desc));

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
	let mut rcp = putki::PackageRecipe::new();
	rcp.add_object(&(*pipeline), "ptr", true);
	rcp.add_object(&(*pipeline), "multi", true);

	let data = putki::write_package(&(*pipeline), &rcp).expect("It should have worked");

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
		let obj_maybe = mgr.resolve::<PointerOutki>("ptr");
		assert_eq!(obj_maybe.is_ok(), true);  	
		let obj = obj_maybe.unwrap();
		assert_eq!(obj.contained.value1, 1 + 1000);
		assert_eq!(obj.contained.value2, 2 + 2000);
		assert_eq!(obj.next.unwrap().contained.value1, 222 + 1000);
		assert_eq!(obj.next.unwrap().contained.value2, 333 + 2000);
	}	
	
}
