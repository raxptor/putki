use shared;
use outki;
use std::rc::Rc;
use std::any::Any;
use shared::tag_of;

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
            <PointedTo as shared::OutkiTypeDescriptor>::TAG => return Some(Rc::new(<PointedTo as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)) as Rc<Any>),
            <PtrStruct as shared::OutkiTypeDescriptor>::TAG => return Some(Rc::new(<PtrStruct as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)) as Rc<Any>),
            _ => return None
        }
    }
}

impl shared::OutkiTypeDescriptor for PointedTo {
    const TAG : &'static str = "PointedTo";
    const SIZE : usize = 4;
}

impl shared::OutkiTypeDescriptor for PtrStruct {
    const TAG : &'static str = "PtrStruct";
    const SIZE : usize = 4;
}

#[test]
pub fn unpack_simple()
{
    let mut pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
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
    let mut pm : outki::PackageManager<TypeResolver, shared::BinLayout> = outki::PackageManager::new();
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