pub struct BinDataStream<'a> {
    slice: &'a [u8]
}

impl<'a> BinDataStream<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        BinDataStream {
            slice: slice
        }
    }
}

pub trait BinReader {
    fn read(stream:&mut BinDataStream) -> Self;    
}

impl BinReader for u32 {
    fn read(stream:&mut BinDataStream) -> Self {
        let v = (stream.slice[0] as u32) |
        ((stream.slice[1] as u32) << 8) | 
        ((stream.slice[2] as u32) << 16) | 
        ((stream.slice[3] as u32) << 24);
        stream.slice = &stream.slice[4..];
        return v as u32;
    }
}

impl BinReader for u8 {
    fn read(stream:&mut BinDataStream) -> Self {
        let v = stream.slice[0];
        stream.slice = &stream.slice[1..];
        return v;
    }
}

impl<'a> BinReader for i32 {
    fn read(ctx: &mut BinDataStream) -> i32 {
        u32::read(ctx) as i32
    }
}
