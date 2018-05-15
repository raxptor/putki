#![allow(unused_imports)]
#![allow(dead_code)]

use std::borrow::BorrowMut;
use std::intrinsics;
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
use std::collections::HashMap;
use std::collections::HashSet;
use shared;
use inki;
use source;
use ptr;

mod writer;


pub struct BuilderDesc {
    pub description: &'static str    
}

pub struct InputDeps {
}

pub trait BuildFields {
    fn build_fields(&mut self, _pl:&Pipeline, _br:&mut BuildRecord) -> Result<(), shared::PutkiError> { return Ok(())}
}

pub trait Builder<T> where Self : Send + Sync {
    fn build(&self, _input:&mut T) -> Result<(), shared::PutkiError> { return Ok(()) }
    fn build2(&self, _br:&mut BuildRecord, input:&mut T) -> Result<(), shared::PutkiError> { return self.build(input); }
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
        match self.builder.build2(br, input.downcast_mut().unwrap()) {
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
    error: Option<String>,
    success: bool,
    deps: HashSet<String>,

    // helper
    context: Arc<source::InkiPtrContext>,
    created: Vec<(String, Box<BuildCandidate>)>
}

impl BuildRecord
{
    pub fn is_ok(&self) -> bool {
        return self.success;
    }
    pub fn get_path<'a>(&'a self) -> &'a str {
        return self.path.as_str();
    }
    pub fn create_object<T>(&mut self, tag:&str, obj:T) -> ptr::Ptr<T> where T:BuildCandidate + source::ParseFromKV + Send + Sync + Default {                
        let tmp_path = format!("{}!{}", &self.path, tag);
        println!("Created object with path [{}]!", tmp_path);
        self.created.push((tmp_path.clone(), Box::new(obj)));
        ptr::Ptr::new(self.context.clone(), tmp_path.as_str())
    }
}

pub struct PipelineDesc
{
    source: Arc<source::ObjectLoader>,
    builders: Vec<Arc<BuilderAny>>    
}

impl PipelineDesc {
    pub fn new(source:Arc<source::ObjectLoader>) -> PipelineDesc {
        PipelineDesc {
            source: source,
            builders: Vec::new()
        }
    }
    pub fn add_builder<T, K>(mut self, bld: T) -> Self where T : Builder<K> + 'static, K : Send + Sync + 'static {
        self.builders.push(Arc::new(BuilderBox { 
            builder: Arc::new(bld),
            object_type: any::TypeId::of::<K>()
        }));
        unsafe {println!("adding with {}", intrinsics::type_name::<K>()); }
        self
    }
}

pub trait BuildCandidate where Self : 'static + Send + Sync + BuildFields {
    fn as_any_ref(&mut self) -> &mut Any;
    fn build(&mut self, p:&Pipeline, br: &mut BuildRecord);
    fn scan_deps(&self, _p:&Pipeline, _br: &mut BuildRecord) { }
}

pub struct Pipeline
{
    desc: PipelineDesc,
    to_build: RwLock<Vec<BuildRequest>>
}

struct BuildRequest {
    path: String,
    obj: Box<BuildCandidate>,
    context: Arc<source::InkiPtrContext>
}

impl Pipeline
{    
    pub fn new(desc:PipelineDesc) -> Self {
        Pipeline {
            desc: desc,
            to_build: RwLock::new(Vec::new())
        }
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

    pub fn add_output_dependency<T>(&self, br:&mut BuildRecord, ptr: &ptr::Ptr<T>) where T : 'static + source::ParseFromKV + BuildCandidate {
        if let Some(path) = ptr.get_target_path() {
            println!("adding output dependency {}", path);
            self.build_as::<T>(path);
            br.deps.insert(String::from(path));
        }
    }

    pub fn build_as<T>(&self, path:&str) where T : 'static + source::ParseFromKV + BuildCandidate {
        let context = Arc::new(source::InkiPtrContext {
            tracker: None,
            source: Arc::new(source::InkiResolver::new(self.desc.source.clone())),
        });
        if let source::ResolveStatus::Resolved(ptr) = source::resolve_from::<T>(&context, path) {
            let mut lk = self.to_build.write().unwrap();
            lk.push(BuildRequest {
                path: String::from(path),
                obj: Box::new((*ptr).clone()),
                context: context                
            });
        } else {
            panic!("FAILED TO RESOLVE [{}]", path);
        }
    }

    pub fn take(&self) -> bool
    {
        let mut request = {        
            let mut lk = self.to_build.write().unwrap();        
            if lk.len() == 0 {
                return false;
            }
            lk.remove(0)
        };
        let mut br = BuildRecord {
            error: None,
            success: true,
            path: request.path,
            context: request.context.clone(),
            deps: HashSet::new(),
            created: Vec::new()
        };
        request.obj.build(self, &mut br);
        if br.success {
            request.obj.scan_deps(self, &mut br);
        }
        if br.created.len() > 0 {
            // TODO: Unsure what to do with created exactly, output deps will be tracked in
            // scan_deps if they are used anyway...
            println!("Build of {} created {} items", br.path, br.created.len());
            /* let mut lk = self.to_build.write().unwrap();
            for x in br.created.drain(..) {
                lk.push(BuildRequest {
                    path : x.0,
                    obj: x.1,
                    context: br.context.clone()
                });
            }      
            */      
        }
        return true;
    }

    // This function is re-entrant when building fields.
    pub fn build<T>(&self, br:&mut BuildRecord, obj:&mut T) where T : 'static + BuildFields {
        if !br.success {
            return;
        }
        for x in self.builders_for_obj(obj) {
            let res = x.build(br, obj);
            if let Err(_) = res {
                br.error = Some(String::from("An error occured in the pipeline"));
                br.success = false;
                return;
            }
        }
        if let Err(_) = (obj as &mut BuildFields).build_fields(&self, br) {
            br.error = Some(String::from("An error occured in the pipeline"));
            br.success = false;
        }
    }
}

pub trait Forcate where Self : Send + Sync {    
}
impl Forcate for Pipeline {}