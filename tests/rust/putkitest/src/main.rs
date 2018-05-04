extern crate putki;
extern crate gen_test;

pub fn main()
{
    let k = gen_test::mixki::Main { value : 33 };
    putki::putki_init();
}

/*
extern crate putki;
extern crate putki_gen;

use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;
use std::thread;
use putki::mixki_parser;
use putki_gen::mixki;
use std::collections::HashMap;

pub fn skit(fin : &mut mixki_parser::ResolvedDB<mixki::AnyRc>)
{
8
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
            let db = mixki_parser::lex_file(&tc);    
            for (ref id, ref value) in &db {
                match *value {
                    &mixki_parser::LexedData::Object {ref id, ref type_name, ref kv} => {
                        println!("{} {} props={}", type_name, id, kv.len());                        
                    },
                    _ => { }
                }
            }

            let mut apa = mixki_parser::ResolveContext { 
                unparsed: &db,
                resolved: RefCell::new(HashMap::new())
            };            
            let k = mixki_parser::resolve(&apa, "main1");
            match (k)
            {
                Some(u) => {
                    println!("aaa");
                    match (u) {
                        mixki::AnyRc::Main(m) => {
                            println!("yes!! {}", (*m).val_int);
                        }
                        _ => {

                        }
                    }

                },
                None => {

                }
            }
	        return apa.resolved.borrow().len();
        }));
    }

    for h in thrs {
        println!("Thread parsed {} objects", h.join().unwrap());
    }

    /*
	impl<'a> mixki_parser::Parse for AnyRc {
		fn parse(ctx:&mut mixki_parser::ResolveContext, unparsed: &mixki_parser::LexedDB, resolved:&mut mixki_parser::ResolvedDB<Self>, obj:&mixki_parser::LexedData) -> Option<rc::Rc<Self>>
		{
			match (obj) {
				&mixki_parser::LexedData::Object{ref id, ref type_name, ref kv} => {
					match (*type_name) {
						"Main" => { println!("trying to parse {}", type_name); }
						_ => { println!("trying to parse {}", type_name); }
					}
				}
				_ => { }
			}			
			return None;
		}
	}	
    */

}*/
