use shared;
use shared::tag_of;
use shared::size_of;
use outki;
use std::rc::Rc;
use std::any::Any;

use outki::UnpackStatic;

struct IRootData { }

enum IRoot {
    Pure(IRootData),
    Sub1(Sub1),
    Sub2(Sub2),
    SubRoot(SubRoot)
}

enum SubRoot {
    Pure(SubRootData),
    SubSub1(SubSub1)
}

struct SubRootData {
    root: IRootData
}

struct SubSub1 {
    root: SubRootData,
    value: i32
}

struct Sub1 {
    root: IRootData,
    value: i32
}

struct Sub2 {
    root: IRootData,
    value: i32
}

impl shared::OutkiTypeDescriptor for IRootData { const TAG : &'static str = "IRootData"; const SIZE : usize = 0; }
impl shared::OutkiTypeDescriptor for SubRootData { const TAG : &'static str = "IRootData"; const SIZE : usize = <IRootData as shared::OutkiTypeDescriptor>::SIZE; }
impl shared::OutkiTypeDescriptor for SubRoot { const TAG : &'static str = "SubRoot"; const SIZE : usize = <SubRootData as shared::OutkiTypeDescriptor>::SIZE; }
impl shared::OutkiTypeDescriptor for SubSub1 {  const TAG : &'static str = "SubSub1"; const SIZE : usize = 4; }
impl shared::OutkiTypeDescriptor for IRoot { const TAG : &'static str = "IRoot"; const SIZE : usize = 0; }
impl shared::OutkiTypeDescriptor for Sub1 { const TAG : &'static str = "Sub1"; const SIZE : usize = 4; }
impl shared::OutkiTypeDescriptor for Sub2 { const TAG : &'static str = "Sub2"; const SIZE : usize = 4; }

impl outki::UnpackWithRefs<shared::BinLayout> for Sub1 {
    fn unpack(_refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {        
        let s1 = size_of::<IRootData>();
        let s2 = s1 + 4;
        return Self {
            root: IRootData::unpack(_refs, &data[0..s1]),
            value: i32::unpack(&data[s1..s2])
        }
    }
}

impl outki::UnpackWithRefs<shared::BinLayout> for Sub2 {
    fn unpack(_refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {        
        let s1 = size_of::<IRootData>();
        let s2 = s1 + 4;
        return Self {
            root: IRootData::unpack(_refs, &data[0..s1]),
            value: i32::unpack(&data[s1..s2])
        }
    }
}

impl outki::UnpackWithRefs<shared::BinLayout> for IRootData {
    fn unpack(_refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {
        IRootData { }
    }
}

impl outki::UnpackWithRefs<shared::BinLayout> for IRoot {
    fn unpack(pkg:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {
        unimplemented!();
    }
    fn unpack_with_type(pkg:&outki::RefsSource<shared::BinLayout>, data:&[u8], type_name:&str) -> Self {
        match type_name {
            <IRoot as shared::OutkiTypeDescriptor>::TAG => return IRoot::Pure(<IRootData as outki::UnpackWithRefs<shared::BinLayout>>::unpack(pkg, data)),
            <Sub1 as shared::OutkiTypeDescriptor>::TAG => return IRoot::Sub1(<Sub1 as outki::UnpackWithRefs<shared::BinLayout>>::unpack(pkg, data)),
            <Sub2 as shared::OutkiTypeDescriptor>::TAG => return IRoot::Sub2(<Sub2 as outki::UnpackWithRefs<shared::BinLayout>>::unpack(pkg, data)),            
            <SubRoot as shared::OutkiTypeDescriptor>::TAG => return IRoot::SubRoot(SubRoot::Pure(<SubRootData as outki::UnpackWithRefs<shared::BinLayout>>::unpack(pkg, data))),
            <SubSub1 as shared::OutkiTypeDescriptor>::TAG => return IRoot::SubRoot(SubRoot::SubSub1(<SubSub1 as outki::UnpackWithRefs<shared::BinLayout>>::unpack(pkg, data))),
            _ => return IRoot::Pure(IRootData { })
        }
    }    
}

impl outki::UnpackWithRefs<shared::BinLayout> for SubRootData {
    fn unpack(_refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {
        let s1 = size_of::<IRootData>();
        return Self {
            root: IRootData::unpack(_refs, &data[0..s1])            
        }
    }
}

impl outki::UnpackWithRefs<shared::BinLayout> for SubSub1 {
    fn unpack(_refs:&outki::RefsSource<shared::BinLayout>, data:&[u8]) -> Self {
        let s1 = size_of::<SubRootData>();
        return Self {
            root: SubRootData::unpack(_refs, &data[0..s1]),
            value: i32::unpack(&data[s1..])
        }
    }
}
/*
impl outki::TypeResolver<shared::BinLayout> for TypeResolver
{
    fn unpack_with_type(pkg:&outki::RefsSource<Self, shared::BinLayout>, data:&[u8], type_name:&str) -> Option<Rc<Any>> where Self : Sized {
        match type_name {
            <IRoot as shared::OutkiTypeDescriptor>::TAG => return IRoot::Pure(<IRootData as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)),
            <Sub1 as shared::OutkiTypeDescriptor>::TAG => return IRoot::Sub1(<Sub1 as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)),
            /*
            <Sub2 as shared::OutkiTypeDescriptor>::TAG => return IRoot::Sub2(<Sub2 as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data))) as Rc<Any>),            
            <SubRoot as shared::OutkiTypeDescriptor>::TAG => return IRoot::SubRoot(SubRoot::Pure(<SubRootData as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)))) as Rc<Any>),            
            <SubSub1 as shared::OutkiTypeDescriptor>::TAG => return IRoot::SubRoot(SubRoot::SubSub1(<SubSub1 as outki::UnpackWithRefs<Self, shared::BinLayout>>::unpack(pkg, data)))) as Rc<Any>),
            */
            _ => return None            
        }
    }
*/

#[test]
pub fn unpack_enum()
{
    let mut pm : outki::PackageManager<shared::BinLayout> = outki::PackageManager::new();
    let mut pkg = outki::Package::new();
    {
        let data:[u8;0] = [];
        pkg.insert(Some("subroot"), tag_of::<SubRoot>(), &data);        
    }
    {
        let data:[u8;4] = [123,0,0,0];
        pkg.insert(Some("sub1"), tag_of::<Sub1>(), &data);
    }
    pm.insert(pkg);
    let k:Option<Rc<IRoot>> = pm.resolve("subroot");
    assert_eq!(k.is_some(), true);  
    let l:Option<Rc<IRoot>> = pm.resolve("sub1");
    assert_eq!(l.is_some(), true);  
    let k = l.unwrap();
    if let &IRoot::Sub1(ref s1) = &(*k) {
        assert_eq!(s1.value, 123);
    } else {
        panic!("sub1 wasn't sub1");
    }
}
