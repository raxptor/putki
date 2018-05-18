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
	fn parse(kv : &lexer::LexedKv, resolver: &Arc<InkiResolver>) -> Self;
	fn parse_with_type(kv : &lexer::LexedKv, resolver: &Arc<InkiResolver>, type_name:&str) -> Self {
		if type_name.len() > 0 && <Self as TypeDescriptor>::TAG != type_name {
			println!("Mismatched type in parse_with_type {} vs {}", type_name, <Self as TypeDescriptor>::TAG);
		}		
		<Self as ParseFromKV>::parse(kv, resolver)
	}
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
	pub fn resolve<T>(resolver:&Arc<InkiResolver>, path:&str) -> ResolveStatus<T> where T : ParseFromKV {
		match resolver.loader.load(path)
		{
			Some((type_name, data)) => return ResolveStatus::Resolved(Rc::new(<T as ParseFromKV>::parse_with_type(data, resolver, type_name))),
			_ => return ResolveStatus::Failed
		}
	}
}

pub fn resolve_from<T>(resolver: &Arc<InkiResolver>, path:&str) -> ResolveStatus<T> where T : ParseFromKV + 'static
{	
	return InkiResolver::resolve(resolver, path);
}
