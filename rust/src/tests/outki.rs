use shared;
use outki;
use std::rc::Rc;
use shared::tag_of;
use pipeline::writer::BinWriter;

#[derive(Debug)]
struct PointedTo {
    value1: u8,
    value2: i32    
}

//#[derive(Debug)]
struct PtrStruct {
    pub ptr: outki::NullablePtr<PointedTo>,
    pub sib: outki::NullablePtr<PtrStruct>
}

struct PtrStructNotNull {
    pub ptr: outki::Ptr<PointedTo>,
    pub sib: outki::Ptr<PtrStructNotNull>
}

use outki::BinReader;

impl outki::BinReader for PointedTo {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {            
            value1: u8::read(stream),
            value2: i32::read(stream)
        }
    }
}

impl outki::BinReader for PtrStruct {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
            ptr: outki::NullablePtr::<PointedTo>::read(stream),
            sib: outki::NullablePtr::<PtrStruct>::read(stream)
        }
    }
    fn resolve(&mut self, context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> { 
        try!(context.resolve(&mut self.ptr));
        try!(context.resolve(&mut self.sib));
        Ok(())
    }
}

impl outki::BinReader for PtrStructNotNull {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
            ptr: outki::Ptr::<PointedTo>::read(stream),
            sib: outki::Ptr::<PtrStructNotNull>::read(stream)
        }
    }
    fn resolve(&mut self, context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> { 
        try!(context.resolve_not_null(&mut self.ptr));
        try!(context.resolve_not_null(&mut self.sib));
        Ok(())
    }
}

impl shared::TypeDescriptor for PointedTo {
    const TAG : &'static str = "PointedTo";
}

impl shared::TypeDescriptor for PtrStruct {
    const TAG : &'static str = "PtrStruct";
}

impl shared::TypeDescriptor for PtrStructNotNull {
    const TAG : &'static str = "PtrStructNotNull";
}

impl outki::OutkiObj for PointedTo { }
impl outki::OutkiObj for PtrStruct { }
impl outki::OutkiObj for PtrStructNotNull { }

#[test]
pub fn unpack_simple()
{
    let mut pm = outki::BinPackageManager::new();
    let mut pkg = outki::Package::new();
    let data:[u8;5] = [123, 100, 2, 0, 0];
    pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    pm.insert(pkg);
    println!("RESOLVING!");
    let k = pm.resolve::<PointedTo>("pto1");
    assert_eq!(k.is_ok(), true);
    let rf = k.unwrap();
    assert_eq!(rf.value1, 123);
    assert_eq!(rf.value2, 256*2+100);
}

#[test]
pub fn unpack_complex()
{
    let mut pm = outki::BinPackageManager::new();
    let mut pkg = outki::Package::new();
    {
        let data:[u8;8] = [2, 0, 0, 0, 1, 0, 0, 0];
        pkg.insert(Some("ptr1"), tag_of::<PtrStruct>(), &data);
    } 
    {
        let data:[u8;8] = [3, 0, 0, 0, 0, 0, 0, 0];
        pkg.insert(Some("ptr2"), tag_of::<PtrStruct>(), &data);
    }    
    {
        let data:[u8;5] = [123, 100, 2, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    {
        let data:[u8;5] = [124, 100, 3, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    pm.insert(pkg);
    {
        let k = pm.resolve::<PtrStruct>("ptr1");
        assert_eq!(k.is_ok(), true);
        let w = k.unwrap();
        assert_eq!(w.ptr.get().unwrap().value1, 123);
        assert_eq!(w.ptr.get().unwrap().value2, 256*2+100);
        assert_eq!(w.sib.get().unwrap().ptr.get().unwrap().value1, 124);
        assert_eq!(w.sib.get().unwrap().ptr.get().unwrap().value2, 256*3+100);
        assert_eq!(w.sib.get().unwrap().sib.get().unwrap().ptr.get().unwrap().value1, 123);
        assert_eq!(w.sib.get().unwrap().sib.get().unwrap().ptr.get().unwrap().value2, 256*2+100);
    }
    {
        let k:outki::OutkiResult<outki::Ref<PtrStruct>> = pm.resolve("ptr2");
        assert_eq!(k.is_ok(), true);  
        let w = k.unwrap();
        assert_eq!(w.ptr.get().unwrap().value1, 124);
        assert_eq!(w.ptr.get().unwrap().value2, 256*3+100);
    }    
}

#[test]
pub fn unpack_not_null_complex()
{
    let mut pm = outki::BinPackageManager::new();
    let mut pkg = outki::Package::new();
    {
        let data:[u8;8] = [2, 0, 0, 0, 1, 0, 0, 0];
        pkg.insert(Some("ptr1"), tag_of::<PtrStructNotNull>(), &data);
    } 
    {
        let data:[u8;8] = [3, 0, 0, 0, 0, 0, 0, 0];
        pkg.insert(Some("ptr2"), tag_of::<PtrStructNotNull>(), &data);
    }    
    {
        let data:[u8;5] = [123, 100, 2, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    {
        let data:[u8;5] = [124, 100, 3, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    pm.insert(pkg);
    {
        let k = pm.resolve::<PtrStructNotNull>("ptr1");
        assert_eq!(k.is_ok(), true);
        let w = k.unwrap();
        assert_eq!(w.ptr.value1, 123);
        assert_eq!(w.ptr.value2, 256*2+100);
        assert_eq!(w.sib.ptr.value1, 124);
        assert_eq!(w.sib.ptr.value2, 256*3+100);
        assert_eq!(w.sib.sib.ptr.value1, 123);
        assert_eq!(w.sib.sib.ptr.value2, 256*2+100);
    }
}

#[test]
pub fn unpack_not_null_complex_failure()
{
    let mut pm = outki::BinPackageManager::new();
    let mut pkg = outki::Package::new();
    {
        let data:[u8;8] = [255, 255, 255, 255, 1, 0, 0, 0];
        pkg.insert(Some("ptr1"), tag_of::<PtrStructNotNull>(), &data);
    } 
    {
        let data:[u8;8] = [3, 0, 0, 0, 0, 0, 0, 0];
        pkg.insert(Some("ptr2"), tag_of::<PtrStructNotNull>(), &data);
    }    
    {
        let data:[u8;5] = [123, 100, 2, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    {
        let data:[u8;5] = [124, 100, 3, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    pm.insert(pkg);
    {
        let k = pm.resolve::<PtrStructNotNull>("ptr1");
        assert_eq!(k.is_err(), true);
    }
}