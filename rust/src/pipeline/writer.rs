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

pub struct PackageRecipe { 
    objects: HashMap<Rc<Any>, Entry>,
    resources: Vec<Vec<u8>>
}

pub fn write_package(_recipe:&PackageRecipe)
{

}
