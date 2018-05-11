use inki::*;
use shared;
use inki::source;
use std::rc::Rc;
use std::boxed::Box;
use std::any::Any;

#[cfg(test)]
mod outki;
#[cfg(test)]
mod outki_enum;

#[test]
fn test_txty_1() {
	loadall::LoadAll::from_txty_data(r#"
		@MainObject main/obj {
			Stuff: @SubObject id-name {
				Value: "128"
			}
		}	
	"#);
}

#[derive(Debug)]
struct PointedTo {
	value: i32
}

#[derive(Debug)]
struct PtrStruct1 {
    pub ptr: Ptr<PointedTo>
}

impl source::InkiTypeDescriptor for PointedTo {
	const TAG : &'static str = "PointedTo";
	type OutkiType = ();
}

impl source::InkiTypeDescriptor for PtrStruct1 {
	const TAG : &'static str = "PtrStruct1";
	type OutkiType = ();
}

impl ParseFromKV for PointedTo {
	fn parse(kv : &lexer::LexedKv, _pctx: &InkiPtrContext) -> Self {
		return Self {
			value : lexer::get_int(kv.get("Value"), 0)
		}
	}
}

impl ParseFromKV for PtrStruct1 {
	fn parse(kv : &lexer::LexedKv, _pctx: &InkiPtrContext) -> Self {
		return Self {
			ptr : Ptr::new(_pctx.clone(), lexer::get_string(kv.get("Ptr"), "").as_str())
		}
	}
}

#[test]
fn test_ptr_1() {
	let la = Box::new(loadall::LoadAll::from_txty_data(r#"
		@PointedTo main {
			Value: 123
		}
		@PointedTo pto1 {
			Value: 321
		}
		@PtrStruct1 ptr1 {
			Ptr: "pto1"
		}
		@PtrStruct1 ptr2 {
			Ptr: ""
		}	
	"#));	

	let resolver = Rc::new(InkiResolver::new(la));
	let ctx = InkiPtrContext {
		tracker: None,
		source: resolver.clone()
	};
	if let ResolveStatus::Resolved(pto) = resolve_from::<PointedTo>(&ctx, "main") {
		assert_eq!(pto.value, 123);
	} else {
		panic!("Could not resolve main");
	}
	if let ResolveStatus::Resolved(pto) = resolve_from::<PtrStruct1>(&ctx, "ptr1") {
		if let Some(pto) = pto.ptr.resolve() {
			assert_eq!(pto.value, 321);
		} else {
			panic!("Could not resolve pointer in ptr1.")
		}
	} else {
		panic!("Could not resolve main");
	}	
	if let ResolveStatus::Resolved(pto) = resolve_from::<PtrStruct1>(&ctx, "ptr2") {
		println!("ptr2.ptr={:?}", pto.ptr);
		assert!(pto.ptr.resolve().is_none());
	} else {
		panic!("Could not resolve ptr2");
	}	
}
