use shared;
use pipeline;
use outki;
use std::rc::Rc;
use std::collections::HashMap;
use std::collections::HashSet;
use std::any::Any;
use ptr;
use shared::PutkiError;

pub struct Slot {
    data: Vec<u8>,
    type_name: String,
    path: String
}

pub struct Entry
{
    type_tag: &'static str,
    path: Option<String>
}

pub trait BinWriter {
    fn write(&self, data: &mut Vec<u8>);
}

#[derive(Default)]
pub struct PackageRefs {
    path_to_slot: HashMap<String, usize>
}

impl<'a> PackageRefs {
    pub fn new() -> PackageRefs {
        Default::default()
    }
}

pub trait BinSaver where Self : Send + Sync {
    fn write(&self, data: &mut Vec<u8>, refs: &PackageRefs) -> Result<(), PutkiError>;
}

impl BinWriter for i32 {
    fn write(&self, data: &mut Vec<u8>) {
        data.push((self & 0xff) as u8);
        data.push(((self >> 8) & 0xff) as u8);
        data.push(((self >> 16) & 0xff) as u8);
        data.push(((self >> 24) & 0xff) as u8);
    }
}

impl BinWriter for u16 {
    fn write(&self, data: &mut Vec<u8>) {
        data.push((self & 0xff) as u8);
        data.push((self >> 8) as u8);
    }
}

impl BinWriter for u32 {
    fn write(&self, data: &mut Vec<u8>) {
        i32::write(&(*self as i32), data);
    }
}

impl BinWriter for usize {
    fn write(&self, data: &mut Vec<u8>) {
        u32::write(&(*self as u32), data);
        u32::write(&(0 as u32), data);
    }
}

impl BinWriter for u8 {
    fn write(&self, data: &mut Vec<u8>) {
        data.push(*self);
    }
}

impl BinWriter for f32 {
    fn write(&self, data: &mut Vec<u8>) {
        unsafe {
            u32::write(&std::mem::transmute::<f32, u32>(*self), data);
        }
    }
}

impl BinWriter for bool {
    fn write(&self, data: &mut Vec<u8>) {
        if *self {
            data.push(1);
        } else {
            data.push(0);
        }
    }
}


impl BinWriter for &str {
    fn write(&self, data: &mut Vec<u8>) {
        let b = self.as_bytes();
        b.len().write(data);
        data.extend_from_slice(self.as_bytes());
    }
}

impl BinWriter for String {
    fn write(&self, data: &mut Vec<u8>) {
        let b = self.as_bytes();
        b.len().write(data);
        data.extend_from_slice(self.as_bytes());
    }
}

impl<T> BinSaver for ptr::Ptr<T> where T : Send + Sync {
    fn write(&self, data: &mut Vec<u8>, refs: &PackageRefs) -> Result<(), PutkiError> {                
        let slot:i32 = self.get_target_path().and_then(|x| { refs.path_to_slot.get(x) }).map(|x| { (*x) as i32 }).unwrap_or(-1);
        slot.write(data);
        Ok(())        
    }
}

#[derive(Default)]
pub struct PackageRecipe { 
    paths: HashSet<String>,
    types: HashSet<&'static str>
}

impl PackageRecipe {
    pub fn new() -> PackageRecipe {
        Default::default()
    }
    pub fn add_object(&mut self, p:&pipeline::Pipeline, path:&str, recurse_deps:bool) -> Result<(), shared::PutkiError> {
        let k = p.peek_build_records().unwrap();
        if let Some(br) = k.get(path) {
            self.types.insert(br.type_tag);
            if self.paths.insert(String::from(path)) && recurse_deps {
                for x in br.deps.keys() {
                    self.add_object(p, x.as_str(), true)?
                }
            }
            Ok(())
        } else {
            Err(shared::PutkiError::ObjectNotFound)
        }
    }
}

pub fn insert_value<T>(data: &mut Vec<u8>, offset:usize, value: T) where T : BinWriter
{
    let mut tmp = Vec::new();
    value.write(&mut tmp);
    data[offset..(tmp.len() + offset)].clone_from_slice(&tmp[..]);
}

pub fn write_package(p:&pipeline::Pipeline, recipe:&PackageRecipe) -> Result<Vec<u8>, shared::PutkiError>
{    
    let mut types : Vec<&'static str> = Vec::new();
    for t in recipe.types.iter() {
        types.push(t);
    }

    let mut manifest:Vec<u8> = Vec::new();
    let mut slot_data_ofs:Vec<(usize, usize)> =Vec::new();
    let mut type_rev:HashMap<&'static str, usize> = HashMap::new();

    (0 as usize).write(&mut manifest);
    types.len().write(&mut manifest);    
    for (tindex, t) in types.iter().enumerate() {
        (*t).write(&mut manifest);
        type_rev.insert(*t, tindex);
    }

    let mut items : Vec<&str> = Vec::new();
    let mut indices : HashMap<&str, usize> = HashMap::new();
    let mut refs = PackageRefs::new();
    for path in recipe.paths.iter() {
        indices.insert(path, items.len());
        refs.path_to_slot.insert(path.to_string(), items.len());
        items.push(path);
    }

    // slot count
    items.len().write(&mut manifest);

    let k = p.peek_build_records().unwrap();    
    for path in items.iter() {          
        let flags:u32 = outki::SLOTFLAG_HAS_PATH | outki::SLOTFLAG_INTERNAL;
        let type_id:usize = *k.get(*path).and_then(|x| type_rev.get(x.type_tag)).unwrap_or(&0);
        flags.write(&mut manifest);
        path.write(&mut manifest);
        type_id.write(&mut manifest);
        let begin = manifest.len();
        (0 as usize).write(&mut manifest);
        let end = manifest.len();
        (0 as usize).write(&mut manifest);
        slot_data_ofs.push((begin, end));
    }

    let manifest_size = manifest.len();
    insert_value(&mut manifest, 0, manifest_size);    

    // All the data.
    for i in 0..items.len() {        
        let path = items[i];
        if let Some(br) = k.get(path) {
            if let Some(ref obj) = br.built_obj {
                let begin = manifest.len();
                obj.write(&mut manifest, &refs)?;
                let end = manifest.len();
                let offsets  = slot_data_ofs[i];
                insert_value(&mut manifest, offsets.0, begin);
                insert_value(&mut manifest, offsets.1, end);
            } else {
                return Err(shared::PutkiError::ObjectNotFound);
            }
        } else {
            return Err(shared::PutkiError::ObjectNotFound);
        }                
    }
    Ok(manifest)
}
