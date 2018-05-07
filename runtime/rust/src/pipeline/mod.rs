#![allow(unused_imports)]
#![allow(dead_code)]

use std::rc::Rc;
use std::any;
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

use super::loader;

pub trait Builder where Self : Sync + Send
{    
    fn build(&self, input:&Box<Any>) -> Box<Any>;
    fn make(&self);
}

struct ObjEntry
{
    object: Box<Any>    
}

pub struct BuildContext<'a>
{
    loader: &'a loader::SourceLoader
}

pub struct BuildRecord<'a>
{
    path: &'a str,
    ctx: &'a BuildContext<'a>,    
    deps: RefCell<HashSet<&'a str>>
}

impl<'a> BuildRecord<'a>
{
    fn new(ctx:&'a BuildContext<'a>, path:&'a str) -> BuildRecord<'a>
    {
        return BuildRecord {
            path: path,
            ctx: ctx,
            deps: RefCell::new(HashSet::new())
        }
    }
}

