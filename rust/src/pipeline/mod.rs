#![allow(unused_imports)]
#![allow(dead_code)]

use std::intrinsics;
use std::rc::Rc;
use std::any;
use std::any::TypeId;
use std::sync::Arc;
use std::any::Any;
use std::cell::Cell;
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
mod writer;

pub struct BuilderDesc {
    pub description: &'static str    
}

pub struct InputDeps {
}

pub trait BuildFields {
    fn build(&mut self, pl:&Pipeline, br:&mut BuildRecord) -> Result<(), shared::PutkiError> { return Ok(())}
}

pub trait Builder<T> where Self : Send + Sync {
    fn build(&self, input:&mut T) -> Result<(), shared::PutkiError> { return Ok(()) }
    fn desc(&self) -> BuilderDesc;
}

struct BuilderBox <T> {
    builder: Arc<Builder<T>>,
    object_type: any::TypeId
}

trait BuilderAny where Self : Send + Sync {
    fn object_type(&self) -> any::TypeId;
    fn build(&self, input:&mut any::Any) -> Result<(), shared::PutkiError>;
    fn accept(&self, b:&Any) -> bool;
}

impl<T> BuilderAny for BuilderBox<T> where T : Send + Sync + 'static {
    fn object_type(&self) -> any::TypeId {
        self.object_type
    }
    fn build(&self, input:&mut any::Any) -> Result<(), shared::PutkiError> {
        match self.builder.build(input.downcast_mut().unwrap()) {
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

pub struct BuildRecord
{
   pub result: Result<(), shared::PutkiError>,
    deps: HashSet<String>
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

pub struct Pipeline
{
    desc: PipelineDesc
}

impl Pipeline
{    
    pub fn new(desc:PipelineDesc) -> Self {
        Pipeline {
            desc: desc
        }
    }

    fn builders_for_obj(&self, obj: &any::Any) -> Vec<Arc<BuilderAny>> {
        self.desc.builders.iter().filter(|x| { x.accept(obj) }).map(|x| { x.clone() } ).collect()
    }

    pub fn build_field<T>(&self, _br:&mut BuildRecord, obj:&mut T) -> Result<(), shared::PutkiError> where T : 'static
    {
        for x in self.builders_for_obj(obj) {
            x.build(obj)?;
        }
        Ok(())
    }

    pub fn build<T>(&self, obj:&mut T) -> BuildRecord where T : 'static + BuildFields {        
        for x in self.builders_for_obj(obj) {
            let res = x.build(obj);
            if let Err(_) = res {
                return BuildRecord {
                    result: res,
                    deps: HashSet::new()
                }
            }
        }
        let mut br = BuildRecord {
            result: Ok(()),
            deps: HashSet::new()
        };
        br.result = (obj as &mut BuildFields).build(self, &mut br);        
        br
    }
}

pub trait Forcate where Self : Send + Sync {    
}
impl Forcate for Pipeline {}