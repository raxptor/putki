use shared;
use outki;
use std::rc::Rc;
use shared::tag_of;

#[derive(Debug)]
struct PointedTo {
	value: i32
}

#[derive(Debug)]
struct PtrStruct {
    pub ptr: Option<Rc<PointedTo>>
}

use outki::UnpackStatic;

impl outki::UnpackWithRefs<shared::BinLayout> for PointedTo {
    fn unpack(_refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {
        return Self {
            value: i32::unpack(&data[0..4])
        }
    }
}

impl outki::UnpackWithRefs<shared::BinLayout> for PtrStruct {
    fn unpack(refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {
        return Self {
            ptr: <Option<Rc<PointedTo>> as outki::UnpackWithRefs<shared::BinLayout>>::unpack(refs, &data[0..4])
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
    let mut pm : outki::PackageManager<shared::BinLayout> = outki::PackageManager::new();
    let mut pkg = outki::Package::new();
    let data:[u8;4] = [100, 2, 0, 0];
    pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    pm.insert(pkg);
    let k:Option<Rc<PointedTo>> = pm.resolve("pto1");
    assert_eq!(k.is_some(), true);
    assert_eq!(k.unwrap().value, 256*2+100);
}

#[test]
pub fn unpack_complex()
{
    let mut pm : outki::PackageManager<shared::BinLayout> = outki::PackageManager::new();
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
        let data:[u8;4] = [100, 2, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    {
        let data:[u8;4] = [100, 3, 0, 0];
        pkg.insert(Some("pto1"), tag_of::<PointedTo>(), &data);
    }
    pm.insert(pkg);
    {
        let k:Option<Rc<PtrStruct>> = pm.resolve("ptr1");
        assert_eq!(k.is_some(), true);  
        assert_eq!(k.clone().unwrap().ptr.clone().unwrap().value, 256*2+100);
    }
    {
        let k:Option<Rc<PtrStruct>> = pm.resolve("ptr2");
        assert_eq!(k.is_some(), true);  
        assert_eq!(k.clone().unwrap().ptr.clone().unwrap().value, 256*3+100);
    }    
}