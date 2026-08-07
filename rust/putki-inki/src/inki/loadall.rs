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
		self.objs.get(path).and_then(|x| {
			Some((x.type_.as_str(), &x.data))
		})
	}
}

fn visit_dirs(dir: &Path, cb: &mut dyn FnMut(&DirEntry)) -> io::Result<()> {
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
		lexer::LexedData::Object { kv, .. } => {
			if let Ok(ref_) = file.strip_prefix(base) {
				let bp = String::from(ref_.to_string_lossy());				
				let mut obj_ref = bp.trim_end_matches(".json").replace('\\', "/");
				if let Some(refval) = kv.get("ref") {
					if let lexer::LexedData::StringLiteral(path_piece) = refval {
						obj_ref.push_str(path_piece);
					}
				}
				if let Some(val) = kv.get("type") {
					if let lexer::LexedData::StringLiteral(type_name) = val {
						if let Some(val) = kv.get("data") {
							if let lexer::LexedData::Object{kv, ..} = val {
								let entry = ObjEntry {
									type_: (*type_name).clone(),
									data: (*kv).clone()
								};
								idx.objs.insert(obj_ref, entry);
							}
						}
					}
				}
				if let Some(val) = kv.get("aux") {
					if let lexer::LexedData::Array(arr) = val {
						for auxobj in arr {
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
	Ok(())
}

fn collect_txty_inline_objs(idx: &mut LoadAll, obj: &lexer::LexedData)
{
	if let lexer::LexedData::Object{ type_name, kv, id } = obj {
		if !id.is_empty() {
			idx.objs.insert(id.clone(), ObjEntry {
				type_: type_name.clone(),
				data: kv.clone()
			});
		}
		for sub in kv.values() {
			collect_txty_inline_objs(idx, sub);
		}		
	}
	if let lexer::LexedData::Array(arr) = obj {
		for obj in arr {
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
	Ok(())
}

impl LoadAll {
	/// Every object that was loaded, as `(path, type name, fields)`.
	///
	/// Sorted by path so that anything derived from a whole data set -- string
	/// extraction in particular -- comes out in the same order every run rather
	/// than in hash order.
	pub fn objects(&self) -> Vec<(&str, &str, &lexer::LexedKv)> {
		let mut all: Vec<(&str, &str, &lexer::LexedKv)> = self.objs
			.iter()
			.map(|(path, e)| (path.as_str(), e.type_.as_str(), &e.data))
			.collect();
		all.sort_by(|a, b| a.0.cmp(b.0));
		all
	}

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
		idx
	}

	pub fn from_txty_data(data: &str) -> LoadAll {
		let mut idx = LoadAll {
			objs: HashMap::new()
		};
		for (_, obj) in lexer::lex_file(data) {		
			collect_txty_inline_objs(&mut idx, &obj);
		}
		idx
	}	
}
