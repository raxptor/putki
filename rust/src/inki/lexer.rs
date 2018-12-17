use std::collections::HashMap;
use std::vec::Vec;
use std::str::FromStr;
use std::string::ToString;
use std::default;
use std::slice;


#[derive(Clone, PartialEq)]
pub enum LexedData
{
	Empty,
	Object { 
		kv : HashMap<String, LexedData>, 
		id: String,
		type_name: String 
	},
	Array (Vec<LexedData>),
	Value (String),
	StringLiteral(String),
	Comment
}

pub type LexedKv = HashMap<String, LexedData>;

impl default::Default for LexedData
{
	fn default() -> LexedData { return LexedData::Empty; }
}

pub struct ScanResult<'a> 
{
	pub cont: &'a str,
	pub data: LexedData
}

pub fn escape_string(input:&String) -> String
{
	let mut s = String::with_capacity(input.len() + 32);
	s.push('\"');
	s.push_str(&input.replace("\"", "\\\""));
	s.push('\"');
	s
}

pub fn kv_to_string(kv: &HashMap<String, LexedData>) -> String
{
	let mut tmp = String::new();
	tmp.push('{');
	for (key, value) in kv {
		tmp.push_str(key);
		tmp.push_str(":");
		tmp.push_str(value.to_string().as_str());
		tmp.push_str(",");
	}
	tmp.push('}');
	tmp
}

impl ToString for LexedData
{
	fn to_string(&self) -> String {
		let mut tmp = String::new();
		match self {
			&LexedData::Object { ref kv, ref id, ref type_name } => {
				if type_name.len() > 0 {
					tmp.push('@');
					tmp.push_str(type_name);
					tmp.push(' ');
					if id.len() > 0 {
						tmp.push_str(id);
						tmp.push(' ');
					}					
				}
				tmp.push_str(kv_to_string(kv).as_str());
			},
			&LexedData::Value(ref val) => tmp.push_str(val),
			&LexedData::StringLiteral(ref val) => {
				tmp.push_str(&escape_string(val));
			}
			&LexedData::Array(ref vec) => {
				tmp.push('[');
				for val in vec {				
					tmp.push_str(val.to_string().as_str());
					tmp.push(',');
				}
				tmp.push(']');
			}
			&LexedData::Comment => { },
			&LexedData::Empty => { }
		}
		return tmp;
	}
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
	println!("expected other type");
	return Default::default();
}

pub fn get_value<T>(data: Option<&LexedData>, default: T) -> T where T : Default + FromStr
{
	data.map(|v| { parse_val(v) }).unwrap_or(default)
}

pub fn get_int(data: Option<&LexedData>, default: i32) -> i32
{
	data.map(|v| { parse_val(v) }).unwrap_or(default)
}

pub fn get_bool(data: Option<&LexedData>, default: bool) -> bool
{
	data.and_then(|val| {
		match val {
			&LexedData::Value(ref x) => {
				match x.as_ref() {
					"True" => return Some(true),
					"true" => return Some(true),
					"1" => return Some(true),
					_ => return None
				}
			}
			_ => return None
		}
	}).unwrap_or(default)
}

pub fn get_string(data: Option<&LexedData>, default: &str) -> String
{
	data.map(|val| {
		match val {
			&LexedData::Value(ref x) => x.clone(),
			&LexedData::StringLiteral(ref x) => x.clone(),
			_ => String::from(default)
		}
	}).unwrap_or(String::from(default))
}

pub fn get_array<'a>(data: Option<&'a LexedData>) -> Option<slice::Iter<'a, LexedData>>
{
	data.and_then(|v| { 
		match v {
			&LexedData::Array(ref arr) => Some(arr.iter()),
			_ => None
		}
	})
}

pub fn get_object<'a>(data: Option<&'a LexedData>) -> Option<(&'a LexedKv, &'a str)>
{
	data.and_then(|val| {
		match val {
			&LexedData::Object{ref kv, ref type_name, ..} => Some((kv, type_name.as_ref())),
			_ => None
		}
	})
}

pub fn get_kv<'a>(data: Option<&'a LexedData>) -> Option<&'a LexedKv>
{
	data.and_then(|val| {
		match val {
			&LexedData::Object{ref kv, ..} => Some(kv),
			_ => None
		}
	})
}

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
	return c.is_whitespace() || c == ',' || c == '}' || c == ']' || c == ':' || c == '=' || c == ']' || c == '}' || c == '{' || c == '[';
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
					string_start = value.0;
					continue;
				}
				if (inside_string && value.1 == '\"') || (!inside_string && is_syntax_delimiter(value.1)) {
					if !inside_string {
						if value.0 > 0 {
							return ScanResult {							
								cont: &data[value.0 ..],
								data: LexedData::Value(String::from(&data[0 .. value.0]))
							};
						} else {
							return ScanResult {							
								cont: "",
								data: LexedData::Empty
							};
						}
					} else {
						return ScanResult {
							cont: &data[(value.0 + 1) ..],
							data: LexedData::StringLiteral(String::from(&data[(string_start + 1) .. value.0]))
						};
					}
				}
			}
		}
	}    
}

pub fn parse_array<'a>(data: &'a str) -> ScanResult<'a>
{
	let mut cur = data;
	let mut it = data.char_indices().enumerate();		
	if let Some(first) = it.next() {
		if (first.1).1 != '[' {
			return ScanResult {
				data: LexedData::Empty,
				cont: ""
			}
		}
	}
	let mut arr = Vec::new();	
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
				} else if value.1 == ']' {
					return ScanResult {
						data: LexedData::Array(arr),
						cont: &cur[(value.0+1) ..]						
					}
				} else {
					let result = parse_auto_detect(&cur[value.0 ..], false);
					if result.data != LexedData::Comment {
						arr.push(result.data);					
					}
					cur = result.cont;
					it = cur.char_indices().enumerate();
				}
			}
		}
	}
}

fn parse_auto_detect<'a>(data: &'a str, require_value:bool) -> ScanResult<'a>
{
	// first should be {
	let mut it = data.char_indices().enumerate();
	let mut is_comment = false;
	loop {
		match it.next() {
			None => return make_parse_error("Unexpected end at auto."),
			Some(ref x) => {
				let value = &x.1;
				if is_comment {
					if value.1 == '\n' {
						is_comment = false;
						if !require_value {
							return ScanResult {
								data: LexedData::Comment,
								cont: &data[value.0..]
							}
						}
					} else {
						continue;
					}
				}
				if value.1.is_whitespace() {
					continue;
				} else if value.1 == '#' {
					is_comment = true;
				} else if value.1 == '{' {
					return parse_object_data(&data[(value.0) ..]);
				} else if value.1 == '[' {
					return parse_array(&data[(value.0) ..]);				
				} else if value.1 == '@' {
					return parse_object_with_header(&data[(value.0) ..]);
				} else {
					return parse_keyword_or_string(&data[value.0 ..]);
				}
			}
		}
	}	
}

pub fn parse_object_data<'a>(data: &'a str) -> ScanResult<'a>
{
	let mut cur = data;
	let mut it = data.char_indices().enumerate();
	let mut field_name = String::new();
	let mut kv = HashMap::new();

	if let Some(first) = it.next() {
		if (first.1).1 != '{' {
			panic!("Object did not start with {{, it was {}!", (first.1).1);
		}
	}

	loop {
		match it.next() {
			None => {
				println!("Reached end of file before object is done.");
				return ScanResult {
					cont: "",
					data: LexedData::Empty
				};
			}
			Some(ref x) => {                 
				let value = &x.1;
				if value.1.is_whitespace() || value.1 == ',' || value.1 == ':' || value.1 == '=' {
					continue;
				} else if value.1 == '}' {
					return ScanResult {
						cont: &cur[(value.0 + 1) ..],
						data: LexedData::Object {
							id: String::new(),
							type_name: String::new(),
							kv: kv
						}
					};
				} else if field_name.is_empty() {
					let res = parse_auto_detect(&cur[value.0 ..], true);
					let d = res.data;
					match d {
						LexedData::Value(v) => {
							field_name = v;
							cur = res.cont;
							it = cur.char_indices().enumerate();							
						},
						LexedData::StringLiteral(v) => {
							field_name = v;
							cur = res.cont;
							it = cur.char_indices().enumerate();							
						}                        
						_ => {
							println!("Parse error. Could not parse field name at {}", &cur[value.0 ..]);
							println!("Full: {}", cur);							
							return ScanResult {
								cont: "",
								data: LexedData::Empty
							}
						}
					}
				} else {
					let res = parse_auto_detect(&cur[value.0 ..], true);
					cur = res.cont;
					it = cur.char_indices().enumerate();
					kv.insert(String::from(field_name), res.data);
					field_name = String::new();
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

	if let Some(first) = it.next() {
		if (first.1).1 != '@' {
			return ScanResult {
				data: LexedData::Empty,
				cont: ""
			}
		}
	}

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
					let res = parse_object_data(&cur[value.0 ..]);
					match res.data {
						LexedData::Object { kv, .. } => {
							return ScanResult {
								cont: res.cont,
								data: LexedData::Object {                                    
									id: String::from(&cur[id_begin .. id_end]),
									type_name: String::from(&cur[1..type_end]),
									kv: kv
								}
							};
						},
						_ => {
							println!("How can i fail {}?", &cur[value.0 ..]);
							return make_parse_error("Parsed object but not real!");
						}
					}
				}
			}
		}
	}
}

pub fn lex_file(data: &str) -> LexedKv
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
					let result = parse_object_with_header(&cur[value.0..]);
					match result.data {
						LexedData::Object { kv, id, type_name } => {
							objs.insert(id.clone(), LexedData::Object {
								kv: kv,
								id: id,
								type_name: type_name
							});
						}
						_ => {
							println!("parse error; expected object at {}", &cur[value.0 ..]);
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