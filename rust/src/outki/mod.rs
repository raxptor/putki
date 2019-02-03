use std::rc::Rc;
use std::ops::Deref;
use std::sync::Arc;
use std::marker::PhantomData;
use std::num::NonZeroUsize;
use std::collections::HashMap;
use std::mem::forget;
use std::ops::DerefMut;
use std::io;
use shared;
mod binreader;
mod pkmanifest;

pub use self::binreader::*;
pub use self::pkmanifest::*;

pub const SLOTFLAG_HAS_PATH:u32   = 1;
pub const SLOTFLAG_INTERNAL:u32   = 2;

#[derive(Debug)]
pub enum OutkiError {
    SlotNotFound,
    ResolveFailed,
    DataMissing,
    NonNullIsNull,
    IOError
}

impl From<io::Error> for OutkiError {
    fn from(_e : io::Error) -> OutkiError {
        OutkiError::IOError
    }
}

pub trait PackageRandomAccess {    
    fn read_chunk(&self, begin:usize, into:&mut [u8]) -> OutkiResult<()>;
}

pub type OutkiResult<T> = Result<T, OutkiError>;
pub trait OutkiObj : BinLoader + shared::TypeDescriptor { }
type Destructor = fn(usize);

const UNRESOLVED_MASK:usize = 0xff00000000000000;
const UNRESOLVED_VALUE:usize = 0xcd00000000000000;

pub trait BinLoader {
    fn read(stream:&mut BinDataStream) -> Self;
    fn resolve(&mut self, context: &mut BinResolverContext) -> OutkiResult<()>;
}

impl<T> BinReader for NullablePtr<T> where T : BinLoader {
    fn read(stream: &mut BinDataStream) -> Self {
        let mut slotplusone:usize = (i32::read(stream) + 1) as usize;
        if slotplusone != 0 {
            slotplusone = slotplusone | UNRESOLVED_VALUE;
            println!("and it is now {}", slotplusone);
        }
        NullablePtr { ptr: NonZeroUsize::new(slotplusone), _ph: PhantomData { } }
    }
}

impl<T> BinReader for Ptr<T> where T : BinLoader {
    fn read(stream: &mut BinDataStream) -> Self {
        let slotplusone:i32 = i32::read(stream) + 1;
        Ptr { ptr: (slotplusone as usize | UNRESOLVED_VALUE), _ph: PhantomData { } }
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
    _mempin: Arc<MemoryPin>
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
    _pin: Rc<PinTag>
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
                debug_assert!((x.get() & UNRESOLVED_MASK) != UNRESOLVED_VALUE);
                &(*(x.get() as (*const T)))
            })
        }
    }
    pub fn unwrap(&self) -> &'a T {
        self.get().unwrap()
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
        debug_assert!((self.ptr & UNRESOLVED_MASK) != UNRESOLVED_VALUE);
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
//    tag: &'static str,
//    slot: usize    
}

pub struct BinResolverContext<'a> {
    context:&'a BinReaderContext<'a>, 
    _unresolved: Vec<UnresolvedEntry>,
    loaded: HashMap<usize, usize>,      // slot to addr
    pindata: MemoryPin
}

impl<'a> BinResolverContext<'a> {

    pub fn resolve_not_null<T>(&mut self, ptr: &mut Ptr<T>) -> OutkiResult<()> where T : OutkiObj {
        let addr = ptr.ptr & !UNRESOLVED_MASK;
        if addr == 0 {
            return Err(OutkiError::NonNullIsNull);
        }
        debug_assert!((ptr.ptr & UNRESOLVED_MASK) == UNRESOLVED_VALUE);
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
                let rslot = usize::from(slot) & !UNRESOLVED_MASK;
                println!("{} {}", usize::from(slot), rslot);
                debug_assert!((usize::from(slot) & UNRESOLVED_MASK) == UNRESOLVED_VALUE);                
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
                        /*
                        self.unresolved.push(
                            UnresolvedEntry {
                                slot: rslot,
                                tag: shared::tag_of::<T>()
                            }
                        );*/                        
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

pub struct Package {
    manifest: PackageManifest,
    reader: Box<dyn PackageRandomAccess>
}

impl Package {    
    pub fn new(mf:PackageManifest, rdr:Box<dyn PackageRandomAccess>) -> Self {
        Package { 
            manifest: mf,
            reader: rdr
        }
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
    pub fn resolve<'a, T>(&'a self, path:&str) -> OutkiResult<Ref<T>> where T : OutkiObj { 
        for ref p in &self.packages {
            for idx in 0 .. p.manifest.slots.len() {
                let s = &p.manifest.slots[idx];
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
            _unresolved: Vec::new(),
            loaded: HashMap::new(),
            pindata: MemoryPin { destructors: HashMap::new() },
            context: &context
        };
    
        let mut ptr : NullablePtr<T> = NullablePtr {
            ptr: NonZeroUsize::new(slot as usize | UNRESOLVED_VALUE),
            _ph: PhantomData { }
        };

        rctx.resolve(&mut ptr)?;
        if let Some(addr) = ptr.ptr {
            Ok(Ref {
                ptr: Ptr {
                    ptr: addr.get(),
                    _ph: PhantomData { }
                },
                _pin: Rc::new(PinTag { 
                    _mempin: Arc::new(rctx.pindata)
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
        if slotidx >= pkg.manifest.slots.len() {
            return Err(OutkiError::SlotNotFound);
        }

        let s = &pkg.manifest.slots[slotidx];
        let size = s.end - s.begin;
        let mut tmp_buf = vec![0 as u8; size];
        pkg.reader.read_chunk(s.begin, tmp_buf.deref_mut())?;
            
        println!("Unpacking object at slot {}, byte range [{}..{}] in package {} with type {}", slot, s.begin, s.end, slotidx, s.type_id);
        let mut stream = BinDataStream::new(tmp_buf.deref());
        let mut obj:Box<T> = Box::new(BinLoader::read(&mut stream));
        let objptr : *mut T = &mut *obj;
        res.loaded.insert(slot as usize, objptr as usize);
        res.pindata.destructors.insert(objptr as usize, Self::destruct::<T>);
        let r = obj.resolve(res);
        forget(obj); // need to forget before try! or we will have stored double references.
        r?;
        Ok(())
    }
}
