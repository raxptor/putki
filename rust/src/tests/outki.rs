use shared;
use outki;
use outki::*;

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

struct ArrayPodStruct {
    pub values: Vec<i32>
}

struct ArrayStruct {
    pub values: Vec<PointedTo>
}

struct PtrStructNotNull {
    pub ptr: outki::Ptr<PointedTo>,
    pub sib: outki::Ptr<PtrStructNotNull>
}

impl outki::BinLoader for PointedTo {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {            
            value1: u8::read(stream),
            value2: i32::read(stream)
        }
    }
    fn resolve(&mut self, _context: &mut BinResolverContext) -> outki::OutkiResult<()> { Ok(()) }
}

impl outki::BinLoader for ArrayPodStruct {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
            values: outki::BinReader::read(stream)
        }
    }
    fn resolve(&mut self, _context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> {
        Ok(())
    }
}

impl outki::BinLoader for ArrayStruct {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
            values: outki::BinLoader::read(stream)
        }
    }
    fn resolve(&mut self, _context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> {
        self.values.resolve(_context)?;
        Ok(())
    }
}



impl outki::BinLoader for PtrStruct {
    fn read(stream:&mut outki::BinDataStream) -> Self {
        Self {
            ptr: outki::NullablePtr::<PointedTo>::read(stream),
            sib: outki::NullablePtr::<PtrStruct>::read(stream)
        }
    }
    fn resolve(&mut self, context: &mut outki::BinResolverContext) -> outki::OutkiResult<()> { 
        self.ptr.resolve(context)?;
        self.sib.resolve(context)?;
        Ok(())
    }
}

impl outki::BinLoader for PtrStructNotNull {
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

impl shared::TypeDescriptor for ArrayPodStruct {
    const TAG : &'static str = "ArrayPodStruct";
}

impl shared::TypeDescriptor for ArrayStruct {
    const TAG : &'static str = "ArrayPodStruct";
}

impl outki::OutkiObj for PointedTo { }
impl outki::OutkiObj for PtrStruct { }
impl outki::OutkiObj for PtrStructNotNull { }
impl outki::OutkiObj for ArrayPodStruct { }
impl outki::OutkiObj for ArrayStruct { }

impl outki::PackageRandomAccess for Vec<u8>
{
   fn read_chunk(&self, begin:usize, end:usize, f:&mut FnMut(outki::OutkiResult<&[u8]>) -> outki::OutkiResult<()>) -> outki::OutkiResult<()> {       
        (*f)(Ok(&self[begin..end]))
    }
}

#[test]
pub fn unpack_simple()
{    
    let mut pkg_data:Vec<u8> = Vec::new();
    let mut pkg = outki::PackageManifest::new();        
    pkg.add_obj::<PointedTo>(&mut pkg_data, Some("pto1"), &[123, 100, 2, 0, 0]);

    let mut pm = outki::BinPackageManager::new();
    pm.insert(Package::new(pkg, Box::new(pkg_data)));

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
    let mut pkg_data:Vec<u8> = Vec::new();
    let mut pkg = outki::PackageManifest::new();            
    pkg.add_obj::<PtrStruct>(&mut pkg_data, Some("ptr1"), &[2, 0, 0, 0, 1, 0, 0, 0]);
    pkg.add_obj::<PtrStruct>(&mut pkg_data, Some("ptr2"), &[3, 0, 0, 0, 0, 0, 0, 0]);
    pkg.add_obj::<PointedTo>(&mut pkg_data, Some("pto1"), &[123, 100, 2, 0, 0]);
    pkg.add_obj::<PointedTo>(&mut pkg_data, Some("pto2"), &[124, 100, 3, 0, 0]);
    let mut pm = outki::BinPackageManager::new();
    pm.insert(Package::new(pkg, Box::new(pkg_data)));

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
    let mut pkg_data:Vec<u8> = Vec::new();
    let mut pkg = outki::PackageManifest::new();            
    pkg.add_obj::<PtrStructNotNull>(&mut pkg_data, Some("ptr1"), &[2, 0, 0, 0, 1, 0, 0, 0]);
    pkg.add_obj::<PtrStructNotNull>(&mut pkg_data, Some("ptr2"), &[3, 0, 0, 0, 0, 0, 0, 0]);
    pkg.add_obj::<PointedTo>(&mut pkg_data, Some("pto1"), &[123, 100, 2, 0, 0]);
    pkg.add_obj::<PointedTo>(&mut pkg_data, Some("pto2"), &[124, 100, 3, 0, 0]);
    let mut pm = outki::BinPackageManager::new();
    pm.insert(Package::new(pkg, Box::new(pkg_data)));
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
    let mut pkg_data:Vec<u8> = Vec::new();
    let mut pkg = outki::PackageManifest::new();            
    pkg.add_obj::<PtrStructNotNull>(&mut pkg_data, Some("ptr1"), &[255, 255, 255, 255, 1, 0, 0, 0]);
    let mut pm = outki::BinPackageManager::new();
    pm.insert(Package::new(pkg, Box::new(pkg_data)));
    {
        let k = pm.resolve::<PtrStructNotNull>("ptr1");
        assert_eq!(k.is_err(), true);
    }
}

#[test]
pub fn unpack_array_pod()
{
    let mut pkg_data:Vec<u8> = Vec::new();
    let mut pkg = outki::PackageManifest::new();            
    pkg.add_obj::<ArrayPodStruct>(&mut pkg_data, Some("obj0"), &[3, 0, 0, 0, 0, 0, 0, 0, 111, 0, 0, 0, 222, 0, 0, 0, 123, 0, 0, 0]);
    let mut pm = outki::BinPackageManager::new();    
    pm.insert(Package::new(pkg, Box::new(pkg_data)));
    {
        let k = pm.resolve::<ArrayPodStruct>("obj0");
        assert_eq!(k.is_err(), false);
        let obj = k.unwrap();
        assert_eq!(obj.values.len(), 3);
        assert_eq!(obj.values[0], 111);
        assert_eq!(obj.values[1], 222);
        assert_eq!(obj.values[2], 123);
    }
}

#[test]
pub fn unpack_array_struct()
{
    let mut pkg_data:Vec<u8> = Vec::new();
    let mut pkg = outki::PackageManifest::new();            
    pkg.add_obj::<ArrayStruct>(&mut pkg_data, Some("obj0"), &[2, 0, 0, 0, 0, 0, 0, 0, 10, 11, 0, 0, 0, 12, 255, 255, 255, 255]);
    let mut pm = outki::BinPackageManager::new();    
    pm.insert(Package::new(pkg, Box::new(pkg_data)));
    {
        let k = pm.resolve::<ArrayStruct>("obj0");
        assert_eq!(k.is_err(), false);
        let obj = k.unwrap();
        assert_eq!(obj.values.len(), 2);
        assert_eq!(obj.values[0].value1, 10);
        assert_eq!(obj.values[0].value2, 11);
        assert_eq!(obj.values[1].value1, 12);
        assert_eq!(obj.values[1].value2, -1);
    }
}