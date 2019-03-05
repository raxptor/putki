use inki::lexer;
use std::rc::Rc;
use std::sync::Arc;
use shared::TypeDescriptor;
use shared::PutkiError;

pub enum ResolveStatus<T> {
    Resolved(Rc<T>),
    Failed,
    Null
}

pub trait ObjectLoader where Self : Sync + Send {
	fn load(&self, path: &str) -> Option<(&str, &lexer::LexedKv)>;
}

pub trait WriteAsText where Self : Sync + Send {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError>;
}

impl<'a> WriteAsText for &'a str {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.push('\"');
		output.push_str(self);
		output.push('\"');
		Ok(())
	}
}

impl<'a> WriteAsText for String {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.push('\"');
		output.push_str(self);
		output.push('\"');
		Ok(())
	}
}

impl<'a> WriteAsText for i32 {
	fn write_text(&self, output: &mut String) -> Result<(), PutkiError> {
		output.push_str(&self.to_string());
		Ok(())
	}
}

pub trait FieldWriter<Target> {
	fn write_field(&mut self, name:&str, tgt:&Target, sep:bool) -> Result<(), PutkiError>;
}

impl<Target> FieldWriter<Target> for String where Target : WriteAsText {
	fn write_field(&mut self, name:&str, tgt:&Target, sep:bool) -> Result<(), PutkiError> {
		if sep {
			self.push(',');
		}
		self.push_str(name);
		self.push(':');
		tgt.write_text(self)
	}
}

pub trait ParseFromKV where Self:Sized + TypeDescriptor + Clone {
	fn parse(kv : &lexer::LexedKv, resolver: &Arc<InkiResolver>) -> Self;
	fn parse_with_type(kv : &lexer::LexedKv, resolver: &Arc<InkiResolver>, type_name:&str) -> Self {
		if !type_name.is_empty() && <Self as TypeDescriptor>::TAG != type_name {
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
			loader
		}
	}
}

impl InkiResolver {
	pub fn resolve<T>(resolver:&Arc<InkiResolver>, path:&str) -> ResolveStatus<T> where T : ParseFromKV {
		match resolver.loader.load(path)
		{
			Some((type_name, data)) => ResolveStatus::Resolved(Rc::new(<T as ParseFromKV>::parse_with_type(data, resolver, type_name))),
			_ => ResolveStatus::Failed
		}
	}
}

pub fn resolve_from<T>(resolver: &Arc<InkiResolver>, path:&str) -> ResolveStatus<T> where T : ParseFromKV + 'static
{	
	InkiResolver::resolve(resolver, path)
}
