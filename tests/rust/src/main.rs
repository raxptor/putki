#![feature(rc_downcast)]
extern crate putki;
extern crate gen_test;

use gen_test::inki;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::any;
use putki::PtrInkiResolver;

pub fn print_node(node: Option<Arc<inki::IDlgNode>>)
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
    let la = Arc::new(putki::LoadAll::from_txty_data(contents.as_ref()));	
    
    let resolve_context = Arc::new(putki::InkiPtrContext {
        source: Arc::new(putki::InkiResolver::new(la)),
        tracker: None
    });

    let k : putki::ResolveStatus<inki::Main> = putki::InkiResolver::resolve(&resolve_context, "main1");
    match k {
        putki::ResolveStatus::Resolved(m) => println!("I got main with value {}", m.value),
        _ => println!("i got nothing!")
    }

    let ta : putki::ResolveStatus<inki::TestArrays> = putki::InkiResolver::resolve(&resolve_context, "testarrays");
    match ta {
        putki::ResolveStatus::Resolved(m) => println!("I got arrays with length {:?} and {:?}", m.arr_f, m.arr_b),
        _ => println!("i got nothing!")
    }    

    let dlg : putki::ResolveStatus<inki::Dialog> = putki::InkiResolver::resolve(&resolve_context, "dlg");
    match dlg
    {
        putki::ResolveStatus::Resolved(s) => {
            println!("I got dialog id {}", s.id);
            print_node(s.node1.resolve());
            print_node(s.node2.resolve());
        } 
        _ => println!("i got nothing!")
    }        
}
