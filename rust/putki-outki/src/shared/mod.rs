/// Here goes data types thata are shared between outki and pipeline
use std::rc::Rc;
use std::any::Any;
use std::error::Error;
use std::io;

#[derive(Debug)]
pub enum PutkiError {
    Test,
    BuilderError(Box<dyn Error>),
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
    /// Compiler-assigned type id, written into package slots so a reader can
    /// check it is being handed the type it asked for. See
    /// `doc/package-format.md`.
    ///
    /// Defaults to 0, meaning "unknown", which is what a hand-written or
    /// not-yet-regenerated descriptor reports; readers skip the check rather
    /// than reject those.
    const TYPE_ID : usize = 0;
}

pub trait Resolver<ResolveContext> {
	fn load(&self, pctx: &ResolveContext, path:&str) -> Option<Rc<dyn Any>>;
}

pub fn tag_of<T>() -> &'static str where T : TypeDescriptor
{
    <T as TypeDescriptor>::TAG
}
