use std::io::Read;
use std::ops::DerefMut;
use outki::*;

pub struct Slot
{
    pub flags: u32,
    pub path: Option<String>,
    pub begin: usize,
    pub end: usize,
    pub type_id:usize,    
}

pub struct PackageManifest
{
    pub slots: Vec<Slot>,
    pub types: Vec<String>,
    pub manifest_size: usize
}

impl PackageManifest 
{
    pub fn new() -> PackageManifest {
        PackageManifest {
            slots: Vec::new(),
            types: Vec::new(), 
            manifest_size: 0
        }
    }

    pub fn add_obj<T>(&mut self, output: &mut Vec<u8>, path:Option<&str>, data:&[u8]) where T : shared::TypeDescriptor
    {
        let ti = self.types.len();
        self.types.push(shared::tag_of::<T>().to_string());
        let begin = output.len();
        output.extend_from_slice(data);
        let end = output.len();
        self.slots.push(Slot {
            begin: begin,
            end: end,
            flags: 0,
            path: path.map(|x| { x.to_string() }),
            type_id: ti
        });
    }


    pub fn parse(reader:&mut Read) -> OutkiResult<PackageManifest> {
        let mut buffer = [0; 8];
        reader.read_exact(&mut buffer)?;        
        let mut tmp_ds = BinDataStream::new(&mut buffer);
        let mfs:usize = usize::read(&mut tmp_ds);
        println!("Package manifest is {} bytes", mfs);

        let mut buffer:Vec<u8> = vec![0; mfs-8];
        reader.read_exact(buffer.deref_mut())?;
        let mut content  = BinDataStream::new(&buffer);             
        
        // Read types.
        let num_types  = usize::read(&mut content);
        let mut types:Vec<String> = Vec::new();
        for _i in 0..num_types {
            types.push(String::read(&mut content));
        }

        let num_slots  = usize::read(&mut content);
        let mut slots = Vec::new();

        for _i in 0..num_slots {
            let flags = u32::read(&mut content);
            let mut path: Option<String> = None;
            if (flags & SLOTFLAG_HAS_PATH) != 0 {
                path = Some(String::read(&mut content));
            }
            let type_id = usize::read(&mut content);
            let begin = usize::read(&mut content);
            let end = usize::read(&mut content);
            slots.push(Slot {
                begin: begin,
                end: end,
                path: path,
                type_id: type_id,
                flags: flags
            });
        }
        
        Ok(PackageManifest {
            slots: slots,
            types: types,
            manifest_size: mfs
        })
    }
}