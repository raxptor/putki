/// Here goes data types thata are shared between outki and pipeline
use std::rc::Rc;
use std::any::Any;

pub trait Resolver<ResolveContext> {
	fn load(&self, pctx: &ResolveContext, path:&str) -> Option<Rc<Any>>;
}

pub trait Layout where Self : 'static { 
}

pub trait PutkiTypeCast where Self : Sized + 'static {
	fn rc_convert(src:Rc<Any>) -> Option<Rc<Self>> { return src.downcast().ok(); }
}

pub trait LayoutDescriptor {
    const TAG : &'static str;
}

pub struct BinLayout { }
pub struct JsonLayout { }

impl Layout for BinLayout {
}

impl LayoutDescriptor for BinLayout {
    const TAG : &'static str = "BinLayout";
}
