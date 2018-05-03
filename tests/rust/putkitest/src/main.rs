extern crate putki;

use putki::outki;
use std::vec::Vec;
use std::mem::size_of;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::Cell;
use std::cell::RefCell;
use std::env;
use std::fs::File;
use std::io::prelude::*;

pub struct FirstType
{
    gurka: i32,
    kurka: i32
}

pub struct SecondType
{
    apa: i32,
    beta: i32
}

pub enum ChildType
{
    Nothing, 
    Type1 ( Box<FirstType> ),
    Type2 ( Box<SecondType> ),
    Type3 ( Box<SecondType> )
}

pub struct Parent
{
    sub: ChildType,
    always: i32
}

pub fn check_ft(ft: &FirstType)
{
    println!("first type {} {}", ft.gurka, ft.kurka);
}

pub fn check(ct: &ChildType)
{
    match ct {
        &ChildType::Nothing => println!("nothing"),
        &ChildType::Type1(ref fld)  => check_ft(fld),
        _ => {}
    }
}

struct KeyValue
{
    key: String,
    value: String
}

fn parse_list<F>(data: &str, parser:F, term:char) -> &str
    where F: Fn(&str) -> &str
{
    let mut cur = data;
    let mut it = data.char_indices().enumerate(); 
    let mut hx = 0;    
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

// Parse one @type id { block }
fn parse_object<'a>(data: &'a str, default_type: &str) -> &'a str
{
    let mut cur = data;
    let mut it = data.char_indices().enumerate(); 
    let type_begin = 0;
    let mut type_end = 0;
    loop {
        match it.next() {
            None => return "",
            Some(ref x) => {                 
                let value = &x.1;
                if (value.1.is_whitespace() && type_end == 0)
                {
                    type_end = value.0;
                }
                if (value.1 == '{')
                {
                    let content_begin = value.0;
                    let obj_type = String::from(&cur[0..type_end]);
                    let name = String::from(&cur[type_end .. content_begin]);                    
                    println!("type=[{}] name=[{}]", obj_type.trim(), name.trim());
                    match it.next() {
                        None => println!("Unexpected end of file"),
                        Some(ref nextval) => {
                            println!("parisng at {} [{}]", value.0, &cur[nextval.0 .. (nextval.0 + 4)]);
                            return parse_list(&cur[nextval.0 .. ], |pd:&str| -> &str {
                                println!("field[{}]", &pd[0..1]);
                                return &pd[1 ..];    
                            }, '}');
                        }
                    }
                }
            }
        }
    }
}

fn parse_file(data: &str)
{
    let mut cur = data;
    let mut it = data.char_indices().enumerate(); 
    let mut hx = 0;    
    loop {
        match it.next() {
            None => return,
            Some(ref x) => {                 
                let value = &x.1;
                if value.1 == '@' {
                    cur = parse_object(&cur[value.0..], "");
                    it = cur.char_indices().enumerate();
                    hx = hx + 1;
                    if (hx > 4) {
                        return;
                    }
                }
            } 
        }
    }

//    loop {
        /*
        match it {
            None => break,
            Some(x) => it = x.next()
        }
        */
    //}
/*    for (index, ch) in data.char_indices() {
        if (ch == '@') {
            println!("object at {}!", index);
        }
    }
    */
}

pub fn main() 
{   
    let mut contents = String::new();
    { 
        let mut f = File::open("data/main.txt").expect("file not found");    
        f.read_to_string(&mut contents).expect("something went wrong reading the file");
    }
    parse_file(&contents);
    //println!("file contains: [{}]", contents);
/*

    let k = ChildType::Type1(Box::new(FirstType {
        gurka: 3,
        kurka: 9
    }));
    let p = Parent {
        sub: k,
        always: 32
    };
    check(&p.sub);
    println!("size is {} ", size_of::<Parent>());
    println!("size is {} ", size_of::<FirstType>());
    println!("size of box {} ", size_of::<Box<FirstType>>());
    */
}