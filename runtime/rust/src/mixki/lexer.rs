use std::collections::HashMap;
use std::vec::Vec;
use std::str::FromStr;
use std::default;

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

impl<'a> default::Default for LexedData<'a>
{
	fn default() -> LexedData<'a> { return LexedData::Empty; }
}

pub type LexedKv<'a> = HashMap<&'a str, LexedData<'a>>;
pub type LexedDB<'a> = HashMap<&'a str, LexedData<'a>>;

struct ScanResult<'a> {
	cont: &'a str,
	data: LexedData<'a>
}

fn parse_val<T : FromStr + Default>(val: &LexedData) -> T
{
	match val {
		&LexedData::Value(ref x) => {
			match T::from_str(x) {
				Ok(val) => { return val; }
				_ => { }
			}
		}		
		_ => { }
	}
	println!("expected int");
	return Default::default();
}

pub fn get_int(kv: &LexedKv, name: &str, default: i32) -> i32
{
	match &kv.get(name) {
		&Some(ref val) => {
			return parse_val(val);
		}
		_ => { }
	}	
	return default;
}

/*
fn parse_list<F>(data: &str, parser:F, term:char) -> &str
	where F: Fn(&str) -> &str
{
	let mut cur = data;
	let mut it = data.char_indices().enumerate();
	loop {
		match it.next() {
			None => return "",
			Some(ref x) => {                 
				let value = &x.1;
				if value.1 == term {
					return &cur[value.0 ..];
				} else if !value.1.is_whitespace() {
					cur = parser(cur);
					it = cur.char_indices().enumerate();
				}
			}
		}
	}
	return "";
}
*/

fn make_parse_error(err: &str) -> ScanResult
{
	println!("Parse error. {}", err);
	return ScanResult {
		cont: "",
		data: LexedData::Empty
	}
}

fn is_syntax_delimiter(c : char) -> bool
{
	return c.is_whitespace() || c == ',' || c == '}' || c == ']' || c == ':' || c == '=';
}

fn parse_keyword_or_string<'a>(data: &'a str) -> ScanResult<'a>
{
	let mut it = data.char_indices().enumerate(); 
	let mut inside_string = false;
	let mut string_start = 0;
	let mut escaped = false;
	loop {
		match it.next() {
			None => { 
				return ScanResult {
					cont: &data[1 ..],
					data: LexedData::Empty
				}
			},
			Some(ref x) => {
				let value = &x.1;
				if inside_string {
					if escaped {
						escaped = false;
						continue;
					} else if value.1 == '\\' {
						inside_string = true;
						string_start = value.0;
						escaped = true;
						continue;
					}
				} else if value.1 == '\"' {
					inside_string = true;
					continue;
				}
				if (inside_string && value.1 == '\"') || (!inside_string && is_syntax_delimiter(value.1)) {
					if !inside_string {
						return ScanResult {
							cont: &data[value.0 ..],
							data: LexedData::Value(&data[0 .. value.0])
						};
					} else {
						return ScanResult {
							cont: &data[(value.0 + 1) ..],
							data: LexedData::StringLiteral(&data[(string_start + 1) .. value.0])
						};
					}
				}
			}
		}
	}    
}

fn parse_auto_detect<'a>(data: &'a str) -> ScanResult<'a>
{
	// first should be {
	let mut it = data.char_indices().enumerate();
	loop {
		match it.next() {
			None => return make_parse_error("Unexpected end at auto."),
			Some(ref x) => {
				let value = &x.1;
				if value.1.is_whitespace() {
					continue;
				} else if value.1 == '{' {
					return parse_object_data(&data[(value.0 + 1) ..]);
				} else if value.1 == '@' {
					return parse_object_with_header(&data[(value.0+1) ..]);
				} else if value.1 == '[' {
					// parse array
					// return parse_object_data(&cur[value.0 ..]);
				} else {
					return parse_keyword_or_string(&data[value.0 ..]);
				}
			}
		}
	}	
}

fn parse_object_data<'a>(data: &'a str) -> ScanResult<'a>
{
	let mut cur = data;
	let mut it = data.char_indices().enumerate();
	let mut field_name = "";
	let mut kv = HashMap::new();
	//
	loop {
		match it.next() {
			None => return ScanResult {
				cont: "",
				data: LexedData::Empty
			},
			Some(ref x) => {                 
				let value = &x.1;
				if value.1.is_whitespace() || value.1 == ',' {
					continue;
				} else if value.1 == '}' {
					return ScanResult {
						cont: &cur[(value.0 + 1) ..],
						data: LexedData::Object {
							id: "",
							type_name: "",
							kv: kv
						}
					};
				} else if field_name.is_empty() {
					let res = parse_keyword_or_string(&cur[value.0 ..]);
					match res.data {
						LexedData::Value(ref v) => {
							field_name = v;
							cur = res.cont;
							it = cur.char_indices().enumerate();
						},
						LexedData::StringLiteral(ref v) => {
							field_name = v;
							cur = res.cont;
							it = cur.char_indices().enumerate();
						}                        
						_ => {
							println!("Parse error. Could not parse value");
							return ScanResult {
								cont: "",
								data: LexedData::Empty
							}
						}
					}
				} else if value.1 == ':' || value.1 == '=' {
					let res = parse_auto_detect(&cur[value.0 + value.1.len_utf8() ..]);
					cur = res.cont;
					it = cur.char_indices().enumerate();
					kv.insert(field_name, res.data);
					field_name = "";
				} else {
					let a = &cur[value.0 .. (value.0 + 4)];
					println!("Syntax error in object. {}", &a);
					return make_parse_error("Syntax error in object.");
				}
			}
		}
	}
}

// Parse one @type id { block }
fn parse_object_with_header<'a>(data: &'a str) -> ScanResult<'a>
{
	let cur = data;
	let mut it = data.char_indices().enumerate();
	let mut type_end = 0;
	let mut id_begin = 0;
	let mut id_end = 0;		
	loop {
		match it.next() {
			None => return ScanResult {
				cont: "",
				data: LexedData::Empty
			},
			Some(ref x) => {                 
				let value = &x.1;
				if value.1.is_whitespace() && type_end == 0
				{
					type_end = value.0;						
				}
				else if id_begin == 0 && !value.1.is_whitespace() && type_end != 0
				{
					id_begin = value.0;
				}
				else if id_begin != 0 && id_end == 0 && value.1.is_whitespace()
				{
					id_end = value.0;
				}
				if value.1 == '{'
				{
					if type_end == 0 { type_end = value.0 }
					if id_end == 0 { 
						if id_begin != 0 {
							id_end = value.0 
						}
					}
					let res = parse_object_data(&cur[(value.0 + 1) ..]);
					match res.data {
						LexedData::Object { kv, .. } => {
							return ScanResult {

								cont: res.cont,
								data: LexedData::Object {                                    
									id: &cur[id_begin .. id_end],
									type_name: &cur[0..type_end],
									kv: kv
								}
							};
						},
						_ => {
							return make_parse_error("Aah!");
						}
					}
				}
			}
		}
	}
}

pub fn lex_file(data: &str) -> LexedDB
{
	let mut cur = data;
	let mut it = data.char_indices().enumerate();
	let mut objs = HashMap::new();
	loop {
		match it.next() {
			None => return objs,
			Some(ref x) => {                 
				let value = &x.1;
				if value.1 == '@' {
					let result = parse_object_with_header(&cur[(value.0+1)..]);
					match result.data {
						LexedData::Object { kv, id, type_name } => {
							println!("parsed object with type=[{}] id=[{}]", type_name, id);
							objs.insert(id, LexedData::Object {
								kv: kv,
								id: id,
								type_name: type_name
							});
						}
						_ => { 
							println!("parse error; expected object");
							return HashMap::new();
						}
					}
					cur = result.cont;
					it = cur.char_indices().enumerate();
				}
			} 
		}
	}	
}