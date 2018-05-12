use inki::source;
use inki::lexer;
use std::marker::PhantomData;
use std::rc::Rc;
use std::fmt;
use std::result;

#[derive(Clone)]
enum PtrTarget
{
    Null,
    ObjPath {
        context : source::InkiPtrContext,
        path : String
    },
    InlineObject {
        path : String,
        type_name : String,
        context : source::InkiPtrContext,
        data: lexer::LexedKv
    }
}

pub struct Ptr<Target> where Target : source::ParseFromKV
{
    target: PtrTarget,
    _m : PhantomData<Rc<Target>>
}

impl<Target> fmt::Debug for Ptr<Target> where Target : source::ParseFromKV {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match &self.target {
            &PtrTarget::Null  => { write!(f, "Null pointer").ok(); },
            &PtrTarget::ObjPath { ref path, .. } => { write!(f, "Ptr path[{}]", &path).ok(); },
            &PtrTarget::InlineObject { ref type_name, ref path, .. } => { write!(f, "Ptr inline object type[{}] path={}", type_name, path).ok(); }
        }
        return Ok(());
    }
}

impl<Target> Ptr<Target> where Target : source::ParseFromKV
{
    pub fn new(context : source::InkiPtrContext, path: &str) -> Ptr<Target>
    {
        return Ptr {
            target: PtrTarget::ObjPath {
                context: context,
                path: String::from(path)
            },
            _m: PhantomData { }
        }
    }
    pub fn new_inline(context : source::InkiPtrContext, kv: &lexer::LexedKv, type_name: &str, path: &str) -> Ptr<Target>
    {
        return Ptr {
            target: PtrTarget::InlineObject {
                context: context,
                path: String::from(path),
                type_name: String::from(type_name),
                data: kv.clone()
            },
            _m: PhantomData { }
        }
    }    
    pub fn null() -> Ptr<Target>
    {
        return Ptr {
            target: PtrTarget::Null,
            _m: PhantomData { }
        }
    }    
}

impl<T> Default for Ptr<T> where T : source::ParseFromKV {
    fn default() -> Self {
        return Ptr::null();
    }
}

impl<T> Clone for Ptr<T> where T : source::ParseFromKV {
    fn clone(&self) -> Self {
        Self {
            target : self.target.clone(),
            _m : PhantomData
        }        
    }
}

impl<T> Ptr<T> where T : source::ParseFromKV + 'static
{
    pub fn resolve(&self) -> Option<Rc<T>> {
        match &self.target {
            &PtrTarget::Null => return None,
            &PtrTarget::ObjPath { ref context, ref path } => {
                if let &Some(ref trk) = &context.tracker {
                    trk.follow(path);
                }
                if let source::ResolveStatus::Resolved(ptr) = source::resolve_from::<T>(context, path) {
                    return Some(ptr);
                } else {
                    return None;
                } 
            }
            &PtrTarget::InlineObject { ref data, ref context, ref type_name, .. } => {
                return Some(Rc::new(<T as source::ParseFromKV>::parse_with_type(data, context, type_name)));
            }
        }
    }
    pub fn unwrap(&self) -> Rc<T> {
        return self.resolve().unwrap();
    }
    pub fn unwrap_unique(&self) -> Box<T> {
        return Box::new((*self.resolve().unwrap()).clone());
    }
}

pub fn ptr_from_data<T>(context : &source::InkiPtrContext, ld:&lexer::LexedData) -> Ptr<T> where T : source::ParseFromKV {
    match ld {
        &lexer::LexedData::Value(ref path) => Ptr::new(context.clone(), path),
        &lexer::LexedData::StringLiteral(ref path) => Ptr::new(context.clone(), path),
        &lexer::LexedData::Object { ref type_name, ref kv, ref id } => Ptr::new_inline(context.clone(), kv, type_name, id),
        _ => Ptr::null()
    }
}
