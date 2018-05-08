/// Here goes data types thata are shared between outki and pipeline
use std::rc::Rc;
use std::any::Any;

pub trait Resolver<ResolveContext> {
	fn load(&self, pctx: &ResolveContext, path:&str) -> Option<Rc<Any>>;
}

pub trait Layout where Self : 'static { 
}

pub trait LayoutDescriptor {
    const TAG : &'static str;
}

pub trait OutkiTypeDescriptor {
    const TAG : &'static str;
    const SIZE : usize;
}

pub struct BinLayout { }
pub struct JsonLayout { }

impl Layout for BinLayout {
}

impl LayoutDescriptor for BinLayout {
    const TAG : &'static str = "BinLayout";
}

pub fn tag_of<T>() -> &'static str where T : OutkiTypeDescriptor
{
    return <T as OutkiTypeDescriptor>::TAG;    
}

pub fn size_of<T>() -> usize where T : OutkiTypeDescriptor
{
    return <T as OutkiTypeDescriptor>::SIZE;
}