use shared;
use std::rc::Rc;
use std::collections::HashMap;
use std::any::Any;
use ptr;

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

pub struct RefsWriter
{
    pub refs: Vec<(usize, String)>
}

pub trait BinWriter {
    fn write(&self, data: &mut Vec<u8>, refs: &mut RefsWriter);
}

pub struct BinWriteStream {
    data: Vec<u8>
}

impl RefsWriter {
    fn write_ref(&mut self, data: &mut Vec<u8>, path:&str) {
        self.refs.push((data.len(), String::from(path)));        
        0.write(data, self);
    }
}

impl BinWriter for i32 {
    fn write(&self, data: &mut Vec<u8>, _refs: &mut RefsWriter) {
        data.push((self & 0xff) as u8);
        data.push(((self >> 8) & 0xff) as u8);
        data.push(((self >> 16) & 0xff) as u8);
        data.push(((self >> 24) & 0xff) as u8);
    }
}

impl BinWriter for u32 {
    fn write(&self, data: &mut Vec<u8>, refs: &mut RefsWriter) {
        i32::write(&(*self as i32), data, refs);
    }
}

impl BinWriter for u8 {
    fn write(&self, data: &mut Vec<u8>, _refs: &mut RefsWriter) {
        data.push(*self);
    }
}

impl<T> BinWriter for Option<ptr::Ptr<T>> {
    fn write(&self, data: &mut Vec<u8>, refs: &mut RefsWriter) {
        match self {            
            &Some(ref ptr) => refs.write_ref(data, ptr.get_target_path().unwrap_or("")),
            _ => refs.write_ref(data, "")
        };
    }
}

pub struct PackageRecipe { 
    objects: HashMap<Rc<Any>, Entry>,
    resources: Vec<Vec<u8>>
}

pub fn write_package(_recipe:&PackageRecipe)
{

}
