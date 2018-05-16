use std::rc::Rc;
use shared;

impl<T> shared::TypeDescriptor for Option<Rc<T>> {
    const TAG : &'static str = "Option<Rc<T>> placeholder";
}

impl shared::TypeDescriptor for i32 {
    const TAG : &'static str = "i32";
}

impl shared::TypeDescriptor for u32 {
    const TAG : &'static str = "u32";
}

pub struct BinReaderContext<'a> {
    mgr: &'a BinPackageManager,
    package: i32
}

pub struct BinDataStream<'a> {
    context: &'a BinReaderContext<'a>,
    data: &'a [u8],
    pos: usize
}

pub trait BinReader {
    fn read(stream:&mut BinDataStream) -> Self;
}

impl BinReader for u32 {
    fn read(stream:&mut BinDataStream) -> Self {
        let pos = stream.pos;
        stream.pos = stream.pos + 4;
        let v = (stream.data[pos] as u32) |
        ((stream.data[pos+1] as u32) << 8) | 
        ((stream.data[pos+2] as u32) << 16) | 
        ((stream.data[pos+3] as u32) << 24);
        return v as u32;
    }
}

impl<'a> BinReader for i32 {
    fn read(ctx: &mut BinDataStream) -> i32 {
        u32::read(ctx) as i32
    }
}

impl<T> BinReader for Option<Rc<T>> where T : BinReader {
    fn read(ctx: &mut BinDataStream) -> Self {
        return BinPackageManager::obj_from_slot(ctx.context, u32::read(ctx))
    }
}

pub struct Slot {
    path: Option<String>,
    type_name: String,
    _package_ref:u32, // 0 = self, otherwise external package mapping table.
    begin: usize,
    end: usize
}

pub struct Package {
    content: Option<Vec<u8>>,
    slots: Vec<Slot>    
}

impl Package {
    pub fn new() -> Self {
        Package { 
            content: None,
            slots: Vec::new()
        }
    }
    pub fn insert(&mut self, path: Option<&str>, type_name:&str, data:&[u8]) {
        let begin;
        if let Some(ref mut d) = self.content {
            begin = d.len();
            d.extend_from_slice(data);
        } else {
            begin = 0;
            self.content = Some(Vec::from(data));
        }
        self.slots.push(Slot {
            path: path.and_then(|x| { return Some(String::from(x)) }),
            type_name: String::from(type_name),
            _package_ref: 0,
            begin: begin,
            end: begin + data.len()             
        });
    }
}

pub struct BinPackageManager {
    packages: Vec<Package>    
}

// TypeResolver impl knows how map type name strings to unpack implementations
// Layout defines how to serialize
impl BinPackageManager
{
    pub fn new() -> BinPackageManager { BinPackageManager {
        packages: Vec::new()        
    }}

    pub fn insert(&mut self, p:Package) { self.packages.push(p); }
    pub fn resolve<'a, T>(&'a self, path:&str) -> Option<Rc<T>> where T : BinReader { 
        for ref p in &self.packages {
            for idx in 0 .. p.slots.len() {
                let s = &p.slots[idx];
                if let &Some(ref pth) = &s.path {
                    if pth == path {
                        let rs = BinReaderContext {
                            mgr: &self,
                            package: 0
                        };
                        return Self::obj_from_slot(&rs, idx as u32);
                    }
                }
            }
        }
        return None;
    }

    fn obj_from_slot<T>(context:&BinReaderContext, slot:u32) -> Option<Rc<T>> where T : BinReader {
        let pkg = &context.mgr.packages[context.package as usize];
        let slotidx = slot as usize;
        if slotidx >= pkg.slots.len() {
            return None;
        }         
        if let &Some(ref data) = &pkg.content {
            let s = &pkg.slots[slotidx];
            println!("Unpacking object at slot {}, byte range [{}..{}] in package {} with type {}", slot, s.begin, s.end, slotidx, s.type_name);
            let mut stream = BinDataStream {
                context: context,
                data: &data[s.begin .. s.end],
                pos: 0
            };
            return Some(Rc::new(T::read(&mut stream)));
        }
        return None;
    }
}
