use std::rc::Rc;
use std::ops::Deref;
use std::sync::Arc;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::collections::HashMap;
use std::mem::forget;
use shared;
mod binreader;

pub use self::binreader::*;

#[derive(Debug)]
pub enum OutkiError {
    SlotNotFound,
    ResolveFailed,
    DataMissing,
    NonNullIsNull
}

pub type OutkiResult<T> = Result<T, OutkiError>;
pub trait OutkiObj : BinLoader + shared::TypeDescriptor { }
type Destructor = fn(usize);

pub trait BinLoader {
    fn read(stream:&mut BinDataStream) -> Self;
    fn resolve(&mut self, context: &mut BinResolverContext) -> OutkiResult<()> { Ok(()) }
}

impl<T> BinReader for NullablePtr<T> where T : BinLoader {
    fn read(stream: &mut BinDataStream) -> Self {
        let slotplusone:i32 = i32::read(stream) + 1;
        NullablePtr { ptr: NonZeroUsize::new(slotplusone as usize), _ph: PhantomData { } }
    }
}

impl<T> BinReader for Ptr<T> where T : BinLoader {
    fn read(stream: &mut BinDataStream) -> Self {
        let slotplusone:i32 = i32::read(stream) + 1;
        Ptr { ptr: slotplusone as usize, _ph: PhantomData { } }
    }
}


struct MemoryPin 
{
    destructors: HashMap<usize, Destructor>
}

impl Drop for MemoryPin {
    fn drop(&mut self) {
        for (k, v) in self.destructors.iter() {
            v(*k);
        }
    }
}

struct PinTag
{
    mempin: Arc<MemoryPin>
}

pub struct NullablePtr<T>
{
    ptr: Option<NonZeroUsize>,
    _ph: PhantomData<T>
}

pub struct Ptr<T>
{
    ptr: usize,
    _ph: PhantomData<T>
}

pub struct Ref<T>
{
    ptr: Ptr<T>,
    pin: Rc<PinTag>
}

impl<T> Deref for Ref<T>
{
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe {
            &(*(self.ptr.ptr as (*const T)))
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
        self.ptr.ptr as (*const T)
    }
}

impl<T> Deref for Ptr<T>
{
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        unsafe {
            &(*(self.ptr as (*const T)))
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

pub struct UnresolvedEntry {
    tag: &'static str,
    slot: usize    
}

pub struct BinResolverContext<'a> {
    context:&'a BinReaderContext<'a>, 
    unresolved: Vec<UnresolvedEntry>,
    loaded: HashMap<usize, usize>,      // slot to addr
    pindata: MemoryPin
}

type Loader = fn(i32) -> usize;

impl<'a> BinResolverContext<'a> {

    pub fn resolve_not_null<T>(&mut self, ptr: &mut Ptr<T>) -> OutkiResult<()> where T : OutkiObj {
        let addr = ptr.ptr;
        if addr == 0 {
            return Err(OutkiError::NonNullIsNull);
        }
        let mut tmp_ptr : NullablePtr<T> = NullablePtr {
            ptr: Some(NonZeroUsize::new(ptr.ptr).unwrap()),
            _ph: PhantomData { }
        };
        self.resolve(&mut tmp_ptr)?;
        if let Some(x) = tmp_ptr.ptr {
            ptr.ptr = x.get();
            Ok(())
        } else {
            Err(OutkiError::NonNullIsNull)
        }
    }
 
    pub fn resolve<T>(&mut self, ptr: &mut NullablePtr<T>) -> OutkiResult<()> where T : OutkiObj {
        match ptr.ptr {
            Some(slot) => {
                let rslot = usize::from(slot);
                let get_res = self.loaded.get(&rslot).map(|x| x.clone());
                if let Some(addr) = get_res {
                    ptr.ptr = NonZeroUsize::new(addr);
                    Ok(())
                } else {
                    BinPackageManager::load_and_try_resolve::<T>(self, rslot as u32)?;
                    let get_res = self.loaded.get(&rslot).map(|x| x.clone());
                    if let Some(addr) = get_res {
                        println!("PTR! Loaded and reslovede Resolved rslot to addr {} {}", rslot, addr);
                        ptr.ptr = NonZeroUsize::new(addr);
                        Ok(())
                    } else {
                        self.unresolved.push(
                            UnresolvedEntry {
                                slot: rslot,
                                tag: shared::tag_of::<T>()
                            }
                        );                        
                        Err(OutkiError::ResolveFailed)
                    }                    
                }
            },
            None => {
                println!("it is a null pointer!");
                Ok(())
            }
        }     
    }
}

pub struct BinReaderContext<'a> {
    mgr: &'a BinPackageManager,
    package: i32
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
    pub fn resolve<'a, T>(&'a self, path:&str) -> OutkiResult<Ref<T>> where T : OutkiObj { 
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
        Err(OutkiError::ResolveFailed)
    }

    fn tree_resolve<T>(&self, context:&BinReaderContext, slot:u32) -> OutkiResult<Ref<T>> where T : OutkiObj
    {
        let mut rctx = BinResolverContext {
            unresolved: Vec::new(),
            loaded: HashMap::new(),
            pindata: MemoryPin { destructors: HashMap::new() },
            context: &context
        };
    
        let mut ptr : NullablePtr<T> = NullablePtr {
            ptr: NonZeroUsize::new(slot as usize),
            _ph: PhantomData { }
        };

        rctx.resolve(&mut ptr)?;
        if let Some(addr) = ptr.ptr {
            Ok(Ref {
                ptr: Ptr {
                    ptr: addr.get(),
                    _ph: PhantomData { }
                },
                pin: Rc::new(PinTag { 
                    mempin: Arc::new(rctx.pindata)
                })
            })
        } else {
            Err(OutkiError::ResolveFailed)
        }
    }

    fn destruct<T>(objptr:usize)
    {
        println!("destructing {}", objptr);
        unsafe {
            let bx : Box<T> = Box::from_raw(objptr as *mut T);
            drop(bx);
        }
    }

    fn load_and_try_resolve<T>(res: &mut BinResolverContext, slot:u32) -> OutkiResult<()> where T : OutkiObj {
        let pkg = &res.context.mgr.packages[res.context.package as usize];
        let slotidx = (slot-1) as usize;
        if slotidx >= pkg.slots.len() {
            return Err(OutkiError::SlotNotFound);
        }
        if let &Some(ref data) = &pkg.content {
            let s = &pkg.slots[slotidx];
            println!("Unpacking object at slot {}, byte range [{}..{}] in package {} with type {}", slot, s.begin, s.end, slotidx, s.type_name);
            let mut stream = BinDataStream::new(&data[s.begin .. s.end]);
            let mut obj:Box<T> = Box::new(BinLoader::read(&mut stream));
            let objptr : *mut T = &mut *obj;
            res.loaded.insert(slot as usize, objptr as usize);
            res.pindata.destructors.insert(objptr as usize, Self::destruct::<T>);
            let r = obj.resolve(res);
            forget(obj); // need to forget before try! or we will have stored double references.
            r?;
            Ok(())
        } else {
            Err(OutkiError::DataMissing)
        }
    }
}
