#![feature(get_type_id)]
#![allow(unused_imports)]
extern crate putki;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::any;
use std::any::TypeId;
use std::thread;

#[derive(Debug, Clone, Default)]
struct TestValues {
	value1: i32,
	value2: i32
}

#[derive(Debug, Clone)]
struct Multi {
	contained: TestValues
}

#[derive(Debug, Clone, Default)]
struct Pointer {	
	contained: TestValues,
	next: putki::Ptr<Pointer>
}

impl putki::InkiTypeDescriptor for TestValues {
	const TAG : &'static str = "TestValues";
	type OutkiType = ();
}

impl putki::InkiTypeDescriptor for Multi {
	const TAG : &'static str = "Multi";
	type OutkiType = ();
}

impl putki::InkiTypeDescriptor for Pointer {
	const TAG : &'static str = "Pointer";
	type OutkiType = ();
}
  
impl putki::BuildFields for TestValues { }

impl putki::BuildFields for Multi {
	fn build_fields(&mut self, pipeline:&putki::Pipeline, br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {
		pipeline.build(br, &mut self.contained);
		Ok(())
	}
}

impl putki::BuildFields for Pointer {
	fn build_fields(&mut self, pipeline:&putki::Pipeline, br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {
		pipeline.build(br, &mut self.contained);
		Ok(())
	}
}

impl putki::BuildCandidate for TestValues {
    fn as_any_ref(&mut self) -> &mut any::Any { return self; }    
    fn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { p.build(br, self); }
	fn scan_deps(&self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { }
}

impl putki::BuildCandidate for Multi {
    fn as_any_ref(&mut self) -> &mut any::Any { return self; }    
    fn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { p.build(br, self); }
	fn scan_deps(&self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { }
}

impl putki::BuildCandidate for Pointer {
    fn as_any_ref(&mut self) -> &mut any::Any { return self; }    
    fn build(&mut self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { p.build(br, self); }
	fn scan_deps(&self, p:&putki::Pipeline, br: &mut putki::BuildRecord) { 
		println!("Adding output dependencies...");
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
	fn build(&self, input:&mut TestValues) -> Result<(), putki::PutkiError> {		
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
	fn build2(&self, br:&mut putki::BuildRecord, input:&mut Pointer) -> Result<(), putki::PutkiError> {		
		println!("building pointer!");		
		let ptr = br.create_object("neue1", Pointer {
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
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &Arc<putki::InkiPtrContext>) -> Self {
		return Self {
			value1 : putki::lexer::get_int(kv.get("Value1"), 0),
			value2 : putki::lexer::get_int(kv.get("Value2"), 0)
		}
	}
}

impl putki::ParseFromKV for Multi {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &Arc<putki::InkiPtrContext>) -> Self {
		return Self {
			contained : putki::lexer::get_object(kv.get("Contained")).map(|v| { putki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default()
		}
	}
}

impl putki::ParseFromKV for Pointer {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &Arc<putki::InkiPtrContext>) -> Self {
		return Self {
			contained : putki::lexer::get_object(kv.get("Contained")).map(|v| { putki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default(),
			next: kv.get("Next").map(|v| { putki::ptr_from_data(_pctx, v) }).unwrap_or_default()
		}
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

	let desc = putki::PipelineDesc::new(la.clone())
	          .add_builder(TestValueBuilder{ })
			  .add_builder(PointerBuilder{ });

	let pipeline = Arc::new(putki::Pipeline::new(desc));

	pipeline.build_as::<Multi>("multi");
	pipeline.build_as::<Pointer>("ptr");

	let mut thr = Vec::new();
	for _i in 0..3 {
		let pl = pipeline.clone();		
		thr.push(thread::spawn(move || {
			let mut k = 0;
			while pl.take() { k = k + 1; if k > 100 { panic!("Pipeline never finished!") } }
			println!("thread took {} items.", k);
		}));
	}
	for x in thr {
		x.join().ok();
	}
}
