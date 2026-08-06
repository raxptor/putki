use std::io::Read;
use crate::outki::*;

pub struct Slot
{
    pub flags: u32,
    pub path: Option<String>,
    pub begin: usize,
    pub end: usize,
    /// Compiler-assigned type id, or 0 if unknown. See `doc/package-format.md`.
    pub type_id: usize,
}

pub struct TypeEntry
{
    pub id: usize,
    pub name: String,
}

#[derive(Default)]
pub struct PackageManifest
{
    pub slots: Vec<Slot>,
    pub types: Vec<TypeEntry>,
    pub manifest_size: usize
}

/// Bounds-checked reader over the header bytes.
///
/// Package data is not trusted input: a truncated or corrupt file has to come
/// back as an error, never a panic or an out-of-range read. `BinDataStream`
/// indexes its slice directly and would panic, so the manifest is parsed
/// through this instead.
struct Cursor<'a>
{
    data: &'a [u8],
    pos: usize
}

impl<'a> Cursor<'a>
{
    fn new(data: &'a [u8]) -> Self {
        Cursor { data, pos: 0 }
    }

    fn take(&mut self, count: usize) -> OutkiResult<&'a [u8]> {
        match self.pos.checked_add(count) {
            Some(end) if end <= self.data.len() => {
                let slice = &self.data[self.pos .. end];
                self.pos = end;
                Ok(slice)
            }
            _ => Err(OutkiError::CorruptPackage("header ended mid-field"))
        }
    }

    fn u32(&mut self) -> OutkiResult<u32> {
        let b = self.take(4)?;
        Ok(u32::from(b[0]) | (u32::from(b[1]) << 8) | (u32::from(b[2]) << 16) | (u32::from(b[3]) << 24))
    }

    fn usize(&mut self) -> OutkiResult<usize> {
        let low = self.u32()? as usize;
        // The high word is always written as zero; a value in it means either
        // corruption or a package from something that is not this format.
        if self.u32()? != 0 {
            return Err(OutkiError::CorruptPackage("oversized length field"));
        }
        Ok(low)
    }

    fn string(&mut self) -> OutkiResult<String> {
        let len = self.usize()?;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec())
            .map_err(|_| OutkiError::CorruptPackage("string is not valid utf-8"))
    }
}

impl PackageManifest
{
    pub fn new() -> PackageManifest {
        Default::default()
    }

    pub fn add_obj<T>(&mut self, output: &mut Vec<u8>, path:Option<&str>, data:&[u8]) where T : shared::TypeDescriptor
    {
        let type_id = <T as shared::TypeDescriptor>::TYPE_ID;
        if !self.types.iter().any(|t| t.id == type_id) {
            self.types.push(TypeEntry {
                id: type_id,
                name: shared::tag_of::<T>().to_string()
            });
        }
        let begin = output.len();
        output.extend_from_slice(data);
        let end = output.len();
        self.slots.push(Slot {
            begin,
            end,
            flags: 0,
            path: path.map(|x| { x.to_string() }),
            type_id
        });
    }

    /// Name of a type id, for diagnostics.
    pub fn type_name(&self, type_id: usize) -> Option<&str> {
        self.types.iter().find(|t| t.id == type_id).map(|t| t.name.as_str())
    }

    pub fn parse(reader:&mut dyn Read) -> OutkiResult<PackageManifest> {
        let mut head = [0u8; PACKAGE_HEADER_FIXED];
        reader.read_exact(&mut head)?;
        let mut cursor = Cursor::new(&head);

        let magic = cursor.u32()?;
        if magic != PACKAGE_MAGIC {
            return Err(OutkiError::BadMagic(magic));
        }
        let version = cursor.u32()?;
        if version != PACKAGE_VERSION {
            return Err(OutkiError::UnsupportedVersion(version));
        }
        let header_size = cursor.usize()?;
        if header_size < PACKAGE_HEADER_FIXED {
            return Err(OutkiError::CorruptPackage("header size smaller than the header"));
        }

        let mut buffer:Vec<u8> = vec![0; header_size - PACKAGE_HEADER_FIXED];
        reader.read_exact(&mut buffer)?;
        let mut content = Cursor::new(&buffer);

        // Counts are not trusted, so nothing is pre-allocated from them; a
        // bogus count runs the cursor out of data and errors instead of
        // reserving whatever the file asked for.
        let num_types = content.usize()?;
        let mut types:Vec<TypeEntry> = Vec::new();
        for _i in 0..num_types {
            let id = content.usize()?;
            let name = content.string()?;
            types.push(TypeEntry { id, name });
        }

        let num_slots = content.usize()?;
        let mut slots = Vec::new();
        for _i in 0..num_slots {
            let flags = content.u32()?;
            if (flags & !(SLOTFLAG_HAS_PATH | SLOTFLAG_INTERNAL)) != 0 {
                return Err(OutkiError::CorruptPackage("unknown slot flags"));
            }
            let path: Option<String> = if (flags & SLOTFLAG_HAS_PATH) != 0 {
                Some(content.string()?)
            } else {
                None
            };
            let type_id = content.usize()?;
            let begin = content.usize()?;
            let end = content.usize()?;
            if end < begin {
                return Err(OutkiError::CorruptPackage("slot ends before it begins"));
            }
            if begin < header_size {
                return Err(OutkiError::CorruptPackage("slot payload overlaps the header"));
            }
            if type_id != 0 && !types.iter().any(|t| t.id == type_id) {
                return Err(OutkiError::CorruptPackage("slot names a type not in the table"));
            }
            slots.push(Slot {
                begin,
                end,
                path,
                type_id,
                flags
            });
        }

        Ok(PackageManifest {
            slots,
            types,
            manifest_size: header_size
        })
    }
}
