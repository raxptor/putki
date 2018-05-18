use inki::source;
use inki::lexer;
use std::fmt;
use std::result;
use std::sync::Arc;
use seahash;

#[derive(Clone)]
enum PtrTarget<Target>
{
    Null,
    ObjPath {
        resolver : Arc<source::InkiResolver>,
        path : String
    },
    InlineObject {
        path : String,
        type_name : String,
        resolver : Arc<source::InkiResolver>,
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

pub trait Resolver<T>
{
    fn resolve(&self, trk: &mut Tracker) -> Option<Arc<T>>;
}

pub trait Tracker where Self : Send + Sync {
    fn follow(&mut self, path:&str);
}

impl<T> Resolver<T> for Ptr<T> where T : 'static + source::ParseFromKV
{
    fn resolve(&self, trk: &mut Tracker) -> Option<Arc<T>> {
        match &self.target {
            &PtrTarget::ObjPath { ref path, .. } => {
                trk.follow(path);
                (self as &PtrInkiResolver<T>).resolve()
            },
            _ => (self as &PtrInkiResolver<T>).resolve()
        }
    }    
}

impl<T> PtrInkiResolver<T> for Ptr<T> where T : 'static + source::ParseFromKV
{
    fn resolve(&self) -> Option<Arc<T>> {
        match &self.target {
            &PtrTarget::Null => return None,
            &PtrTarget::ObjPath { ref resolver, ref path } => {
                if let source::ResolveStatus::Resolved(ptr) = source::resolve_from::<T>(resolver, path) {
                    return Some(Arc::new( (*ptr).clone() ));
                } else {
                    return None;
                } 
            }
            &PtrTarget::InlineObject { ref data, ref resolver, ref type_name, .. } => {
                return Some(Arc::new(<T as source::ParseFromKV>::parse_with_type(data, resolver, type_name)));
            }
            &PtrTarget::TempObject { ref object, .. } => Some(object.clone())
        }
    }
    fn unwrap(&self) -> Arc<T> {
        return (self as &PtrInkiResolver<T>).resolve().unwrap();
    }
    fn unwrap_unique(&self) -> Box<T> {
        return Box::new((*(self as &PtrInkiResolver<T>).resolve().unwrap()).clone());
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

impl<Target> Ptr<Target>
{
    pub fn get_target_path<'a>(&'a self) -> Option<&'a str>
    {
        match &self.target {
            &PtrTarget::Null => None,
            &PtrTarget::ObjPath { ref path, .. } => Some(path.as_str()),
            &PtrTarget::TempObject { ref path, .. } => Some(path.as_str()),
            &PtrTarget::InlineObject { ref path, .. } => Some(path.as_str()),
        }
    }

    pub fn new(resolver : Arc<source::InkiResolver>, path: &str) -> Ptr<Target> {
        return Ptr {
            target: PtrTarget::ObjPath {
                resolver: resolver,
                path: String::from(path)
            }
        }
    }

    pub fn new_inline(resolver : Arc<source::InkiResolver>, kv: &lexer::LexedKv, type_name: &str, path: &str) -> Ptr<Target> {
        return Ptr {
            target: PtrTarget::InlineObject {
                resolver: resolver,
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

pub fn ptr_from_data<T>(resolver : &Arc<source::InkiResolver>, ld:&lexer::LexedData) -> Ptr<T> where T : source::ParseFromKV {
    match ld {
        &lexer::LexedData::Value(ref path) => Ptr::new(resolver.clone(), path),
        &lexer::LexedData::StringLiteral(ref path) => Ptr::new(resolver.clone(), path),
        &lexer::LexedData::Object { ref type_name, ref kv, ref id } => {
            if id.len() == 0 {
                let mut n_id = String::from(":anon:");
                let hash = seahash::hash(lexer::kv_to_string(kv).as_bytes());
                n_id.push_str(type_name);
                n_id.push(':');
                n_id.push_str(id);
                n_id.push(':');
                n_id.push_str(hash.to_string().as_str());
                Ptr::new_inline(resolver.clone(), kv, type_name, n_id.as_str())
            } else {
                Ptr::new_inline(resolver.clone(), kv, type_name, id)
            }
        },
        _ => Ptr::null()
    }
}
