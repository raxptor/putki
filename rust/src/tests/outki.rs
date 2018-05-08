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

impl outki::UnpackWithRefs<TypeResolver, shared::BinLayout> for PointedTo {
    fn unpack(pkg:&outki::RefsSource<TypeResolver, shared::BinLayout>, data:&[u8]) -> Self {
        return PointedTo {
            value: 3
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

#[test]
pub fn create_package_manager()
{
    let pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
}
