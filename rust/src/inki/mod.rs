use std::marker::PhantomData;
use std::rc::Rc;
use std::fmt;
use std::result;

pub mod source;
pub mod loadall;

use source::*;  

pub struct Ptr<Target> where Target : ParseFromKV
{
    context : PtrContext,
    path : String,
    _m : PhantomData<Rc<Target>>
}

impl<Target> fmt::Debug for Ptr<Target> where Target : ParseFromKV {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        write!(f, "Ptr path[{}]", &self.path);
        return Ok(());
    }
}

impl<Target> Ptr<Target> where Target : ParseFromKV
{
    pub fn new(context : PtrContext, path: &str) -> Ptr<Target>
    {
        return Ptr {
            context: context,
            path: String::from(path),
            _m: PhantomData { }
        }
    }
}

impl<T> Ptr<T> where T : ParseFromKV + PutkiTypeCast
{
    pub fn resolve(&self) -> Option<Rc<T>> {
        if let &Some(ref trk) = &self.context.tracker {
            trk.follow(&self.path);
        }
        if let ResolveStatus::Resolved(ptr) = resolve_from(&self.context.source, &self.context, &self.path) {
            return Some(ptr);
        }
        return None;
    }
}
