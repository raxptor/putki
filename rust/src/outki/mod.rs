use std::rc::Rc;
use std::any::Any;
use std::marker::PhantomData;
use shared;

pub struct RefsSource<'a, TR, Layout> where TR: 'a, Layout : shared::Layout {
    mgr: &'a PackageManager<TR, Layout>,
    package: u32
}

pub trait TypeResolver<Layout> where Layout : shared::Layout {
    fn unpack_with_type(pkg:&RefsSource<Self, Layout>, data:&[u8], type_name:&str) -> Option<Rc<Any>> where Self : Sized;
}

pub trait UnpackWithRefs<TR, Layout> where Layout : shared::Layout {
    fn unpack(pkg:&RefsSource<TR, Layout>, data:&[u8]) -> Self;
}

pub trait UnpackStatic<Layout> {
    fn unpack(data:&[u8]) -> Self;
}

impl UnpackStatic<shared::BinLayout> for u32 {
    fn unpack(data:&[u8]) -> Self { return (data[0] as u32) | ((data[1] as u32) << 8) | ((data[2] as u32) << 16) | ((data[3] as u32) << 24); }
}

impl UnpackStatic<shared::BinLayout> for i32 {
    fn unpack(data:&[u8]) -> Self { return <u32 as UnpackStatic<shared::BinLayout>>::unpack(data) as i32; }
}

impl<TR, Layout, T> UnpackWithRefs<TR, Layout> for Option<Rc<T>> where Layout : shared::Layout, u32: UnpackStatic<Layout>, T : UnpackWithRefs<TR, Layout> + shared::PutkiTypeCast, TR : TypeResolver<Layout> {
    fn unpack(pkg:&RefsSource<TR, Layout>, data:&[u8]) -> Self {
        return PackageManager::obj_from_slot(pkg, <u32 as UnpackStatic<Layout>>::unpack(data)).and_then(|x| { shared::PutkiTypeCast::rc_convert(x) });
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

pub struct PackageManager<TypeResolver, Layout> {
    packages: Vec<Package>,
    _m0: PhantomData<TypeResolver>,
    _m1: PhantomData<Layout>
}

// TypeResolver impl knows how map type name strings to unpack implementations
// Layout defines how to serialize
impl<TR, Layout> PackageManager<TR, Layout> where TR : TypeResolver<Layout>, Layout : shared::Layout
{
    pub fn new() -> PackageManager<TR, Layout> { PackageManager {
        packages: Vec::new(),
        _m0: PhantomData,
        _m1: PhantomData
    }}

    pub fn insert(&mut self, p:Package) { self.packages.push(p); }
    pub fn resolve<T>(&self, path:&str) -> Option<Rc<T>> where T : shared::PutkiTypeCast + UnpackWithRefs<TR, Layout> { 
        for ref p in &self.packages {
            for idx in 0 .. p.slots.len() {
                let s = &p.slots[idx];
                if let &Some(ref pth) = &s.path {
                    if pth == path {
                        let rs:RefsSource<TR, Layout> = RefsSource {
                            mgr: self,
                            package: 0
                        };
                        return Self::obj_from_slot(&rs, idx as u32).and_then(|x| { shared::PutkiTypeCast::rc_convert(x) });;
                    }
                }
            }
        }
        return None;
    }

    fn obj_from_slot<'a>(refs:&RefsSource<'a, TR, Layout>, slot:u32) -> Option<Rc<Any>> where Layout : shared::Layout {
        let pkg = &refs.mgr.packages[refs.package as usize];
        let slotidx = slot as usize;
        if slotidx >= pkg.slots.len() {
            return None;
        }         
        if let &Some(ref data) = &pkg.content {
            let s = &pkg.slots[slotidx];
            println!("Unpacking object at slot {}, byte range [{}..{}] in package {} with type {}", slot, s.begin, s.end, slotidx, s.type_name);
            return TR::unpack_with_type(&refs, &data[s.begin .. s.end], s.type_name.as_str());
        }
        return None;
    }
}


