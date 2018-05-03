extern crate putki;

use std::collections::HashMap;
use std::vec::Vec;
use std::fs::File;
use std::io::prelude::*;
use std::sync::Arc;
use std::thread;

pub enum ParsedData<'a>
{
    Empty,
    Object { 
        kv : HashMap<&'a str, ParsedData<'a>>, 
        id: &'a str,
        type_name: &'a str 
    },
    Array (Vec<&'a str>),
    Value (&'a str),
    StringLiteral(&'a str)
}

struct ParsedResult<'a>
{
    cont: &'a str,
    data: ParsedData<'a>
}

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

fn make_parse_error(err: &str) -> ParsedResult
{
    println!("Parse error. {}", err);
    return ParsedResult {
        cont: "",
        data: ParsedData::Empty
    }
}

fn is_syntax_delimiter(c : char) -> bool
{
    return c.is_whitespace() || c == ',' || c == '}' || c == ']' || c == ':' || c == '=';
}

fn unstring(data: &str) -> &str
{
    return &data[1..(data.len()-1)];
}

fn parse_keyword_or_string<'a>(data: &'a str) -> ParsedResult<'a>
{
    let mut it = data.char_indices().enumerate(); 
    let mut inside_string = false;
    let mut string_start = 0;
    let mut escaped = false;
    loop {
        match it.next() {
            None => { 
                return ParsedResult {
                    cont: &data[1 ..],
                    data: ParsedData::Empty
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
                        return ParsedResult {
                            cont: &data[value.0 ..],
                            data: ParsedData::Value(&data[0 .. value.0])
                        };
                    } else {
                        return ParsedResult {
                            cont: &data[(value.0 + 1) ..],
                            data: ParsedData::StringLiteral(&data[(string_start + 1) .. value.0])
                        };
                    }
                }
            }
        }
    }    
}

fn parse_auto_detect<'a>(data: &'a str) -> ParsedResult<'a>
{
    // first should be {
    let mut it = data.char_indices().enumerate();
    loop {
        match it.next() {
            None => return make_parse_error("Unexpected end at auto."),
            Some(ref x) => {
                let value = &x.1;
                if (value.1.is_whitespace()) {
                    continue;
                } else if value.1 == '{' {
                    return parse_object_data(&data[(value.0 + 1) ..]);
                } else if value.1 == '@' {
                    return parse_object_with_header(&data[value.0 ..]);
                } else if value.1 == '[' {
                    // parse array
                    // return parse_object_data(&cur[value.0 ..]);
                } else {
                    return parse_keyword_or_string(&data[value.0 ..]);
                }
            }
        }
    }
    return make_parse_error("Unexpected end at auto.");
}

fn parse_object_data<'a>(data: &'a str) -> ParsedResult<'a>
{
    let mut cur = data;
    let mut it = data.char_indices().enumerate();
    let mut field_name = "";
    let mut kv = HashMap::new();
    //
    loop {
        match it.next() {
            None => return ParsedResult {
                cont: "",
                data: ParsedData::Empty
            },
            Some(ref x) => {                 
                let value = &x.1;
                if value.1.is_whitespace() || value.1 == ',' {
                    continue;
                } else if value.1 == '}' {
                    return ParsedResult {
                        cont: &cur[(value.0 + 1) ..],
                        data: ParsedData::Object {
                            id: "",
                            type_name: "",
                            kv: kv
                        }
                    };
                } else if field_name.is_empty() {
                    let res = parse_keyword_or_string(&cur[value.0 ..]);
                    cur = res.cont;
                    match (res.data) {
                        ParsedData::Value(ref v) => {
                            field_name = v;
                            cur = res.cont;
                            it = cur.char_indices().enumerate();
                        },
                        ParsedData::StringLiteral(ref v) => {
                            field_name = v;
                            cur = res.cont;
                            it = cur.char_indices().enumerate();
                        }                        
                        _ => {
                            println!("Parse error. Could not parse value");
                            return ParsedResult {
                                cont: "",
                                data: ParsedData::Empty
                            }
                        }
                    }
                } else if (value.1 == ':' || value.1 == '=') {
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
fn parse_object_with_header<'a>(data: &'a str) -> ParsedResult<'a>
{
    let mut cur = data;
    let mut it = data.char_indices().enumerate(); 
    let type_begin = 0;
    let mut type_end = 0;
    loop {
        match it.next() {
            None => return ParsedResult {
                cont: "",
                data: ParsedData::Empty
            },
            Some(ref x) => {                 
                let value = &x.1;
                if (value.1.is_whitespace() && type_end == 0)
                {
                    type_end = value.0;
                }
                if (value.1 == '{')
                {
                    let res = parse_object_data(&cur[(value.0 + 1) ..]);
                    match (res.data) {
                        ParsedData::Object { id, type_name, kv } => {
                            return ParsedResult {
                                cont: res.cont,
                                data: ParsedData::Object {                                    
                                    id: &cur[type_end .. value.0],
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

fn parse_file(data: &str) -> HashMap<&str, ParsedData>
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
                    match (result.data) {
                        ParsedData::Object { kv, id, type_name } => {
                            println!("parsed object with type=[{}] id=[{}]", id, type_name);
                            objs.insert(id, ParsedData::Object {
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
    return objs;
}

pub fn main() 
{   
    let mut contents = String::new();
    { 
        let mut f = File::open("data/main.txt").expect("file not found");    
        f.read_to_string(&mut contents).expect("something went wrong reading the file");
    }

    let parseString = Arc::new(contents);
    let mut thrs = Vec::new();
    for _ in 0..1 {
        let tc = Arc::clone(&parseString);
        thrs.push(thread::spawn(move || {
            let db = parse_file(&tc);    
            for (ref id, ref value) in &db {
                match *value {
                    &ParsedData::Object {ref id, ref type_name, ref kv} => {
                        println!("{} {} props={}", type_name, id, kv.len());
                    }
                    _ => { }
                }
            }
            return db.len();
        }));
    }

    for h in thrs {
        println!("Thread parsed {} objects", h.join().unwrap());
    }

}