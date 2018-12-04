use inki::*;
use std::sync::Arc;
use shared;
use shared::PutkiError;
use inki::source::FieldWriter;
use inki::source::WriteAsText;
use pipeline;

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

#[derive(Debug, Clone)]
struct PointedTo {
	value: i32
}

#[derive(Debug, Clone)]
struct PtrStruct1 {
    pub ptr: Ptr<PointedTo>
}

impl shared::TypeDescriptor for PointedTo {
	const TAG : &'static str = "PointedTo";
}

impl shared::TypeDescriptor for PtrStruct1 {
	const TAG : &'static str = "PtrStruct1";
}

impl WriteAsText for PointedTo {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Ptr", &self.value, false)
	}
}

impl WriteAsText for PtrStruct1 {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.write_field("Ptr", &self.ptr, false)
	}
}

impl ParseFromKV for PointedTo {
	fn parse(kv : &lexer::LexedKv, _pctx: &Arc<InkiResolver>) -> Self {
		return Self {
			value : lexer::get_int(kv.get("Value"), 0)
		}
	}
}

impl ParseFromKV for PtrStruct1 {
	fn parse(kv : &lexer::LexedKv, _pctx: &Arc<InkiResolver>) -> Self {
		return Self {
			ptr : Ptr::new(_pctx.clone(), lexer::get_string(kv.get("Ptr"), "").as_str())
		}
	}
}

#[test]
fn test_ptr_1() {
	let la = Arc::new(loadall::LoadAll::from_txty_data(r#"
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

	let resolver = Arc::new(InkiResolver::new(la));
	if let ResolveStatus::Resolved(pto) = resolve_from::<PointedTo>(&resolver, "main") {
		assert_eq!(pto.value, 123);
	} else {
		panic!("Could not resolve main");
	}
	if let ResolveStatus::Resolved(pto) = resolve_from::<PtrStruct1>(&resolver, "ptr1") {
		assert_eq!(pto.ptr.unwrap().value, 321);
	} else {
		panic!("Could not resolve main");
	}	
	if let ResolveStatus::Resolved(pto) = resolve_from::<PtrStruct1>(&resolver, "ptr2") {
		println!("ptr2.ptr={:?}", pto.ptr);
		assert!(pto.ptr.resolve_notrack().is_none());
	} else {
		panic!("Could not resolve ptr2");
	}	
}
