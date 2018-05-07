/// Here goes data types thata are shared between outki and pipeline

pub trait Layout {
    const TAG : &'static str;
}

pub struct BinLayout { }

impl Layout for BinLayout {
    const TAG : &'static str = "BinLayout";
}