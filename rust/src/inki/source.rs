use mixki::lexer;
use std::rc::Rc;
use std::any::Any;
use std::collections::HashMap;

pub enum ResolveStatus<T>
{
    Resolved(Rc<T>),
    Failed,
    Null
}

pub trait ObjectLoader {
	fn load(&self, path: &str) -> Option<(&str, &lexer::LexedKv)>;
}

pub trait Resolver {
	fn load(&self, pctx: &PtrContext, path:&str) -> Option<Rc<Any>>;
}

pub trait Tracker
{
    fn follow(&self, path:&str);
}

#[derive(Clone)]
pub struct PtrContext
{
    pub tracker: Option<Rc<Tracker>>,
    pub source: Rc<Resolver>
}

pub trait ParseFromKV where Self:Sized {
	fn parse(kv : &lexer::LexedKv, pctx: &PtrContext, res:&Resolver) -> Self;
}

pub trait PutkiTypeCast where Self : Sized + 'static {
	fn rc_convert(src:Rc<Any>) -> Option<Rc<Self>> { return None; /*return src.downcast().ok();*/ }
}

pub fn resolve_from<T>(src:&Rc<Resolver>, pctx: &PtrContext, path:&str) -> ResolveStatus<T> where T : ParseFromKV + PutkiTypeCast
{	
	if path.is_empty() {
		return ResolveStatus::Null;
	}

	// Here is where it will be necessary to deal with the subtypes mess.
	let k = src.load(pctx, path).and_then(|rc| {
		return PutkiTypeCast::rc_convert(rc);
	});

	if let Some(r) = k {
		return ResolveStatus::Resolved(r);
	}

	return ResolveStatus::Failed;
}
