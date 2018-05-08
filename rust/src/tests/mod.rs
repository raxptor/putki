use inki::*;
use shared;
use inki::source;
use std::rc::Rc;
use std::any::Any;

#[cfg(test)]
mod outki;

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

impl ParseFromKV for PointedTo {
	fn parse(kv : &lexer::LexedKv, _pctx: &InkiPtrContext, _res:&source::InkiResolver) -> Self {
		return Self {
			value : lexer::get_int(&kv, "Value", 0)
		}
	}
}

impl ParseFromKV for PtrStruct1 {
	fn parse(kv : &lexer::LexedKv, _pctx: &InkiPtrContext, _res:&InkiResolver) -> Self {
		return Self {
			ptr : Ptr::new(_pctx.clone(), lexer::get_string(&kv, "Ptr", "").as_str())
		}
	}
}

impl shared::PutkiTypeCast for PointedTo {
	fn rc_convert(src: Rc<Any>) -> Option<Rc<PointedTo>> {
		return src.downcast().ok();
	}
}

impl shared::PutkiTypeCast for PtrStruct1 {
	fn rc_convert(src: Rc<Any>) -> Option<Rc<PtrStruct1>> {
		return src.downcast().ok();
	}
}

struct DumbResolver {
	db : loadall::LoadAll 
}

impl shared::Resolver<InkiPtrContext> for DumbResolver
{
	fn load(&self, pctx: &InkiPtrContext, path:&str) -> Option<Rc<Any>>
	{
		return self.db.load(path).and_then(|res| {
			match res.0
			{
				"PointedTo" => return Some(Rc::new(PointedTo::parse(res.1, pctx, self)) as Rc<Any>),
				"PtrStruct1" => return Some(Rc::new(PtrStruct1::parse(res.1, pctx, self)) as Rc<Any>),
				_ => return None
			}
		});
	}
}

#[test]
fn test_ptr_1() {
	let la = loadall::LoadAll::from_txty_data(r#"
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
	"#);	
	let dr = Rc::new(DumbResolver {
		db: la
	}) as Rc<shared::Resolver<InkiPtrContext>>;
	let ctx = InkiPtrContext {
		tracker: None,
		source: dr.clone()
	};
	if let ResolveStatus::Resolved(pto) = resolve_from::<PointedTo>(&dr, &ctx, "main") {
		assert_eq!(pto.value, 123);
	} else {
		panic!("Could not resolve main");
	}
	if let ResolveStatus::Resolved(pto) = resolve_from::<PtrStruct1>(&dr, &ctx, "ptr1") {
		if let Some(pto) = pto.ptr.resolve() {
			assert_eq!(pto.value, 321);
		} else {
			panic!("Could not resolve pointer in ptr1.")
		}
	} else {
		panic!("Could not resolve main");
	}	
	if let ResolveStatus::Resolved(pto) = resolve_from::<PtrStruct1>(&dr, &ctx, "ptr2") {
		println!("ptr2.ptr={:?}", pto.ptr);
		assert!(pto.ptr.resolve().is_none());
	} else {
		panic!("Could not resolve ptr2");
	}	
}
