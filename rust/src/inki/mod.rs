use std::marker::PhantomData;
use std::rc::Rc;
use std::fmt;
use std::result;
use std::any::Any;

pub mod source;
pub mod loadall;
pub mod lexer;

pub use self::source::*;
pub use self::loadall::*;
pub use self::lexer::*;

pub trait SourceLoader
{
    fn load(&self, path: &str) -> Option<Rc<Any>>;
}

pub struct Ptr<Target> where Target : source::ParseFromKV
{
    context : source::PtrContext,
    path : String,
    _m : PhantomData<Rc<Target>>
}

impl<Target> fmt::Debug for Ptr<Target> where Target : source::ParseFromKV {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "Ptr path[{}]", &self.path);
        return Ok(());
    }
}

impl<Target> Ptr<Target> where Target : source::ParseFromKV
{
    pub fn new(context : source::PtrContext, path: &str) -> Ptr<Target>
    {
        return Ptr {
            context: context,
            path: String::from(path),
            _m: PhantomData { }
        }
    }
}

impl<T> Ptr<T> where T : source::ParseFromKV + source::PutkiTypeCast
{
    pub fn resolve(&self) -> Option<Rc<T>> {
        if let &Some(ref trk) = &self.context.tracker {
            trk.follow(&self.path);
        }
        if let source::ResolveStatus::Resolved(ptr) = source::resolve_from(&self.context.source, &self.context, &self.path) {
            return Some(ptr);
        }
        return None;
    }
}
