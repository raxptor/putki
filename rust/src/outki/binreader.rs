use outki::{BinLoader, BinResolverContext, OutkiResult};

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

impl BinReader for u16 {
    fn read(stream:&mut BinDataStream) -> Self {
        let v = (stream.slice[0] as u16) | ((stream.slice[1] as u16) << 8);
        stream.slice = &stream.slice[2..];
        return v as u16;
    }
}

impl BinReader for usize {
    fn read(stream:&mut BinDataStream) -> Self {
        let v0 = u32::read(stream) as usize;
        let v1 = u32::read(stream) as usize;
        return v0 | (v1 << 32);
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
        return vec;
    }        
}

impl<T> BinLoader for Vec<T> where T : BinLoader {
    fn read(stream: &mut BinDataStream) -> Self {
        let len = usize::read(stream);
        let mut vec = Vec::with_capacity(len);
        for _i in 0..len {
            vec.push(<T as BinLoader>::read(stream));
        }
        return vec;        
    }
    fn resolve(&mut self, context: &mut BinResolverContext) -> OutkiResult<()> {
        for x in self.iter_mut() {
            x.resolve(context)?;
        }
        Ok(())
    }    
}
