use std::rc::Rc;
use std::ops::Deref;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::collections::HashMap;
use std::mem::forget;
use shared;

pub trait OutkiObj : BinReader + shared::TypeDescriptor
{
}

struct Pin
{

}

pub struct NullablePtr<T>
{
    ptr: Option<NonZeroUsize>,
    _ph: PhantomData<T>
}

pub struct Ptr<T>
{
    ptr: NonZeroUsize,
    _ph: PhantomData<T>
}

pub struct Ref<T>
{
    ptr: Ptr<T>,
    pin: Rc<Pin>
}

impl<T> Deref for Ref<T>
{
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe {
            println!("deref {}", self.ptr.ptr.get());
            &(*(self.ptr.ptr.get() as (*const T)))
        }
    }
}

impl<'a, T> NullablePtr<T>
{
    pub fn get(&self) -> Option<&'a T>
    {
        unsafe {
            self.ptr.map(|x| {
                &(*(x.get() as (*const T)))
            })
        }
    }
}

impl<T> Ref<T>
{
    pub fn get_pointer(&self) -> *const T {
        self.ptr.ptr.get() as (*const T)
    }
}

impl<T> Deref for Ptr<T>
{
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe {
            println!("deref {}", self.ptr.get());
            &(*(self.ptr.get() as (*const T)))
        }
    }
}

impl<T> shared::TypeDescriptor for NullablePtr<T> {
    const TAG : &'static str = "NullablePtr<T>> placeholder";
}

impl<T> shared::TypeDescriptor for Ptr<T> {
    const TAG : &'static str = "NullablePtr<T>> placeholder";
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

pub struct UnresolvedEntry {
    tag: &'static str,
    slot: usize    
}

pub struct BinResolverContext<'a> {
    context:&'a BinReaderContext<'a>, 
    unresolved: Vec<UnresolvedEntry>,
    loaded: HashMap<usize, usize>      // slot to addr
}

type Loader = fn(i32) -> usize;

impl<'a> BinResolverContext<'a> {
    pub fn resolve<T>(&mut self, ptr: &mut NullablePtr<T>) -> bool where T : OutkiObj {
        match ptr.ptr {
            Some(slot) => {
                let rslot = usize::from(slot);
                let get_res = self.loaded.get(&rslot).map(|x| x.clone());
                if let Some(addr) = get_res {
                    println!("PTR! Resolved rslot to addr {} {}", rslot, addr);
                    ptr.ptr = NonZeroUsize::new(addr);
                    true
                } else {
                    println!("PTR! load_and_try_resolve rslot {}", rslot);
                    BinPackageManager::load_and_try_resolve::<T>(self, rslot as u32);
                    let get_res = self.loaded.get(&rslot).map(|x| x.clone());
                    if let Some(addr) = get_res {
                        println!("PTR! Loaded and reslovede Resolved rslot to addr {} {}", rslot, addr);
                        ptr.ptr = NonZeroUsize::new(addr);
                        true
                    } else {
                        println!("Failed after load");
                        self.unresolved.push(
                            UnresolvedEntry {
                                slot: rslot,
                                tag: shared::tag_of::<T>()
                            }
                        );                        
                        false
                    }                    
                }
            },
            None => {
                println!("it is a null pointer!");
                true
            }
        }

/*
        let p : *mut NullablePtr<T> = ptr;
        let q = p as *mut NullablePtr<i32>;
        
        

        self.unresolved.push(
            ResolveEntry {
                tag: shared::tag_of::<T>(),
                slot: 0,
                ptr_ptr: q
            }
        );
*/        
    }
}


pub struct BinDataStream<'a> {
    context: &'a BinReaderContext<'a>,
    data: &'a [u8],
    pos: usize
}

pub trait BinReader {
    fn read(stream:&mut BinDataStream) -> Self;
    fn resolve(&mut self, context: &mut BinResolverContext) { }
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

impl BinReader for u8 {
    fn read(stream:&mut BinDataStream) -> Self {
        stream.pos = stream.pos + 1;
        stream.data[stream.pos-1]
    }
}

impl<'a> BinReader for i32 {
    fn read(ctx: &mut BinDataStream) -> i32 {
        u32::read(ctx) as i32
    }
}

impl<T> BinReader for NullablePtr<T> where T : BinReader {
    fn read(ctx: &mut BinDataStream) -> Self {
        let slotplusone:i32 = i32::read(ctx) + 1;
        NullablePtr { ptr: NonZeroUsize::new(slotplusone as usize), _ph: PhantomData { } }
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

pub struct ResolvedBunch {
    addrs: HashMap<u32, usize>
}

// TypeResolver impl knows how map type name strings to unpack implementations
// Layout defines how to serialize
impl BinPackageManager
{
    pub fn new() -> BinPackageManager { BinPackageManager {
        packages: Vec::new()        
    }}

    pub fn insert(&mut self, p:Package) { self.packages.push(p); }
    pub fn resolve<'a, T>(&'a self, path:&str) -> Option<Ref<T>> where T : OutkiObj { 
        for ref p in &self.packages {
            for idx in 0 .. p.slots.len() {
                let s = &p.slots[idx];
                if let &Some(ref pth) = &s.path {
                    if pth == path {
                        let rs = BinReaderContext {
                            mgr: &self,
                            package: 0
                        };
                        return self.tree_resolve::<T>(&rs, (idx as u32) + 1);
                    }
                }
            }
        }
        return None;
    }

    fn tree_resolve<T>(&self, context:&BinReaderContext, slot:u32) -> Option<Ref<T>> where T : OutkiObj
    {
        let mut rctx = BinResolverContext {
            unresolved: Vec::new(),
            loaded: HashMap::new(),
            context: &context
        };
    
        let mut ptr : NullablePtr<T> = NullablePtr {
            ptr: NonZeroUsize::new(slot as usize),
            _ph: PhantomData { }
        };

        if rctx.resolve(&mut ptr) {
            ptr.ptr.map(|addr| { Ref {
                    ptr: Ptr {
                        ptr: addr,
                        _ph: PhantomData { }
                    },
                    pin: Rc::new(Pin { })
                }
            })
        } else {
            None
        }
    }

    fn load_and_try_resolve<T>(res: &mut BinResolverContext, slot:u32) -> bool where T : OutkiObj {
        let pkg = &res.context.mgr.packages[res.context.package as usize];
        let slotidx = (slot-1) as usize;
        if slotidx >= pkg.slots.len() {
            return false;
        }
        if let &Some(ref data) = &pkg.content {
            let s = &pkg.slots[slotidx];
            println!("Unpacking object at slot {}, byte range [{}..{}] in package {} with type {}", slot, s.begin, s.end, slotidx, s.type_name);
            let mut stream = BinDataStream {
                context: res.context,
                data: &data[s.begin .. s.end],
                pos: 0
            };
            let mut obj:Box<T> = Box::new(BinReader::read(&mut stream));
            let objptr : *mut T = &mut *obj;
            res.loaded.insert(slot as usize, objptr as usize);
            obj.resolve(res);
            forget(obj);
            return true;
        }
        false
    }
}
