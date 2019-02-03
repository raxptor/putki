use std::io::Read;
use outki::*;

pub struct PackageManifest
{

}

impl PackageManifest 
{
    pub fn parse(reader:&mut Read) -> OutkiResult<PackageManifest> {
        let mut buffer = [0; 8];
        reader.read_exact(&mut buffer)?;

        let mut tmp_ds = BinDataStream::new(&mut buffer);
        let mfs:usize = usize::read(&mut tmp_ds);
        println!("Package manifest is {} bytes", mfs);
        Ok(PackageManifest {

        })
    }
}