use shared;
use std::rc::Rc;
use std::collections::HashMap;
use std::any::Any;

pub trait PackWriter<Layout> where Layout : shared::Layout { }

pub trait PackWithRefs<Writer, Layout> where Layout : shared::Layout, Writer : PackWriter<Layout> {
    fn pack(&self, pkg:&mut Writer, data:&mut [u8]) -> usize;
}

pub trait PackStatic<Layout> {
    fn pack(&self, data:&mut [u8]) -> usize;
}

impl PackStatic<shared::BinLayout> for i32 {
    fn pack(&self, data:&mut [u8]) -> usize {
        let k = *self as u32;
        <u32 as PackStatic<shared::BinLayout>>::pack(&k, data)
    }
}

impl PackStatic<shared::BinLayout> for u32 {
    fn pack(&self, data:&mut [u8]) -> usize {
        for i in 0..4 {
            data[i] = ((self >> 8*i) & 0xff) as u8;
        }
        return 4;
    }
}

pub struct Slot {
    data: Vec<u8>,
    type_name: String,
    path: String
}

pub struct Entry
{
    type_tag: &'static str,
    path: Option<String>,
}

pub struct PackageRecipe { 
    objects: HashMap<Rc<Any>, Entry>,
    resources: Vec<Vec<u8>>
}

pub fn write_package(_recipe:&PackageRecipe)
{

}
