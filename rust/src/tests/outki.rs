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
    pub ptr: Rc<PointedTo>
}

#[derive(Debug)]
struct TypeResolver {	
}

use outki::UnpackStatic;

impl outki::UnpackWithRefs<TypeResolver, shared::BinLayout> for PointedTo {
    fn unpack(pkg:&outki::RefsSource<TypeResolver, shared::BinLayout>, data:&[u8]) -> Self {
        return PointedTo {
            value: i32::unpack(&data[0..4])
        }
    }
}

impl outki::TypeResolver<shared::BinLayout> for TypeResolver
{
    fn unpack_with_type(pkg:&outki::RefsSource<Self, shared::BinLayout>, data:&[u8], type_name:&str) -> Option<Rc<Any>> where Self : Sized {
        match type_name {
            "PointedTo" => return Some(Rc::new(<PointedTo as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)) as Rc<Any>),
            _ => return None
        }
    }
}

impl shared::PutkiTypeCast for PointedTo { }

#[test]
pub fn create_package_manager()
{
    let pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
}

#[test]
pub fn unpack_simple()
{
    let mut pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
    let mut pkg = outki::Package::new();
    let data:[u8;4] = [100, 2, 0, 0];
    pkg.insert(Some(String::from("pto1")), Some(String::from("PointedTo")), &data);
    pm.insert(pkg);
    let k:Option<Rc<PointedTo>> = pm.resolve("pto1");
    assert_eq!(k.is_some(), true);
    assert_eq!(k.unwrap().value, 256*2+100);
}
