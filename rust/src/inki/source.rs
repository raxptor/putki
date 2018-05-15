use inki::lexer;
use std::rc::Rc;
use std::sync::Arc;
use shared::TypeDescriptor;

pub enum ResolveStatus<T> {
    Resolved(Rc<T>),
    Failed,
    Null
}

pub trait ObjectLoader where Self : Sync + Send {
	fn load(&self, path: &str) -> Option<(&str, &lexer::LexedKv)>;
}

pub trait ParseFromKV where Self:Sized + TypeDescriptor + Clone {
	fn parse(kv : &lexer::LexedKv, pctx: &Arc<InkiPtrContext>) -> Self;
	fn parse_with_type(kv : &lexer::LexedKv, pctx: &Arc<InkiPtrContext>, type_name:&str) -> Self {
		if <Self as TypeDescriptor>::TAG != type_name {
			println!("Mismatched type in parse_with_type {} vs {}", type_name, <Self as TypeDescriptor>::TAG);
		}		
		<Self as ParseFromKV>::parse(kv, pctx)
	}
}

pub trait Tracker where Self : Send + Sync {
    fn follow(&self, path:&str);
}

#[derive(Clone)]
pub struct InkiPtrContext
{
    pub tracker: Option<Arc<Tracker>>,
    pub source: Arc<InkiResolver>
}

pub struct InkiResolver {
	loader: Arc<ObjectLoader>
}

impl InkiResolver {
	pub fn new(loader:Arc<ObjectLoader>) -> Self {
		Self {
			loader: loader
		}
	}
}

impl InkiResolver {
	pub fn resolve<T>(ctx:&Arc<InkiPtrContext>, path:&str) -> ResolveStatus<T> where T : ParseFromKV {
		match ctx.source.loader.load(path)
		{
			Some((type_name, data)) => return ResolveStatus::Resolved(Rc::new(<T as ParseFromKV>::parse_with_type(data, ctx, type_name))),
			_ => return ResolveStatus::Failed
		}
	}
}

pub fn resolve_from<T>(ctx: &Arc<InkiPtrContext>, path:&str) -> ResolveStatus<T> where T : ParseFromKV + 'static
{	
	return InkiResolver::resolve(ctx, path);
}
