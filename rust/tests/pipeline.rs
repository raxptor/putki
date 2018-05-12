#![feature(get_type_id)]
#![allow(unused_imports)]
extern crate putki;

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::any;
use std::any::TypeId;

#[derive(Debug, Clone, Default)]
struct TestValues {
	value1: i32,
	value2: i32
}

#[derive(Debug, Clone)]
struct Multi {
	contained: TestValues
}

impl putki::InkiTypeDescriptor for TestValues {
	const TAG : &'static str = "TestValues";
	type OutkiType = ();
}

impl putki::InkiTypeDescriptor for Multi {
	const TAG : &'static str = "Multi";
	type OutkiType = ();
}
 
impl putki::BuildFields for TestValues { }

impl putki::BuildFields for Multi {
	fn build(&mut self, pipeline:&putki::Pipeline, br:&mut putki::BuildRecord) -> Result<(), putki::PutkiError> {
		pipeline.build_field(br, &mut self.contained)?;
		Ok(())
	}
}

struct TestValueBuilder { }

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

impl putki::ParseFromKV for TestValues {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &putki::InkiPtrContext) -> Self {
		return Self {
			value1 : putki::lexer::get_int(kv.get("Value1"), 0),
			value2 : putki::lexer::get_int(kv.get("Value2"), 0)
		}
	}
}

impl putki::ParseFromKV for Multi {
	fn parse(kv : &putki::lexer::LexedKv, _pctx: &putki::InkiPtrContext) -> Self {
		return Self {
			contained : putki::lexer::get_object(kv.get("Contained")).map(|v| { putki::ParseFromKV::parse(v.0, &_pctx) }).unwrap_or_default()
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
	"#));	

	let desc = putki::PipelineDesc::new(la.clone())
	          .add_builder(TestValueBuilder{ });

	let pipeline = putki::Pipeline::new(desc);

	pipeline.build_as::<Multi>("multi");

	let ctx = putki::InkiPtrContext {
		tracker: None,
		source: Rc::new(putki::InkiResolver::new(la.clone())),
	};
	{
		let p : putki::Ptr<TestValues> = putki::Ptr::new(ctx.clone(), "tv0");
		let mut o1:TestValues = (*p.unwrap()).clone();
		let br = pipeline.build(&mut o1);
		assert!(br.is_ok());
		assert_eq!(o1.value1, 1123);
		assert_eq!(o1.value2, 2456);
	}
	{
		let p : putki::Ptr<Multi> = putki::Ptr::new(ctx, "multi");
		let mut o1:Multi = (*p.unwrap()).clone();
		let br = pipeline.build(&mut o1);
		assert!(br.is_ok());
		assert_eq!(o1.contained.value1, 1321);
		assert_eq!(o1.contained.value2, 2654);
	}
}
