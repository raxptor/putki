use outki::{BinLoader, BinResolverContext, OutkiResult};

pub struct BinDataStream<'a> {
    slice: &'a [u8]
}

impl<'a> BinDataStream<'a> {
    pub fn new(slice: &'a [u8]) -> Self {
        BinDataStream {
            slice
        }
    }
}

pub trait BinReader {
    fn read(stream:&mut BinDataStream) -> Self;    
}

impl BinReader for u32 {
    fn read(stream:&mut BinDataStream) -> Self {
        let v = u32::from(stream.slice[0]) |
        (u32::from(stream.slice[1]) << 8) | 
        (u32::from(stream.slice[2]) << 16) | 
        (u32::from(stream.slice[3]) << 24);
        stream.slice = &stream.slice[4..];
        v
    }
}

impl BinReader for u16 {
    fn read(stream:&mut BinDataStream) -> Self {
        let v = u16::from(stream.slice[0]) | (u16::from(stream.slice[1]) << 8);
        stream.slice = &stream.slice[2..];
        v
    }
}

impl BinReader for usize {
    fn read(stream:&mut BinDataStream) -> Self {
        let v0 = u32::read(stream) as usize;
        let v1 = u32::read(stream) as usize;
        v0 | (v1 << 32)
    }
}

impl BinReader for f32 {
    fn read(stream:&mut BinDataStream) -> Self {
        f32::from_bits(u32::read(stream))
    }
}

impl BinReader for bool {
    fn read(stream:&mut BinDataStream) -> Self {
        u8::read(stream) == 1
    }
}

impl BinReader for u8 {
    fn read(stream:&mut BinDataStream) -> Self {
        let v = stream.slice[0];
        stream.slice = &stream.slice[1..];
        v
    }
}

impl<'a> BinReader for i32 {
    fn read(ctx: &mut BinDataStream) -> i32 {
        u32::read(ctx) as i32
    }
}

impl<'a> BinReader for String {
    fn read(stream: &mut BinDataStream) -> String {
        let len = usize::read(stream);
        let res = String::from_utf8((&stream.slice[0..len]).to_vec());
        stream.slice = &stream.slice[len..];
        res.unwrap()
    }
}

impl<T> BinReader for Vec<T> where T : BinReader
{    
    fn read(stream: &mut BinDataStream) -> Vec<T> {
        let len = usize::read(stream);
        let mut vec = Vec::with_capacity(len);
        for _i in 0..len {
            vec.push(<T as BinReader>::read(stream));
        }
        vec
    }        
}

impl<T> BinLoader for Vec<T> where T : BinLoader {
    fn read(stream: &mut BinDataStream) -> Self {
        let len = usize::read(stream);
        let mut vec = Vec::with_capacity(len);
        for _i in 0..len {
            vec.push(<T as BinLoader>::read(stream));
        }
        vec
    }
    fn resolve(&mut self, context: &mut BinResolverContext) -> OutkiResult<()> {
        for x in self.iter_mut() {
            x.resolve(context)?;
        }
        Ok(())
    }    
}
