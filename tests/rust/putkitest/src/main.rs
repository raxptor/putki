extern crate putki;

use putki::outki;
use std::vec::Vec;
use std::mem::size_of;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::Cell;
use std::cell::RefCell;

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

pub fn main() 
{    
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

/*
    let mut np = Pa
    {
        ch: Some(c)
    };
*/
    //ch.p.ch = Some(&ch);

    /*
    let mut x = Vec::new();
    x.push(1);
    x.push(2);
    let sl:&[i32] = (&x);

    let k:Vec<i32> = sl.to_owned();
    */
}