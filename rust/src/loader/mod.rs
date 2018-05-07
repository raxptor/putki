use std::rc::Rc;
use std::any::Any;

pub trait SourceLoader
{
    fn load(&self, path: &str) -> Option<Rc<Any>>;
}
