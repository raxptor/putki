use std::any::TypeId;
use std::rc::Rc;

pub trait Downcast<BaseInner>
{
    fn downcast<CInner>(&self) -> Option<(&BaseInner, Rc<CInner>)> where CInner : 'static;
    fn type_id(&self) -> TypeId;
}


/*
impl Deref for Base
{
    type Target = BaseInner;
    fn deref(&self) -> &BaseInner {
        return &self.inner;
    }
}
*/