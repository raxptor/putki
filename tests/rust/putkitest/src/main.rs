extern crate putki;
extern crate putki_gen;

use std::fs::File;
use std::io::prelude::*;
use std::vec::Vec;
use std::sync::Arc;
use std::thread;
use putki::mixki_parser;
use putki_gen::mixki;

pub fn main() 
{   
    let mut contents = String::new();   
    { 
        let mut f = File::open("data/main.txt").expect("file not found");    
        f.read_to_string(&mut contents).expect("something went wrong reading the file");
    }

    mixki::ParseYes();

    let parseString = Arc::new(contents);
    let mut thrs = Vec::new();
    for _ in 0..1 {
        let tc = Arc::clone(&parseString);
        thrs.push(thread::spawn(move || {
            let db = mixki_parser::parse_file(&tc);    
            for (ref id, ref value) in &db {
                match *value {
                    &mixki_parser::ParsedData::Object {ref id, ref type_name, ref kv} => {
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