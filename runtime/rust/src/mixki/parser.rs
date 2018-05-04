use std::collections::HashMap;
use mixki::lexer::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::marker;

pub type ResolvedDB<'a, ResolvedType> = HashMap<&'a str, ResolvedType>;

pub struct ResolveContext<'a, Base> {
	pub unparsed: &'a LexedDB<'a>,
	pub resolved: RefCell<ResolvedDB<'a, Base>>
}

pub trait ParseSpecific<Base> {
	fn parse_to_rc(ctx:&ResolveContext<Base>, obj: &LexedKv) -> Rc<Self> where Self: marker::Sized;
}	

pub trait ParseGeneric {
	fn parse(ctx:&ResolveContext<Self>, obj: &LexedData) -> Option<Self> where Self: marker::Sized;
}

pub fn resolve<'a, Base>(ctx:&'a ResolveContext<'a, Base>, path: &'a str) -> Option<Base> where Base : ParseGeneric + Clone
{	
	println!("trying to resolve [{}]", path);
	{
		let chk = ctx.resolved.borrow_mut();
		match chk.get(path) {
			Some(x) => return Some(x.clone()),
			_ => { }
		}
		drop(chk);
	}
	match ctx.unparsed.get(path)
	{
		Some(ref x) => {						
			println!(" => found unparsed [{}]!", path);						
			match Base::parse(ctx, x)
			{
				Some(x) => { 
					println!("   => managed to resolve");
					ctx.resolved.borrow_mut().insert(path, x.clone()); 
					return Some(x);
				}
				None => { return Option::None; }
			}						
		}
		_ => {
			println!("Did not find [{}] unparsed.", path);					
			return Option::None;
		}
	}
}