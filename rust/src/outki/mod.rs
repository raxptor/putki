use std::rc::Rc;
use std::marker::PhantomData;

pub trait Unpack<Layout> {
    fn unpack(pkg:&Package<Layout>) -> Self;
}

pub struct Package<Layout> {
    layout: PhantomData<Layout> 
}

impl<Layout> Package<Layout> {
   pub fn unpack<Target>(&self) -> Target where Target : Unpack<Layout> {
       return Unpack::unpack(&self);
   }
   pub fn new() -> Self {
        return Self {
            layout: PhantomData
       }
   }
}
