use inki::lexer;
use shared;
use std::rc::Rc;
use std::any::Any;

pub enum ResolveStatus<T>
{
    Resolved(Rc<T>),
    Failed,
    Null
}

pub trait SourceLoader {
    fn load(&self, path: &str) -> Option<Rc<Any>>;
}

pub trait ObjectLoader {
	fn load(&self, path: &str) -> Option<(&str, &lexer::LexedKv)>;
}

pub type InkiResolver = shared::Resolver<InkiPtrContext>;

pub trait Tracker {
    fn follow(&self, path:&str);
}

#[derive(Clone)]
pub struct InkiPtrContext
{
    pub tracker: Option<Rc<Tracker>>,
    pub source: Rc<InkiResolver>
}

pub trait ParseFromKV where Self:Sized {
	fn parse(kv : &lexer::LexedKv, pctx: &InkiPtrContext, res:&InkiResolver) -> Self;
}

pub fn resolve_from<T>(src:&Rc<InkiResolver>, pctx: &InkiPtrContext, path:&str) -> ResolveStatus<T> where T : ParseFromKV + shared::PutkiTypeCast
{	
	if path.is_empty() {
		return ResolveStatus::Null;
	}

	// Here is where it will be necessary to deal with the subtypes mess.
	let k = src.load(pctx, path).and_then(|rc| {
		return shared::PutkiTypeCast::rc_convert(rc);
	});

	if let Some(r) = k {
		return ResolveStatus::Resolved(r);
	}

	return ResolveStatus::Failed;
}
