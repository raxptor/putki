use shared;
use outki;
use std::rc::Rc;
use shared::tag_of;
use pipeline::writer;
use pipeline::writer::BinWriter;

#[derive(Debug)]
struct PointedTo {
    value1: u8,
    value2: i32    
}

#[derive(Debug)]
struct PtrStruct {
    pub ptr: Option<Rc<PointedTo>>
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
            ptr: <Option<Rc<PointedTo>> as outki::BinReader>::read(stream)
        }
    }
}

impl shared::TypeDescriptor for PointedTo {
    const TAG : &'static str = "PointedTo";
}

impl shared::TypeDescriptor for PtrStruct {
    const TAG : &'static str = "PtrStruct";
}

#[test]
pub fn unpack_simple()
{
    let mut pm = outki::BinPackageManager::new();
    let mut pkg = outki::Package::new();
    let data:[u8;5] = [123, 100, 2, 0, 0];
    pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    pm.insert(pkg);
    let k:Option<Rc<PointedTo>> = pm.resolve("pto1");
    assert_eq!(k.is_some(), true);
    assert_eq!(k.clone().unwrap().value1, 123);
    assert_eq!(k.clone().unwrap().value2, 256*2+100);
}

#[test]
pub fn unpack_complex()
{
    let mut pm = outki::BinPackageManager::new();
    let mut pkg = outki::Package::new();
    {
        let data:[u8;4] = [2, 0, 0, 0];
        pkg.insert(Some("ptr1"), tag_of::<PtrStruct>(), &data);
    } 
    {
        let data:[u8;4] = [3, 0, 0, 0];
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
        let k:Option<Rc<PtrStruct>> = pm.resolve("ptr1");
        assert_eq!(k.is_some(), true);  
        assert_eq!(k.clone().unwrap().clone().ptr.clone().unwrap().clone().value1, 123);
        assert_eq!(k.clone().unwrap().clone().ptr.clone().unwrap().clone().value2, 256*2+100);        
    }
    {
        let k:Option<Rc<PtrStruct>> = pm.resolve("ptr2");
        assert_eq!(k.is_some(), true);  
        assert_eq!(k.clone().unwrap().ptr.clone().unwrap().value1, 124);
        assert_eq!(k.clone().unwrap().ptr.clone().unwrap().value2, 256*3+100);
    }    
}