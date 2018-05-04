use std::collections::HashMap;
use mixki::lexer::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::marker;
use std::default;

pub type ResolvedDB<'a, ResolvedType> = HashMap<&'a str, ResolvedType>;

pub struct ResolveContext<'a, Base> {
	pub unparsed: &'a LexedDB<'a>,
	pub resolved: RefCell<ResolvedDB<'a, Base>>
}

pub trait ParseSpecific<'a, 'b, Base> {
	fn parse(ctx:&'a ResolveContext<'b, Base>, obj: &'b LexedKv) -> Self where Self: marker::Sized;
	fn parse_or_default(ctx:&'a ResolveContext<'b, Base>, obj: Option<&'b LexedData>) -> Self where Self: marker::Sized + default::Default
	{		
		match obj.and_then(|ld| { match ld { &LexedData::Object { ref kv, .. } => { return Some(kv) }, _ => return None } })
		{
			Some(x) => return Self::parse(ctx, x),
			None => return Default::default()
		}
	}
}

pub trait ParseGeneric<'a, 'b, Base> {
	fn parse(ctx:&'a ResolveContext<'b, Self>, type_name: &'b str, kv : &'b LexedKv) -> Option<Self> where Self: marker::Sized;
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
		impl<'a, 'b> parser::ParseGeneric<'a, 'b, $targetName> for $targetName
		{
			fn parse(_ctx:&'a parser::ResolveContext<'b, Self>, type_name: &'b str, _obj: &'b lexer::LexedKv) -> Option<Self>
			{
				match type_name
				{
					$(
						stringify!($v) => return Some($targetName::$v(rc::Rc::new(parser::ParseSpecific::parse(_ctx, _obj)))),
					)*
					_ => return None
				}
			}
		}		
	)
}

/*
mod test {
	use std::rc;
	use super::super::parser as parser;
	use super::super::lexer as lexer;
	mod mixki {
		pub struct TestA { }
		pub struct TestB { }
	}
	make_any!(TestRC, TestA, TestB);
}
*/


pub fn resolve<'a, 'b, Base, Target>(ctx:&'a ResolveContext<'b, Base>, path: &'b str) -> Option<Rc<Target>> 
	where Base : ParseGeneric<'a, 'b, Base> + Clone, Base : Cast<Target>
{
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
		match x
		{
			&LexedData::Object{ref type_name, ref id, ref kv} => {
				return Base::parse(ctx, type_name, kv).and_then(|x| {
					match x.cast()
					{
						Some(c) => {
							println!("=> parsed {} from unparsed with type {}", id, type_name);
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

pub fn resolve_from_value<'a, 'b, Base, Target>(ctx:&'a ResolveContext<'b, Base>, value: &'b LexedData) -> Option<Rc<Target>> 
	where Base : ParseGeneric<'a, 'b, Base> + Clone, Base : Cast<Target>, Target : ParseSpecific<'a, 'b, Base>
{
	match value {
		&LexedData::Object { ref kv, type_name, .. } => {
				if type_name.is_empty()
				{
					println!("Empty type name! Assuming type and force parse.");
					return Some(Rc::new(Target::parse(ctx, kv)));
				}
				return Base::parse(ctx, type_name, kv).and_then(|obj| {
					match obj.cast() 
					{
						Some(c) => { return Some((*c).clone()); }
						None => { println!("Type mismatch in annoymous object!"); return None; }
					}
				});
		},
		&LexedData::StringLiteral ( path ) => { return resolve(ctx, path) }
		_ => { println!("unexpected contents at pointer"); return None }
	}
}