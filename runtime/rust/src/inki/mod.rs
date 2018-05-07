use std::ops::Deref;
use std::marker::PhantomData;
use std::cell::RefCell;
use std::rc::Rc;

pub trait Tracker
{
    fn follow(&self, path:&str);
}

pub trait ResolveFrom<ObjSource>
{
    fn resolve(src:&ObjSource, path:&str) -> Option<Rc<Self>>;
}

pub struct PtrContext<'a, ObjSource:'a>
{
    tracker: Option<&'a Tracker>,
    source: &'a ObjSource
}

enum PtrStatus<T>
{
    Unresolved,
    Resolved(Rc<T>),
    Failed,
    Null
}

pub struct Ptr<'b, ObjSource:'b, Target:'b> where Target : ResolveFrom<ObjSource>
{
    context : PtrContext<'b, ObjSource>,
    status: RefCell<PtrStatus<Target>>,
    path : String    
}

struct Vacuum { }
impl<'a> ResolveFrom<Vacuum> for Apa<'a>
{
    fn resolve(src:&Vacuum, path:&str) -> Option<Rc<Apa<'a>>> { return None; }
}

struct Apa<'a>
{
    k: Ptr<'a, Vacuum, Apa<'a>>
}

impl<'a> Apa<'a>
{
    fn new(source:&'a Vacuum) -> Apa<'a>
    {
        return Apa {
            k: Ptr {
                status: RefCell::new(PtrStatus::Unresolved),
                context: PtrContext {
                    tracker: None,
                    source: source
                },
                path: String::from("hej")                
            }
        }
    }
}

impl<'a, ObjSource, T> Ptr<'a, ObjSource, T> where T : ResolveFrom<ObjSource>
{
    pub fn resolve(&self) -> Option<Rc<T>> {
        if let Some(trk) = self.context.tracker {
            trk.follow(&self.path);
        }
        // attempt to pick up pointer.
        if let PtrStatus::Resolved(ptr) = self.status.borrow().deref() {
            return Some(ptr.clone());
        } else {
            let res:Option<Rc<T>> = ResolveFrom::resolve(self.context.source, self.path.as_str());
            match res {
                Some(x) => { 
                    let mut cache = self.status.borrow_mut();
                    *cache = PtrStatus::Resolved(x.clone());
                    return Some(x);
                }
                None => {

                }
            }
        }
        unimplemented!();
    }
}
