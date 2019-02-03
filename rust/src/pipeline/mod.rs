#![allow(unused_imports)]
#![allow(dead_code)]

use std::borrow::BorrowMut;
use std::rc::Rc;
use std::any;
use std::any::TypeId;
use std::sync::Arc;
use std::any::Any;
use std::cell::Cell;
use std::sync::RwLock;
use std::thread;
use std::ops;
use std::cell::RefCell;
use std::sync::*;
use std::default::Default;
use std::vec;
use std::marker::PhantomData;
use std::collections::*;
use inki::ptr::PtrInkiResolver;
use shared;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path;
use inki;
use source;
use ptr;
use source::WriteAsText;
use shared::PutkiError;

pub mod writer;
use writer::*;

pub struct BuilderDesc {
    pub description: &'static str    
}

pub struct InputDeps {
}

pub trait BuildFields {
    fn build_fields(&mut self, _pl:&Pipeline, _br:&mut BuildRecord) -> Result<(), shared::PutkiError> { return Ok(())}
}

pub trait Builder<T> where Self : Send + Sync {
    fn build(&self, _br:&mut BuildRecord, input:&mut T) -> Result<(), shared::PutkiError>;
    fn desc(&self) -> BuilderDesc;
}

struct BuilderBox <T> {
    builder: Arc<Builder<T>>,
    object_type: any::TypeId
}

trait BuilderAny where Self : Send + Sync {
    fn object_type(&self) -> any::TypeId;
    fn build(&self, br:&mut BuildRecord, input:&mut any::Any) -> Result<(), shared::PutkiError>;
    fn accept(&self, b:&Any) -> bool;
}

impl<T> BuilderAny for BuilderBox<T> where T : Send + Sync + 'static {
    fn object_type(&self) -> any::TypeId {
        self.object_type
    }
    fn build(&self, br:&mut BuildRecord, input:&mut any::Any) -> Result<(), shared::PutkiError> {
        match self.builder.build(br, input.downcast_mut().unwrap()) {
            Ok(x) => return Ok(x),
            Err(x) => return Err(x)
        }
    }    
    fn accept(&self, b:&Any) -> bool {
        b.is::<T>()
    }
}

struct ObjEntry
{
    object: Arc<Any>    
}

// One per object and per builder.
pub struct BuildRecord
{
    path: String,    
    type_tag: &'static str,
    res_base: path::PathBuf,
    built_obj: Option<Box<BuildResultObj>>,
    visited: HashSet<String>,
    deps: HashMap<String, Box<ObjDepRef>>,
    error: Option<String>,
    success: bool,        
}

impl BuildRecord
{
    pub fn is_ok(&self) -> bool {
        return self.success;
    }
    pub fn get_path<'a>(&'a self) -> &'a str {
        return self.path.as_str();
    }
    pub fn create_object<T>(&mut self, tag:&str, obj:T) -> ptr::Ptr<T> where T:BuildCandidate + Send + Sync + Default + source::ParseFromKV {
        let tmp_path = format!("{}!{}", &self.path, tag);
        ptr::Ptr::new_temp_object(tmp_path.as_str(), Arc::new(obj))
    }
    pub fn load_file(&mut self, res_path:&str) -> Result<Vec<u8>, shared::PutkiError> {
        let np = self.res_base.join(res_path);
        println!("mapping file {:?} + {:?} => {:?}", self.res_base, res_path, np);
        let mut f = File::open(np)?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
    pub fn built_object<'a>(&'a self) -> Option<&'a BuildResultObj> {
        if let &Some(ref b) = &self.built_obj {
            Some(&(**b))
        } else {
            None
        }
    }
}

impl ptr::Tracker for BuildRecord {
    fn follow(&mut self, path:&str) {
        println!("Builder visited [{}]", path);
        self.visited.insert(String::from(path));
    }
}

pub struct PipelineDesc
{
    source: Arc<source::ObjectLoader>,
    builders: Vec<Arc<BuilderAny>>,
    res_base: path::PathBuf
}

impl PipelineDesc {
    pub fn new(source:Arc<source::ObjectLoader>, res_base: &path::Path) -> PipelineDesc {
        PipelineDesc {
            source: source,
            builders: Vec::new(),
            res_base: path::PathBuf::from(res_base)                
        }
    }
    pub fn add_builder<T, K>(mut self, bld: T) -> Self where T : Builder<K> + 'static, K : Send + Sync + 'static {
        self.builders.push(Arc::new(BuilderBox { 
            builder: Arc::new(bld),
            object_type: any::TypeId::of::<K>()
        }));
        self
    }
}

pub trait BuildCandidate where Self : 'static + Send + Sync + BuildFields {
    fn as_any_ref(&mut self) -> &mut Any;
    fn build(&mut self, p:&Pipeline, br: &mut BuildRecord) -> Result<(), shared::PutkiError>;
    fn scan_deps(&self, _p:&Pipeline, _br: &mut BuildRecord) { }
}

pub struct Pipeline
{
    desc: PipelineDesc,
    to_build: RwLock<Vec<BuildRequest>>,
    inserted: RwLock<HashSet<String>>,
    built: RwLock<HashMap<String, BuildRecord>>
}

pub trait InkiObj : source::WriteAsText + Send + Sync + BuildCandidate + shared::TypeDescriptor + source::ParseFromKV + writer::BinSaver + Default
{
}

struct PtrBox<T> where T : 'static + Send + Sync
{
    pub ptr: ptr::Ptr<T>
}

trait BuildInvoke where Self : Send + Sync
{
    fn build(&self, p:&Pipeline, br:&BuildRequest);
}

pub trait BuildResultObj where Self : Send + Sync + BinSaver {
    fn write_object(&self, output:&mut String) -> Result<(), PutkiError>;
}

trait ObjDepRef where Self : 'static + Send + Sync { }

impl<T> BuildResultObj for PtrBox<T> where T : 'static + InkiObj
{
    fn write_object(&self, output:&mut String) -> Result<(), PutkiError> {
        if let Some(x) = self.ptr.get_owned_object() {
            <T as WriteAsText>::write_text(&x, output)
        } else {
            Err(PutkiError::ObjectNotFound)
        }
    }
}

impl<T> BinSaver for PtrBox<T> where T : 'static + InkiObj
{    
    fn write(&self, data: &mut Vec<u8>, refwriter: &PackageRefs) -> Result<(), PutkiError> {
        if let Some(x) = self.ptr.get_owned_object() {
            <T as BinSaver>::write(&x, data, refwriter)
        } else {
            Err(PutkiError::ObjectNotFound)
        }
    }    
}

impl<T> BinSaver for Vec<T> where T : 'static + BinSaver
{    
    fn write(&self, data: &mut Vec<u8>, refwriter: &PackageRefs) -> Result<(), PutkiError> {
        self.len().write(data);
        for x in self.iter() {
            (*x).write(data, refwriter)?;
        }
        Ok(())
    }
}

impl<T> BinWriter for Vec<T> where T : 'static + BinWriter
{    
    fn write(&self, data: &mut Vec<u8>) {
        self.len().write(data);
        for x in self.iter() {
            (*x).write(data);
        }
    }
}

impl<T> ObjDepRef for PtrBox<T> where T : 'static + Sync + Send + InkiObj
{

}

impl<T> BuildInvoke for PtrBox<T> where T : 'static + InkiObj
{
    fn build(&self, p:&Pipeline, br:&BuildRequest)
    {
        if let Some(obj) = inki::ptr::PtrInkiResolver::resolve_notrack(&self.ptr) {
            let mut br = BuildRecord {
                error: None,
                type_tag: <T as shared::TypeDescriptor>::TAG,
                success: true,
                path: br.path.clone(),
                deps: HashMap::new(),
                visited: HashSet::new(),
                built_obj: None,
                res_base: p.desc.res_base.clone()
            };            
            let mut clone = (*obj).clone();
            {
                let bc : &mut BuildCandidate = &mut clone;                
                if let Err(_res) = bc.build(p, &mut br) {
                    br.success = false;
                    println!("ERROR in build of {}", br.path);
                }
                bc.scan_deps(p, &mut br);
            }
            br.built_obj = Some(Box::new(PtrBox { ptr: ptr::Ptr::new_temp_object(&br.path, Arc::new(clone))}));
            p.on_build_done(br);
        } else {
            panic!("Unable to resolve ptr {:?}", self.ptr);
        }
    }
}

struct BuildRequest {
    path: String,
    invoker: Box<BuildInvoke>
}

impl Pipeline
{    
    pub fn new(desc:PipelineDesc) -> Self {
        Pipeline {
            desc: desc,
            to_build: RwLock::new(Vec::new()),
            built: RwLock::new(HashMap::new()),
            inserted: RwLock::new(HashSet::new())
        }
    }

    fn on_build_done(&self, br:BuildRecord) {
        self.built.write().unwrap().insert(br.path.clone(), br);
    }

    fn builders_for_obj(&self, obj: &any::Any) -> Vec<Arc<BuilderAny>> {
        self.desc.builders.iter().filter(|x| { x.accept(obj) }).map(|x| { x.clone() } ).collect()
    }

    pub fn build_field<T>(&self, br:&mut BuildRecord, obj:&mut T) -> Result<(), shared::PutkiError> where T : 'static
    {
        for x in self.builders_for_obj(obj) {
            x.build(br, obj)?;
        }
        Ok(())
    }

    pub fn add_output_dependency<T>(&self, br:&mut BuildRecord, ptr: &ptr::Ptr<T>) where T : 'static + InkiObj {
        if let Some(path) = ptr.get_target_path() {
            self.build_ptr(ptr);
            br.deps.insert(String::from(path), Box::new(PtrBox { ptr: (*ptr).clone() }));
        }
    }

    pub fn build_ptr<T>(&self, ptr : &ptr::Ptr<T>) where T : 'static + InkiObj {
        let path = String::from(ptr.get_target_path().unwrap());
        if self.insert_path_to_build(path.as_ref()) {
            let mut lk = self.to_build.write().unwrap();        
            lk.push(BuildRequest {
                path: path,
                invoker: Box::new(PtrBox::<T> { ptr: ptr.clone() })
            });
        }
    }

    pub fn make_src_ptr<T>(&self, path:&str) -> ptr::Ptr<T> {
        ptr::Ptr::new(Arc::new(source::InkiResolver::new(self.desc.source.clone())), path)
    }

    pub fn build_as<T>(&self, path:&str) where T : 'static + InkiObj {
        if self.insert_path_to_build(path) {        
            let mut lk = self.to_build.write().unwrap();       
            lk.push(BuildRequest {
                path: String::from(path),
                invoker: Box::new(PtrBox::<T> { ptr: self.make_src_ptr(path) })
            });
        }
    }

    pub fn insert_path_to_build(&self, path:&str) -> bool
    {
        let mut lk = self.inserted.write().unwrap();
        lk.insert(String::from(path))
    }

    pub fn take(&self) -> bool
    {
        let request = {        
            let mut lk = self.to_build.write().unwrap();        
            if lk.len() == 0 {
                return false;
            }
            lk.remove(0)
        };
        request.invoker.build(self, &request);
        return true;
    }

    pub fn peek_build_records(&self) -> LockResult<RwLockReadGuard<HashMap<String, BuildRecord>>>
    {
        return self.built.read();
    }

    // This function is re-entrant when building fields.
    pub fn build<T>(&self, br:&mut BuildRecord, obj:&mut T) -> Result<(), shared::PutkiError> where T : 'static + BuildFields {
        for x in self.builders_for_obj(obj) {
            x.build(br, obj)?;
        }
        (obj as &mut BuildFields).build_fields(&self, br)?;
        Ok(())
    }
}

pub trait Forcate where Self : Send + Sync {    
}
impl Forcate for Pipeline {}