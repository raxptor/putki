use super::lexer;
use super::source;

use std::collections::HashMap;
use std::io;
use std::fs::{self, DirEntry, File};
use std::path::Path;
use std::io::BufReader;
use std::io::prelude::*;

pub struct ObjEntry {
	type_: String,
	data: lexer::LexedKv	
}

pub struct LoadAll {
	objs: HashMap<String, ObjEntry>
}

impl source::ObjectLoader for LoadAll
{
	fn load(&self, path: &str) -> Option<(&str, &lexer::LexedKv)>
	{
		return self.objs.get(path).and_then(|x| {
			return Some((x.type_.as_str(), &x.data));
		});
	}
}

fn visit_dirs(dir: &Path, cb: &mut FnMut(&DirEntry)) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb)?;
            } else {
                cb(&entry);
            }
        }
    }
    Ok(())
}

fn process_jsony_obj(base:&Path, file:&Path, idx: &mut LoadAll, ld:&lexer::LexedData)
{
	match ld {
		&lexer::LexedData::Object { ref kv, .. } => {
			if let Ok(ref_) = file.strip_prefix(base) {
				let mut bp = String::from(ref_.to_string_lossy());				
				let mut obj_ref = bp.trim_right_matches(".json").replace('\\', "/");
				if let Some(refval) = kv.get("ref") {
					if let &lexer::LexedData::StringLiteral(ref path_piece) = refval {
						obj_ref.push_str(path_piece);
					}
				}
				if let Some(val) = kv.get("type") {
					if let &lexer::LexedData::StringLiteral(ref type_name) = val {
						if let Some(val) = kv.get("data") {
							if let &lexer::LexedData::Object{ref kv, ..} = val {
								let entry = ObjEntry {
									type_: (*type_name).clone(),
									data: (*kv).clone()
								};
								let s = String::from(obj_ref);
								idx.objs.insert(s, entry);
							}
						}
					}
				}
				if let Some(val) = kv.get("aux") {
					if let &lexer::LexedData::Array(ref arr) = val {
						for ref auxobj in arr {
							process_jsony_obj(base, file, idx, auxobj);
						}
					}
				}				
			}		
		}
		_ => { println!("There was nothing..."); }
	}	
}

fn index_jsony_data(base:&Path, path:&Path, idx: &mut LoadAll) -> io::Result<()>
{
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let mut contents = String::new();
	reader.read_to_string(&mut contents)?;	
	let res = lexer::parse_object_data(&contents);
	process_jsony_obj(base, path, idx, &res.data);
	return Ok(());
}

fn collect_txty_inline_objs(idx: &mut LoadAll, obj: &lexer::LexedData)
{
	if let &lexer::LexedData::Object{ ref type_name, ref kv, ref id } = obj {
		if !id.is_empty() {
			idx.objs.insert(id.clone(), ObjEntry {
				type_: type_name.clone(),
				data: kv.clone()
			});
		}
		for (_, sub) in kv {
			collect_txty_inline_objs(idx, sub);
		}		
	}
	if let &lexer::LexedData::Array(ref arr) = obj {
		for ref obj in arr {
			collect_txty_inline_objs(idx, obj);
		}
	}
}

fn index_txty_data(_base:&Path, path:&Path, idx: &mut LoadAll) -> io::Result<()>
{
	let file = File::open(path)?;
	let mut reader = BufReader::new(file);
	let mut contents = String::new();	
	reader.read_to_string(&mut contents)?;
	for (_, obj) in lexer::lex_file(&contents) {		
		collect_txty_inline_objs(idx, &obj);
	}	
	return Ok(());
}

impl LoadAll {
	pub fn new(dir: &Path) -> LoadAll {
		let mut idx = LoadAll {
			objs: HashMap::new()
		};		
		{ // scope for borrow 
			let mut process = |entry:&DirEntry| {
				if entry.path().to_string_lossy().ends_with(".json") {
					index_jsony_data(&dir, &entry.path(), &mut idx).ok();
				} else if entry.path().to_string_lossy().ends_with(".txt") {
					index_txty_data(&dir, &entry.path(), &mut idx).ok();
				}
			};
			visit_dirs(dir, &mut process).ok();
		}
		return idx;
	}

	pub fn from_txty_data(data: &str) -> LoadAll {
		let mut idx = LoadAll {
			objs: HashMap::new()
		};
		for (_, obj) in lexer::lex_file(data) {		
			collect_txty_inline_objs(&mut idx, &obj);
		}
		return idx;
	}	
}
