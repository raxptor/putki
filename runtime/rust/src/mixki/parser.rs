use std::collections::HashMap;
use mixki::lexer::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::marker;
use std::default;
use std::any::Any;

pub type ResolvedDB<'a> = HashMap<&'a str, Rc<Any>>;

pub struct ResolveContext<'a, ParseDef> {
	pub def:ParseDef,
	pub unparsed: &'a LexedDB<'a>,
	pub resolved: RefCell<ResolvedDB<'a>>
}

pub trait ParseSpecific<'a, 'b, ParseDef> where Self : Sized {
	fn parse(ctx:&'a ResolveContext<'b, ParseDef>, obj: &'b LexedKv) -> Self;
	fn parse_or_default(ctx:&'a ResolveContext<'b, ParseDef>, obj: Option<&'b LexedData>) -> Self where Self: default::Default
	{		
		match obj.and_then(|ld| { match ld { &LexedData::Object { ref kv, .. } => { return Some(kv) }, _ => return None } })
		{
			Some(x) => return Self::parse(ctx, x),
			None => return Default::default()
		}
	}
}

pub trait ParseToRoot<'a, 'b, ParseDef, Sub> where Sub : ParseSpecific<'a, 'b, ParseDef> {
	fn parse(ctx:&'a ResolveContext<'b, ParseDef>, obj: &'b LexedKv) -> Self;
}

pub trait ParseToPure<'a, 'b, ParseDef> {
	fn parse(ctx:&'a ResolveContext<'b, ParseDef>, obj: &'b LexedKv) -> Self;
}

pub trait ParseGeneric<'a, 'b, ParseDef> {
	fn parse(ctx:&'a ResolveContext<'b, ParseDef>, type_name: &'b str, obj: &'b LexedKv) -> Option<Rc<Any>>;
}

#[macro_export]
macro_rules! impl_subtype_parse {
	($parser:path, $base:path, $baseInner:path, $sub:path) => {		
		impl<'a, 'b> parser::ParseToRoot<'a, 'b, $parser, $sub> for $base
		{
			fn parse(ctx:&'a parser::ResolveContext<'b, $parser >, obj: &'b lexer::LexedKv) -> ($base)
			{
				let child : $sub = parser::ParseSpecific::<'a, 'b, $parser >::parse(ctx, obj);
				let inner : $baseInner = parser::ParseSpecific::<'a, 'b, $parser >::parse(ctx, obj);
				return Self {
					child: rc::Rc::new(child),
					type_id: any::TypeId::of::< $sub >(),
					inner: inner
				}
			}
		}
	}
}

#[macro_export]
macro_rules! impl_root_parse {
	($parser:path, $base:path, $baseInner:path) => {		
		impl<'a, 'b> parser::ParseToPure<'a, 'b, $parser > for $base
		{
			fn parse(ctx:&'a parser::ResolveContext<'b, $parser >, obj: &'b lexer::LexedKv) -> Self
			{
				let inner : $baseInner = parser::ParseSpecific::<'a, 'b, $parser >::parse(ctx, obj);
				return Self {
					child: rc::Rc::new(0),
					type_id: any::TypeId::of::<Self>(),
					inner: inner
				}
			}
		}
	}
}


pub fn resolve<'a, 'b, ParseDef, Target>(ctx:&'a ResolveContext<'b, ParseDef>, path: &'b str) -> Option<Rc<Target>> 
	where ParseDef : ParseGeneric<'a, 'b, ParseDef>, Target : 'static
{
	{		
		let res = ctx.resolved.borrow_mut().get(path).and_then(|x| {
			return x.clone().downcast::<Target>().ok();
		});
		if res.is_some() {
			return res;
		}
	}
	return ctx.unparsed.get(path).and_then(|x| {
		match x
		{
			&LexedData::Object{ref type_name, ref id, ref kv} => {
				return ParseDef::parse(ctx, type_name, kv).and_then(|x| {
					match x.downcast::<Target>().ok()
					{
						Some(c) => {
							println!("=> parsed {} from unparsed with type {}", id, type_name);
							ctx.resolved.borrow_mut().insert(path, c.clone());
							return Some(c);
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

pub fn resolve_from_value<'a, 'b, ParseDef, Target>(ctx:&'a ResolveContext<'b, ParseDef>, value: &'b LexedData) -> Option<Rc<Target>> 
	where ParseDef : ParseGeneric<'a, 'b, ParseDef>, Target : ParseSpecific<'a, 'b, ParseDef> + 'static
{
	match value {
		&LexedData::Object { ref kv, type_name, .. } => {
				if type_name.is_empty()
				{
					println!("Empty type name! Assuming type and force parse.");
					return Some(Rc::new(Target::parse(ctx, kv)));
				}
				return ParseDef::parse(ctx, type_name, kv).and_then(|obj| {
					match obj.downcast::<Target>()
					{
						Ok(c) => { return Some(c); },
						_ => { println!("Type mismatch in annoymous object!"); return None; }
					}
				});
		},
		&LexedData::StringLiteral ( path ) => { return resolve(ctx, path) }
		_ => { println!("unexpected contents at pointer"); return None }
	}
}
