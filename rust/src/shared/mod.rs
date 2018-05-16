/// Here goes data types thata are shared between outki and pipeline
use std::rc::Rc;
use std::any::Any;
use std::error::Error;

pub enum PutkiError {
    Test,
    BuilderError(Box<Error>),
    ObjectNotFound,
}

pub trait TypeDescriptor {
    const TAG : &'static str;
}

pub trait Resolver<ResolveContext> {
	fn load(&self, pctx: &ResolveContext, path:&str) -> Option<Rc<Any>>;
}

pub fn tag_of<T>() -> &'static str where T : TypeDescriptor
{
    return <T as TypeDescriptor>::TAG;    
}
