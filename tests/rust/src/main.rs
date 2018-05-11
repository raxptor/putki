#![feature(rc_downcast)]
extern crate putki;
extern crate gen_test;

use putki::inki as pinki;
use gen_test::inki;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any;

pub fn print_node(node: Option<Rc<inki::IDlgNode>>)
{
    match node {
        Some(node) => {
            match &(*node) {
                &inki::IDlgNode::DlgMood(ref mood) => println!("mood text={}", mood.text),
                &inki::IDlgNode::DlgSay(ref say) => println!("say text={} who={}", say.text, say.who),
                &inki::IDlgNode::IDlgNode(_) => println!("IDlgNodoe pure"),
                _ => println!("aah")
            }
            println!("got node");
        },
        _ => println!("no node")             
    }    
}

pub fn main() 
{     
    let mut contents = String::new();   
    { 
        let mut f = File::open("data/main.txt").expect("file not found");    
        f.read_to_string(&mut contents).expect("something went wrong reading the file");
    }
    let la = Box::new(pinki::LoadAll::from_txty_data(contents.as_ref()));	
    
    let resolveContext = pinki::InkiPtrContext {
        source: Rc::new(pinki::InkiResolver::new(la)),
        tracker: None
    };

    let k : pinki::ResolveStatus<inki::Main> = pinki::InkiResolver::resolve(&resolveContext, "main1");
    match k {
        pinki::ResolveStatus::Resolved(m) => println!("I got main with value {}", m.value),
        _ => println!("i got nothing!")
    }

    let ta : pinki::ResolveStatus<inki::TestArrays> = pinki::InkiResolver::resolve(&resolveContext, "testarrays");
    match ta {
        pinki::ResolveStatus::Resolved(m) => println!("I got arrays with length {:?} and {:?}", m.arr_f, m.arr_b),
        _ => println!("i got nothing!")
    }    

    let dlg : pinki::ResolveStatus<inki::Dialog> = pinki::InkiResolver::resolve(&resolveContext, "dlg");
    match dlg
    {
        pinki::ResolveStatus::Resolved(s) => {
            println!("I got dialog id {}", s.id);
            print_node(s.node1.resolve());
            print_node(s.node2.resolve());
            /*
            match s.node1.get_child()
            {
                DlgMood (_) => { println!("mood!"); },
                DlgSay (k) => { println!("say! {}", k.text); },
                _ => {}
            }*/
        } 
        _ => println!("i got nothing!")
    }        
}
    /*
    p1 = parser::resolve(&apa, "pc1");
    p3 = parser::resolve(&apa, "pc3");
    tt = parser::resolve(&apa, "tt1");
    dlg = parser::resolve(&apa, "dlg");

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
              
    */

