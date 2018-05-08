use shared;
use outki;
use std::rc::Rc;
use std::any::Any;

#[derive(Debug)]
struct PointedTo {
	value: i32
}

#[derive(Debug)]
struct PtrStruct {
    pub ptr: Option<Rc<PointedTo>>
}

#[derive(Debug)]
struct TypeResolver {	
}

use outki::UnpackStatic;

impl outki::UnpackWithRefs<TypeResolver, shared::BinLayout> for PointedTo {
    fn unpack(_refs:&outki::RefsSource<TypeResolver, shared::BinLayout>, data:&[u8]) -> Self {
        return Self {
            value: i32::unpack(&data[0..4])
        }
    }
}

impl outki::UnpackWithRefs<TypeResolver, shared::BinLayout> for PtrStruct {
    fn unpack(refs:&outki::RefsSource<TypeResolver, shared::BinLayout>, data:&[u8]) -> Self {
        return Self {
            ptr: <Option<Rc<PointedTo>> as outki::UnpackWithRefs<TypeResolver, shared::BinLayout>>::unpack(refs, &data[0..4])
        }
    }
}

impl outki::TypeResolver<shared::BinLayout> for TypeResolver
{
    fn unpack_with_type(pkg:&outki::RefsSource<Self, shared::BinLayout>, data:&[u8], type_name:&str) -> Option<Rc<Any>> where Self : Sized {
        match type_name {
            "PointedTo" => return Some(Rc::new(<PointedTo as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)) as Rc<Any>),
            "PtrStruct" => return Some(Rc::new(<PtrStruct as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)) as Rc<Any>),
            _ => return None
        }
    }
}

impl shared::PutkiTypeCast for PointedTo { }
impl shared::PutkiTypeCast for PtrStruct { }

#[test]
pub fn unpack_simple()
{
    let mut pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
    let mut pkg = outki::Package::new();
    let data:[u8;4] = [100, 2, 0, 0];
    pkg.insert(Some(String::from("pto1")), String::from("PointedTo"), &data);
    pm.insert(pkg);
    let k:Option<Rc<PointedTo>> = pm.resolve("pto1");
    assert_eq!(k.is_some(), true);
    assert_eq!(k.unwrap().value, 256*2+100);
}

#[test]
pub fn unpack_complex()
{
    let mut pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
    let mut pkg = outki::Package::new();
    {
        let data:[u8;4] = [2, 0, 0, 0];
        pkg.insert(Some(String::from("ptr1")), String::from("PtrStruct"), &data);
    } 
    {
        let data:[u8;4] = [3, 0, 0, 0];
        pkg.insert(Some(String::from("ptr2")), String::from("PtrStruct"), &data);
    }    
    {
        let data:[u8;4] = [100, 2, 0, 0];
        pkg.insert(Some(String::from("pto1")), String::from("PointedTo"), &data);
    }
    {
        let data:[u8;4] = [100, 3, 0, 0];
        pkg.insert(Some(String::from("pto1")), String::from("PointedTo"), &data);
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

