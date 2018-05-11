use inki::source;
use std::marker::PhantomData;
use std::rc::Rc;
use std::fmt;
use std::result;

struct PtrTarget
{
    context : source::InkiPtrContext,
    path : String,
}

pub struct Ptr<Target> where Target : source::ParseFromKV
{
    target: Option<PtrTarget>,
    _m : PhantomData<Rc<Target>>
}

impl<Target> fmt::Debug for Ptr<Target> where Target : source::ParseFromKV {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match &self.target {
            &None => { write!(f, "Null pointer").ok(); }
            &Some(ref target) => { write!(f, "Ptr path[{}]", &target.path).ok(); }
        }
        return Ok(());
    }
}

impl<Target> Ptr<Target> where Target : source::ParseFromKV
{
    pub fn new(context : source::InkiPtrContext, path: &str) -> Ptr<Target>
    {
        return Ptr {
            target: Some(PtrTarget {
                context: context,
                path: String::from(path)
            }),
            _m: PhantomData { }
        }
    }
    pub fn null() -> Ptr<Target>
    {
        return Ptr {
            target: None,
            _m: PhantomData { }
        }
    }    
}

impl<T> Ptr<T> where T : source::ParseFromKV + 'static
{
    pub fn resolve(&self) -> Option<Rc<T>> {
        match &self.target {
            &None => return None,
            &Some(ref target) => {
                if let &Some(ref trk) = &target.context.tracker {
                    trk.follow(&target.path);
                }
                if let source::ResolveStatus::Resolved(ptr) = source::resolve_from::<T>(&target.context, &target.path) {
                    return Some(ptr.clone());
                } else {
                    return None;
                } 
            }
        }
    }
}
