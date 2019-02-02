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

use outki::BinReader;

impl outki::BinReader for PointedTo {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {            
            value1: u8::read(stream),
            value2: i32::read(stream)
        }
    }
    fn resolve(&mut self, context: &mut outki::BinResolverContext) { 
        let w: *const PointedTo = self;
        unsafe {
            println!("resolev {} {} {}", (*w).value1, (*w).value2, w as usize);
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
    fn resolve(&mut self, context: &mut outki::BinResolverContext) { 
        context.resolve(&mut self.ptr);
        context.resolve(&mut self.sib);
    }
}

impl shared::TypeDescriptor for PointedTo {
    const TAG : &'static str = "PointedTo";
}

impl shared::TypeDescriptor for PtrStruct {
    const TAG : &'static str = "PtrStruct";
}

impl outki::OutkiObj for PointedTo { }
impl outki::OutkiObj for PtrStruct { }

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
    assert_eq!(k.is_some(), true);
    assert_eq!(k.as_ref().unwrap().value1, 123);
    assert_eq!(k.as_ref().unwrap().value2, 256*2+100);
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
        assert_eq!(k.is_some(), true);
        let w = k.as_ref().unwrap();
        assert_eq!(w.ptr.get().unwrap().value1, 123);
        assert_eq!(w.ptr.get().unwrap().value2, 256*2+100);
        assert_eq!(w.sib.get().unwrap().ptr.get().unwrap().value1, 124);
        assert_eq!(w.sib.get().unwrap().ptr.get().unwrap().value2, 256*3+100);
        assert_eq!(w.sib.get().unwrap().sib.get().unwrap().ptr.get().unwrap().value1, 123);
        assert_eq!(w.sib.get().unwrap().sib.get().unwrap().ptr.get().unwrap().value2, 256*2+100);
    }
    {
        let k:Option<outki::Ref<PtrStruct>> = pm.resolve("ptr2");
        assert_eq!(k.is_some(), true);  
        let w = k.as_ref().unwrap();
        assert_eq!(w.ptr.get().unwrap().value1, 124);
        assert_eq!(w.ptr.get().unwrap().value2, 256*3+100);
    }    
}