#![feature(rc_downcast)]

pub mod mixki;
pub mod rtti;
pub fn putki_init()
{

}

pub use mixki::lexer::*;
pub use mixki::parser::*;
pub use rtti::*;


//pub mod putki
//{
//use pukti::mixki::lexer;
// pub use lexer::*;
	/*
	pub mod parser
	{
		use std::collections::HashMap;
		use std::collections::hash_map::Entry;
		use std::vec::Vec;
		use std::rc::Rc;
		
		use std::default::Default;
		use std::marker;
		use std::cell::RefCell;

		mod mixki_lexer;

	pub enum LexedData<'a>
	{
		Empty,
		Object { 
			kv : HashMap<&'a str, LexedData<'a>>, 
			id: &'a str,
			type_name: &'a str 
		},
		Array (Vec<&'a str>),
		Value (&'a str),
		StringLiteral(&'a str)
	}

	*/
//}