#![allow(clippy::implicit_hasher)]
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
	fn default() -> LexedData { LexedData::Empty }
}

pub struct ScanResult<'a> 
{
	pub cont: &'a str,
	pub data: LexedData
}

/// Encode a string as a quoted literal. See `doc/text-format.md`.
///
/// Emits only `\\`, `\"`, `\n` and `\t`; everything else goes out as raw UTF-8.
/// Putki draws no distinction between carriage return and newline, so CRLF and
/// a lone CR both normalise to `\n`. `\uXXXX` is accepted on read but is never
/// produced, so it drains out of the data as files are re-saved.
pub fn escape_string(input:&str) -> String
{
	let mut s = String::with_capacity(input.len() + 32);
	s.push('\"');
	let mut it = input.chars().peekable();
	while let Some(c) = it.next() {
		match c {
			'\\' => s.push_str("\\\\"),
			'\"' => s.push_str("\\\""),
			'\n' => s.push_str("\\n"),
			'\t' => s.push_str("\\t"),
			'\r' => {
				if it.peek() == Some(&'\n') {
					it.next();
				}
				s.push_str("\\n");
			}
			_ => s.push(c),
		}
	}
	s.push('\"');
	s
}

/// Decode the body of a quoted literal, i.e. what sits between the quotes.
///
/// Returns None on an invalid escape rather than guessing, which is what the
/// older readers did and what made bad data invisible.
fn decode_string_body(body: &str) -> Option<String>
{
	// Accumulated as bytes because legacy `\uXXXX` encoded the individual bytes
	// of the original UTF-8, not a code point, so a run of them has to be
	// reassembled before it is valid text.
	let mut out: Vec<u8> = Vec::with_capacity(body.len());
	let mut it = body.chars().peekable();
	while let Some(c) = it.next() {
		if c == '\r' {
			if it.peek() == Some(&'\n') {
				it.next();
			}
			out.push(b'\n');
			continue;
		}
		if c != '\\' {
			let mut buf = [0u8; 4];
			out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
			continue;
		}
		match it.next() {
			Some('n') => out.push(b'\n'),
			Some('t') => out.push(b'\t'),
			Some('r') => out.push(b'\n'),
			Some('\\') => out.push(b'\\'),
			Some('\"') => out.push(b'\"'),
			Some('u') => {
				let mut code: u32 = 0;
				for _ in 0..4 {
					code = (code << 4) | it.next().and_then(|h| h.to_digit(16))?;
				}
				if code > 0xff {
					return None;
				}
				out.push(code as u8);
			}
			_ => return None,
		}
	}
	String::from_utf8(out).ok()
}

pub fn kv_to_string(kv: &HashMap<String, LexedData>) -> String
{
	// Emit keys in sorted order. `HashMap` iteration is seeded per process, and
	// this output is both written back out as text and hashed to derive the
	// identity of anonymous inline objects, so an arbitrary order would make
	// both differ between otherwise identical runs.
	let mut keys: Vec<&String> = kv.keys().collect();
	keys.sort();
	let mut tmp = String::new();
	tmp.push('{');
	for key in keys {
		tmp.push_str(key);
		tmp.push(':');
		tmp.push_str(kv[key].to_string().as_str());
		tmp.push(',');
	}
	tmp.push('}');
	tmp
}

impl ToString for LexedData
{
	fn to_string(&self) -> String {
		let mut tmp = String::new();
		match self {
			LexedData::Object { kv, id, type_name } => {
				if !type_name.is_empty() {
					tmp.push('@');
					tmp.push_str(type_name);
					tmp.push(' ');
					if !id.is_empty() {
						tmp.push_str(id);
						tmp.push(' ');
					}					
				}
				tmp.push_str(kv_to_string(kv).as_str());
			},
			LexedData::Value(val) => tmp.push_str(val),
			LexedData::StringLiteral(val) => {
				tmp.push_str(&escape_string(val));
			}
			LexedData::Array(vec) => {
				tmp.push('[');
				for val in vec {
					tmp.push_str(val.to_string().as_str());
					tmp.push(',');
				}
				tmp.push(']');
			}
			LexedData::Comment => { },
			LexedData::Empty => { }
		}
		tmp
	}
}

fn parse_val<T : FromStr + Default>(val: &LexedData) -> T
{
	if let LexedData::Value(x) = val {
		if let Ok(val) = T::from_str(x) {
			return val;
		}
	}
	println!("expected other type");
	Default::default()
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
			LexedData::Value(x) => {
				match x.as_ref() {
					"True" => Some(true),
					"true" => Some(true),
					"1" => Some(true),
					_ => Some(false)
				}
			}
			_ => None
		}
	}).unwrap_or(default)
}

pub fn get_string(data: Option<&LexedData>, default: &str) -> String
{
	data.map(|val| {
		match val {
			LexedData::Value(x) => x.clone(),
			LexedData::StringLiteral(x) => x.clone(),
			_ => String::from(default)
		}
	}).unwrap_or_else(|| String::from(default))
}

/// Reports an enum value in the data that names no variant.
///
/// Generated parsers call this instead of silently falling back to the first
/// variant, which turned a typo into wrong data that only showed up much later.
/// Parsing has no error channel (`ParseFromKV::parse` returns `Self`), and this
/// only ever runs over hand-authored data at build time, so it fails loudly.
pub fn unknown_enum_value(enum_name: &str, field: &str, value: &str, allowed: &[&str]) -> !
{
	panic!(
		"unknown value '{}' for enum {} in field '{}'; expected one of: {}",
		value,
		enum_name,
		field,
		allowed.join(", ")
	)
}

pub fn get_array(data: Option<&LexedData>) -> Option<slice::Iter<'_, LexedData>>
{
	data.and_then(|v| { 
		match v {
			LexedData::Array(arr) => Some(arr.iter()),
			_ => None
		}
	})
}

pub fn get_object(data: Option<&LexedData>) -> Option<(&LexedKv, &str)>
{
	data.and_then(|val| {
		match val {
			LexedData::Object{kv, type_name, ..} => Some((kv, type_name.as_ref())),
			_ => None
		}
	})
}

pub fn get_kv(data: Option<&LexedData>) -> Option<&LexedKv>
{
	data.and_then(|val| {
		match val {
			LexedData::Object{kv, ..} => Some(kv),
			_ => None
		}
	})
}

fn make_parse_error(err: &str) -> ScanResult<'_>
{
	println!("Parse error. {}", err);
	ScanResult {
		cont: "",
		data: LexedData::Empty
	}
}

fn is_syntax_delimiter(c : char) -> bool
{	
	c == '[' || c == '{' || c == '=' || c == ':' || c == ']' || c == '}' || c == ',' || c.is_whitespace()
}

fn parse_keyword_or_string(data: &str) -> ScanResult<'_>
{
	let mut it = data.char_indices();
	let mut inside_string = false;
	let mut body_start = 0;
	let mut escaped = false;
	loop {
		match it.next() {
			None => {
				if inside_string {
					return make_parse_error("Unterminated string literal.");
				}
				return ScanResult {
					cont: "",
					data: LexedData::Empty
				}
			},
			Some((pos, c)) => {
				if inside_string {
					// Only ever skips the one character after a backslash, so a
					// `\\` pair cannot swallow the character that follows it.
					if escaped {
						escaped = false;
						continue;
					}
					if c == '\\' {
						escaped = true;
						continue;
					}
					if c == '\"' {
						return match decode_string_body(&data[body_start .. pos]) {
							Some(value) => ScanResult {
								cont: &data[(pos + 1) ..],
								data: LexedData::StringLiteral(value)
							},
							None => make_parse_error("Invalid escape sequence in string literal.")
						};
					}
					continue;
				}
				if c == '\"' {
					inside_string = true;
					body_start = pos + 1;
					continue;
				}
				if is_syntax_delimiter(c) {
					if pos > 0 {
						return ScanResult {
							cont: &data[pos ..],
							data: LexedData::Value(String::from(&data[0 .. pos]))
						};
					}
					return ScanResult {
						cont: "",
						data: LexedData::Empty
					};
				}
			}
		}
	}
}

pub fn parse_array(data: &str) -> ScanResult<'_>
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

fn parse_auto_detect(data: &str, require_value:bool) -> ScanResult<'_>
{
	// first should be {
	let mut it = data.char_indices().enumerate().peekable();
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
				} else if value.1 == '#' || (value.1 == '/' && it.peek().map(|x| {(x.1).1}).unwrap_or_else(|| {' '}) == '/') {
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

pub fn parse_object_data(data: &str) -> ScanResult<'_>
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
							 kv
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
					kv.insert(field_name, res.data);
					field_name = String::new();
				}
			}
		}
	}
}

// Parse one @type id { block }
fn parse_object_with_header(data: &str) -> ScanResult<'_>
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
					if id_end == 0 && id_begin != 0 {
						id_end = value.0 
					}
					let res = parse_object_data(&cur[value.0 ..]);
					match res.data {
						LexedData::Object { kv, .. } => {
							return ScanResult {
								cont: res.cont,
								data: LexedData::Object {                                    
									id: String::from(&cur[id_begin .. id_end]),
									type_name: String::from(&cur[1..type_end]),
									kv
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
								kv,
								id,
								type_name
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