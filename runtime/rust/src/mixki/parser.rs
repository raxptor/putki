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
	fn parse(ctx:&ResolveContext<Self>,	type_name: &str, kv : &LexedKv) -> Option<Self> where Self: marker::Sized;
}

pub trait Cast<Target> {
	fn cast(&self) -> Option<&Rc<Target>>;
}

#[macro_export]
macro_rules! make_any {	
	($targetName:ident, $($v:ident),*) => (
		#[derive(Clone)]
		pub enum $targetName
		{			
			$($v (rc::Rc<mixki::$v>),)*
		}
		$(
			impl parser::Cast<mixki::$v> for $targetName
			{
				fn cast(&self) -> Option<&rc::Rc<mixki::$v>>
				{
					match self {
						&$targetName::$v(ref x) => return Some(&x),
						_ => return None
					}
				}
			}			
		)*
		impl parser::ParseGeneric for $targetName
		{
			fn parse(_ctx:&parser::ResolveContext<Self>, type_name: &str, _obj: &lexer::LexedKv) -> Option<Self>
			{
				match type_name
				{
					$(
						stringify!($v) => return Some($targetName::$v(parser::ParseSpecific::parse_to_rc(_ctx, _obj))),						
					)*
					_ => return None
				}		
			}
		}
	)
}

pub fn resolve<'a, 'b, Base, Target>(ctx:&'a ResolveContext<'b, Base>, path: &'b str) -> Option<Rc<Target>> 
	where Base : ParseGeneric + Clone, Base : Cast<Target>
{	
	println!("trying to resolve [{}]", path);
	{		
		let res = ctx.resolved.borrow_mut().get(path).and_then(|x| {
			return x.cast();
		}).and_then(|y| {
			return Some((*y).clone());
		});
		if res.is_some() {
			return res;
		}
	}
	return ctx.unparsed.get(path).and_then(|x| {
		println!(" => found unparsed [{}]!", path);
		match (x)
		{
			&LexedData::Object{ref type_name, ref id, ref kv} => {
				return Base::parse(ctx, type_name, kv).and_then(|x| {
					println!("   => managed to resolve it.");
					match x.cast()
					{
						Some(c) => {
							println!("   => and it was correct type, too.");
							ctx.resolved.borrow_mut().insert(path, x.clone());
							return Some((*c).clone());
						}
						None => { return None }
					}
				});
			},
			_ => {
				unreachable!("Why is there a non-object in the unparsed data?!");
			} 
		}
	});
}