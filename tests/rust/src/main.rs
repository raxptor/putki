#![feature(rc_downcast)]
extern crate putki;
extern crate gen_test;

use gen_test::inki;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any;

pub struct Hej
{
}

impl Hej {const TYPE_ID: i32 = 1; }

pub fn main() 
{ 
    /*
    println!("type is {}", Hej::TYPE_ID);
    let k:Option<Rc<mixki::Main>>;
    let p1:Option<Rc<mixki::PointerContainer>>;
    let p3:Option<Rc<mixki::PointerContainer>>;
    let tt:Option<Rc<mixki::TestTypes>>;    
    let dlg:Option<Rc<mixki::Dialog>>;    
    {
        let mut contents = String::new();   
        { 
            let mut f = File::open("data/main.txt").expect("file not found");    
            f.read_to_string(&mut contents).expect("something went wrong reading the file");
        }
        let db = lexer::lex_file(&contents);    
        let apa : parser::ResolveContext<mixki::ParseRc> = parser::ResolveContext {
            def: mixki::ParseRc { },
            unparsed: &db,
            resolved: RefCell::new(HashMap::new())
        };            
        k = parser::resolve(&apa, "main1");        
        p1 = parser::resolve(&apa, "pc1");
        p3 = parser::resolve(&apa, "pc3");
        tt = parser::resolve(&apa, "tt1");
        dlg = parser::resolve(&apa, "dlg");

    }
    match k
    {
        Some(m) => println!("I got main with value {}", m.value),
        None => println!("i got nothing!")
    }
    match p1
    {
        Some(p) => println!("I got p1 container, required = {}", p.required.value),
        None => println!("i got nothing!")
    } 
    match p3
    {
        Some(ref p) => {
            println!("I got p3 container, required = {} optioal {}", p.required.value, p.optional.as_ref().unwrap().value);
        } 
        None => println!("i got nothing!")
    }   
    match tt
    {
        Some(ref s) => {
            println!("I got test type, inner value = {} {} {} [{}] {} ", s.int, s.float, s.byte, s.string, s.bool);
        } 
        None => println!("i got nothing!")
    }    
    match dlg
    {
        Some(ref s) => {
            println!("I got dialog id {} {} {}", s.id, s.node1.id, s.node2.id);            
            use mixki::IDlgNodeTypes::*;
            match s.node1.get_child()
            {
                DlgMood (_) => { println!("mood!"); },
                DlgSay (k) => { println!("say! {}", k.text); },
                _ => {}
            }
        } 
        None => println!("i got nothing!")
    }                  
    */
}
