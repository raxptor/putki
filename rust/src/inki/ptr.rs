use inki::source;
use inki::lexer;
use std::fmt;
use std::result;
use std::sync::Arc;

#[derive(Clone)]
enum PtrTarget<Target>
{
    Null,
    ObjPath {
        context : Arc<source::InkiPtrContext>,
        path : String
    },
    InlineObject {
        path : String,
        type_name : String,
        context : Arc<source::InkiPtrContext>,
        data: lexer::LexedKv
    },
    TempObject {
        path: String,
        object: Arc<Target>
    }
}

pub struct Ptr<Target> where Target : 'static + Sized
{
    target: PtrTarget<Target>
}

pub trait PtrInkiResolver<T>
{
    fn resolve(&self) -> Option<Arc<T>>;
    fn unwrap(&self) -> Arc<T>;
    fn unwrap_unique(&self) -> Box<T>;
}

impl<T> PtrInkiResolver<T> for Ptr<T> where T : 'static + source::ParseFromKV
{
    fn resolve(&self) -> Option<Arc<T>> {
        match &self.target {
            &PtrTarget::Null => return None,
            &PtrTarget::ObjPath { ref context, ref path } => {
                if let &Some(ref trk) = &context.tracker {
                    trk.follow(path);
                }
                if let source::ResolveStatus::Resolved(ptr) = source::resolve_from::<T>(context, path) {
                    return Some(Arc::new( (*ptr).clone() ));
                } else {
                    return None;
                } 
            }
            &PtrTarget::InlineObject { ref data, ref context, ref type_name, .. } => {
                return Some(Arc::new(<T as source::ParseFromKV>::parse_with_type(data, context, type_name)));
            }
            &PtrTarget::TempObject { ref object, .. } => Some(object.clone())
        }
    }
    fn unwrap(&self) -> Arc<T> {
        return self.resolve().unwrap();
    }
    fn unwrap_unique(&self) -> Box<T> {
        return Box::new((*self.resolve().unwrap()).clone());
    }    
}


impl<Target> fmt::Debug for Ptr<Target> where Target : source::ParseFromKV {
    fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        match &self.target {
            &PtrTarget::Null  => { write!(f, "Null pointer").ok(); },
            &PtrTarget::ObjPath { ref path, .. } => { write!(f, "Ptr path[{}]", &path).ok(); },
            &PtrTarget::InlineObject { ref type_name, ref path, .. } => { write!(f, "Ptr inline object type[{}] path={}", type_name, path).ok(); }
            &PtrTarget::TempObject { ref path, .. } => { write!(f, "Temp object path={}", path).ok(); }
        }
        return Ok(());
    }
}

impl<Target> Ptr<Target> where Target : source::ParseFromKV
{
    pub fn new(context : Arc<source::InkiPtrContext>, path: &str) -> Ptr<Target> {
        return Ptr {
            target: PtrTarget::ObjPath {
                context: context,
                path: String::from(path)
            }
        }
    }

    pub fn new_inline(context : Arc<source::InkiPtrContext>, kv: &lexer::LexedKv, type_name: &str, path: &str) -> Ptr<Target> {
        return Ptr {
            target: PtrTarget::InlineObject {
                context: context,
                path: String::from(path),
                type_name: String::from(type_name),
                data: kv.clone()
            }
        }
    }    

    pub fn new_temp_object(path: &str, obj: Arc<Target>) -> Ptr<Target> where Target : source::ParseFromKV {
        return Ptr {
            target: PtrTarget::TempObject {
                path: String::from(path),
                object: obj
            }
        }
    }

    pub fn null() -> Ptr<Target> {
        return Ptr {
            target: PtrTarget::Null
        }
    }

    pub fn get_target_path<'a>(&'a self) -> Option<&'a str>
    {
        match &self.target {
            &PtrTarget::Null => None,
            &PtrTarget::ObjPath { ref path, .. } => Some(path.as_str()),
            &PtrTarget::TempObject { ref path, .. } => Some(path.as_str()),
            _ => None
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
        }        
    }
}

impl<T> Ptr<T> where T : 'static
{

}

pub fn ptr_from_data<T>(context : &Arc<source::InkiPtrContext>, ld:&lexer::LexedData) -> Ptr<T> where T : source::ParseFromKV {
    match ld {
        &lexer::LexedData::Value(ref path) => Ptr::new(context.clone(), path),
        &lexer::LexedData::StringLiteral(ref path) => Ptr::new(context.clone(), path),
        &lexer::LexedData::Object { ref type_name, ref kv, ref id } => Ptr::new_inline(context.clone(), kv, type_name, id),
        _ => Ptr::null()
    }
}
