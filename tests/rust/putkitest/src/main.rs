extern crate putki;
extern crate gen_test;

use putki::mixki::lexer;
use putki::mixki::parser;
use gen_test::mixki;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub fn main() 
{   
    let k:Option<Rc<mixki::Main>>;
    {
        let mut contents = String::new();   
        { 
            let mut f = File::open("data/main.txt").expect("file not found");    
            f.read_to_string(&mut contents).expect("something went wrong reading the file");
        }
        let db = lexer::lex_file(&contents);    
        let mut apa : parser::ResolveContext<mixki::ParseRc> = parser::ResolveContext { 
            unparsed: &db,
            resolved: RefCell::new(HashMap::new())
        };            
        k = parser::resolve(&apa, "main1");   
    }
    match k
    {
        Some(m) => println!("I got main with value {}", m.value),
        None => println!("i got nothing!")
    }
}
