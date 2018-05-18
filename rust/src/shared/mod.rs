/// Here goes data types thata are shared between outki and pipeline
use std::rc::Rc;
use std::any::Any;
use std::error::Error;
use std::io;

pub enum PutkiError {
    Test,
    BuilderError(Box<Error>),
    IOError(io::Error),
    ObjectNotFound,
}

impl From<io::Error> for PutkiError {
    fn from(err:io::Error) -> Self {
        PutkiError::IOError(err)
    }
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
