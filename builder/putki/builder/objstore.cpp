#include "objstore.h"

#include <putki/sys/files.h>
#include <putki/builder/signature.h>
#include <putki/builder/log.h>
#include <putki/builder/parse.h>
#include <putki/builder/tok.h>
#include <putki/builder/write.h>
#include <putki/builder/typereg.h>

extern "C"
{
#include <md5/md5.h>
}

#include <fstream>
#include <string>
#include <map>
#include <vector>

namespace putki
{
	namespace objstore
	{
		struct file_entry
		{
			file_entry()
			{
				parsed = 0;
				memset(&info, 000, sizeof(info));
				content_bytes = 0;
				content_length = 0;
				from_cache = false;
				has_objects = false;
			}
			parse::data* parsed;
			std::string path;
			std::string full_path;
			std::string signature;
			sys::file_info info;
			bool from_cache;
			const char *content_bytes;
			size_t content_length;
			bool has_objects;
		};

		struct object_entry
		{
			file_entry* file;
			std::string path;
			std::string signature;
			type_handler_i* th;
			parse::node* node;
		};

		struct resource_entry
		{
			file_entry* file;
			std::string path;
			std::string signature;
			size_t size;
			bool cached;
		};

		typedef std::multimap<std::string, object_entry*> CacheMap;
		typedef std::map<std::string, object_entry*> ObjMap;
		typedef std::map<std::string, file_entry*> FileMap;
		typedef std::multimap<std::string, resource_entry> ResMap;

		struct data
		{
			std::string root;
			std::string cache_file;
			bool is_cache;
			std::vector<file_entry*> files;
			std::vector<object_entry*> all_objs;
			FileMap file_map;
			CacheMap obj_cache;
			ObjMap objs;
			ResMap resources;
		};

		bool load_file(const char* path, const char** outBytes, size_t* outSize)
		{
			std::ifstream f(path, std::ios::binary);
			if (!f.good())
			{
				APP_WARNING("Failed to load file [" << path << "]")
					return false;
			}

			f.seekg(0, std::ios::end);
			std::streampos size = f.tellg();
			f.seekg(0, std::ios::beg);

			char *b = new char[(size_t)size];
			f.read(b, size);
			*outBytes = b;
			*outSize = (size_t)size;
			return true;
		}

		std::string file_signature(const char* path, size_t* size)
		{
			const char *bytes;
			size_t sz;
			if (!load_file(path, &bytes, &sz))
			{
				*size = 0;
				return "";
			}
			else
			{
				char signature[64];
				char signature_string[64];
				md5_buffer(bytes, (unsigned int)sz, signature);
				md5_sig_to_string(signature, signature_string, 64);
				*size = sz;
				delete[] bytes;
				return signature_string;
			}
		}

		object_entry* insert_obj_entry(data* d, file_entry* file, const char* path)
		{
			object_entry* oe;
			ObjMap::iterator i = d->objs.find(path);
			if (i == d->objs.end())
			{
				oe = new object_entry();
				oe->path = path;
				d->all_objs.push_back(oe);
				d->objs.insert(std::make_pair(oe->path, oe));
			}
			else
			{
				oe = i->second;
			}
			oe->file = file;
			oe->node = 0;
			oe->th = 0;
			return oe;
		}

		void examine_object_file(data* d, file_entry* file)
		{
			size_t pos = file->path.find_last_of('.');
			if (pos == std::string::npos)
			{
				return;
			}
			if (strcmp(file->path.substr(pos, file->path.size() - pos).c_str(), ".json"))
			{
				return;
			}

			std::string fn2 = file->path.substr(0, pos);
			std::string objname;
			bool is_cache = false;

			size_t sig = fn2.find_last_of('.');
			if (sig != std::string::npos)
			{
				objname = file->path.substr(0, sig);
				is_cache = true;
			}
			else
			{
				objname = fn2;
			}

			parse::data *pd;
			if (file->content_bytes != 0)
			{
				pd = parse::parse_json((char*)file->content_bytes, file->content_length);
				// takes ownership on successful parse.
				if (!pd)
				{
					delete[] file->content_bytes;
				}
				file->content_bytes = 0;
				file->content_length = 0;
			}
			else
			{
				pd = parse::parse(file->full_path.c_str());
			}

			if (!pd)
			{
				APP_INFO("Parse error in file [" << file->full_path << "]");
				return;
			}

			file->parsed = pd;

			file->has_objects = true;

			parse::node *root = parse::get_root(pd);
			std::string objtype = parse::get_value_string(parse::get_object_item(root, "type"));
			type_handler_i *th = typereg_get_handler(objtype.c_str());
			if (th)
			{
				object_entry* e = insert_obj_entry(d, file, objname.c_str());
				e->signature.clear();
				e->node = parse::get_object_item(root, "data");
				e->th = th;
				d->all_objs.push_back(e);
				if (is_cache)
				{
					d->obj_cache.insert(std::make_pair(e->signature, e));
				}
				else
				{
					d->objs.insert(std::make_pair(objname, e));
				}
			}
			else
			{
				APP_WARNING("Unrecognized type [" << objtype << "]");
			}

			parse::node *aux = parse::get_object_item(root, "aux");
			if (aux)
			{
				for (int i = 0;; i++)
				{
					parse::node *aux_obj = parse::get_array_item(aux, i);
					if (!aux_obj)
					{
						break;
					}
					std::string objtype = parse::get_value_string(parse::get_object_item(aux_obj, "type"));
					std::string auxpath = std::string(objname) + parse::get_value_string(parse::get_object_item(aux_obj, "ref"));
					type_handler_i *th = typereg_get_handler(objtype.c_str());
					if (th)
					{
						object_entry* e = insert_obj_entry(d, file, auxpath.c_str());
						e->node = parse::get_object_item(aux_obj, "data");
						e->th = th;
						e->signature.clear();
						if (is_cache)
						{
							APP_ERROR("Cache should not have aux objs!");
						}
					}
				}
			}
		}

		void examine_resource_file(const char *fullname, const char *name, void *userptr)
		{
			data* d = (data *)userptr;
			std::string fn(name);

			bool cached = false;
			size_t sig = fn.find_last_of('#');
			if (sig != std::string::npos)
			{
				std::string signature = fn.substr(sig + 1, fn.size() - sig - 1);
				fn = fn.substr(0, sig);

				size_t dot = signature.find_last_of('.');
				if (dot != std::string::npos)
				{
					signature.erase(0, dot);
					fn.append(signature);
				}
				cached = true;
			}

			file_entry* fe = new file_entry();
			fe->path = name;
			fe->full_path = fullname;
			d->files.push_back(fe);

			resource_entry e;
			e.file = fe;
			e.path = fn;
			e.cached = cached;
			e.signature = file_signature(fullname, &e.size);
			d->resources.insert(std::make_pair(fn, e));
		}

		void examine_file_object(const char *fullname, const char *name, void *userptr)
		{
			std::string fn(name);
			size_t pos = fn.find_last_of('.');
			if (pos == std::string::npos)
			{
				return;
			}
			if (strcmp(fn.substr(pos, fn.size() - pos).c_str(), ".json"))
			{
				return;
			}
			file_entry* fe = new file_entry();
			fe->full_path = fullname;
			fe->path = name;
			fe->has_objects = true;
			fe->parsed = 0;
			data* d = (data *)userptr;
			d->file_map.insert(std::make_pair(std::string(fullname), fe));
		}

		void write_cache(data *d)
		{
			typedef std::multimap<file_entry*, CacheMap::iterator> File2Cache;
			typedef std::multimap<file_entry*, ObjMap::iterator> File2Obj;

			File2Obj f2o, f2c;
			for (CacheMap::iterator q = d->obj_cache.begin(); q != d->obj_cache.end(); q++)
			{
				f2c.insert(std::make_pair(q->second->file, q));
			}
			for (ObjMap::iterator q = d->objs.begin(); q != d->objs.end(); q++)
			{
				f2o.insert(std::make_pair(q->second->file, q));
			}

			std::ofstream cf(d->cache_file);
			FileMap::iterator fs = d->file_map.begin();
			while (fs != d->file_map.end())
			{
				cf << fs->first << "\n";
				cf << fs->second->info.mtime << "\n";
				cf << fs->second->info.size << "\n";
				if (fs->second->signature.empty())
					cf << "?\n";
				else
					cf << fs->second->signature << "\n";
				if (fs->second->has_objects)
				{
					if (d->is_cache)
					{
						// This is for the .tmp and .built domain, we write out by the cache store here, since those will get 
						// uncached that way. There is no need for writing the uncached "non-signatured" objects.
						std::pair<File2Cache::iterator, File2Cache::iterator> c = f2c.equal_range(fs->second);
						cf << std::distance(c.first, c.second) << "\n";
						for (File2Cache::iterator i = c.first; i != c.second; i++)
						{
							cf << i->second->second->path << "\n";
							cf << i->second->second->th->name() << "\n";
							cf << i->second->second->signature << "\n";
							cf << "1\n";
						}
					}
					else
					{
						std::pair<File2Cache::iterator, File2Cache::iterator> d = f2o.equal_range(fs->second);
						cf << std::distance(d.first, d.second) << "\n";
						for (File2Cache::iterator i = d.first; i != d.second; i++)
						{
							cf << i->second->first << "\n";
							cf << i->second->second->th->name() << "\n";
							if (!i->second->second->signature.empty())
								cf << i->second->second->signature << "\n";
							else
								cf << "?\n";
							cf << "0\n";
						}
					}
				}
				else
				{
					cf << "-1\n";
				}
				fs++;
			}
		}

		data* open(const char *root_path, bool is_cache)
		{
			data* d = new data();
			d->is_cache = is_cache;
			d->root = root_path;
			while (d->root.size() > 0 && d->root[d->root.size() - 1] == '/')
				d->root.pop_back();

			d->cache_file = d->root + "/.cache";
			sys::search_tree((d->root + "/objs").c_str(), examine_file_object, d);
			sys::search_tree((d->root + "/res").c_str(), examine_resource_file, d);

			FileMap::iterator fs = d->file_map.begin();
			while (fs != d->file_map.end())
			{
				if (!sys::stat(fs->first.c_str(), &fs->second->info))
				{
					APP_WARNING("Failed to stat file [" << fs->first << "]");
				}
				d->files.push_back(fs->second);
				fs++;
			}

			tok::data* cache = tok::load(d->cache_file.c_str());
			if (cache)
			{
				tok::tokenize_newlines(cache);
				int ptr = 0;
				while (true)
				{
					const char* hdr[5] = {
						tok::get(cache, ptr),     // path
						tok::get(cache, ptr + 1), // mtime
						tok::get(cache, ptr + 2), // size
						tok::get(cache, ptr + 3), // signature
						tok::get(cache, ptr + 4), // objects
					};
					ptr += 5;
					if (!hdr[3] || !hdr[4])
					{
						break;
					}

					const int objects = atoi(hdr[4]);
					const int objs_start = ptr;

					if (objects != -1)
					{
						ptr = ptr + 4 * objects;
					}

					FileMap::iterator i = d->file_map.find(hdr[0]);
					if (i == d->file_map.end())
					{
						APP_DEBUG(hdr[0] << " from cache has been removed.");
						continue;
					}

					long mtime = atol(hdr[1]);
					long size = atol(hdr[2]);
					if (mtime != i->second->info.mtime || size != i->second->info.size || hdr[3][0] == '?')
					{
						if (!load_file(hdr[0], &i->second->content_bytes, &i->second->content_length))
						{
							APP_ERROR("Could not read " << hdr[0]);
							break;
						}
						signature::buffer buf;
						i->second->signature = signature::resource(i->second->content_bytes, i->second->content_length, buf);
						if (strcmp(i->second->signature.c_str(), hdr[3]))
						{
							APP_DEBUG("  Signature changed on " << hdr[0] << " from " << hdr[3] << " to " << i->second->signature.c_str());
							continue;
						}
						continue;
					}

					i->second->signature = hdr[3];
					i->second->from_cache = true;

					for (int k = 0; k < objects; k++)
					{
						const char* obj[4] = {
							tok::get(cache, objs_start + 4 * k),     // path
							tok::get(cache, objs_start + 4 * k + 1), // type
							tok::get(cache, objs_start + 4 * k + 2), // signature
							tok::get(cache, objs_start + 4 * k + 3), // cache
						};

						object_entry* oe = new object_entry();
						oe->path = obj[0];
						if (obj[2][0] != '?')
						{
							oe->signature = obj[2];
						}
						oe->th = typereg_get_handler(obj[1]);
						oe->file = i->second;
						oe->node = 0;
						if (atoi(obj[3]))
						{
							d->obj_cache.insert(std::make_pair(oe->signature, oe));
						}
						else
						{
							d->objs.insert(std::make_pair(oe->path, oe));
						}
						d->all_objs.push_back(oe);
					}
				}
				tok::free(cache);
			}

			FileMap::iterator cf = d->file_map.begin();
			while (cf != d->file_map.end())
			{
				if (!cf->second->from_cache)
				{
					APP_DEBUG("  Reading non-cached file [" << cf->first << "]");
					if (!cf->second->content_bytes)
					{
						if (!load_file(cf->first.c_str(), &cf->second->content_bytes, &cf->second->content_length))
						{
							APP_ERROR("  Could not read [" << cf->first << "]");
						}
					}
					signature::buffer buf;
					cf->second->signature = signature::resource(cf->second->content_bytes, cf->second->content_length, buf);
					examine_object_file(d, cf->second);
				}
				cf++;
			}

			// Read-only so we can write here now.
			if (!d->is_cache)
			{
				write_cache(d);
			}

			return d;
		}

		void free(data *d)
		{
			// Non-cache is updated with signatures as objects are read, so store a second time here.
			write_cache(d);

			for (size_t i = 0; i < d->files.size(); i++)
			{
				if (d->files[i]->parsed)
				{
					parse::free(d->files[i]->parsed);
				}
				if (d->files[i]->content_bytes)
				{
					delete[] d->files[i]->content_bytes;
				}
				delete d->files[i];
			}
			delete d;
		}

		bool query_object(data* d, const char *path, object_info* result)
		{
			ObjMap::iterator i = d->objs.find(path);
			if (i != d->objs.end())
			{
				if (i->second->signature.empty())
				{
					APP_DEBUG("Unknown signature on [" << path << "] so I will load and see...");
					fetch_obj_result result;
					if (fetch_object(d, path, &result))
					{
						signature::buffer buf;
						i->second->signature = signature::object(result.th, result.obj, buf);
						result.th->free(result.obj);
					}
					else
					{
						APP_ERROR("Could not fetch object [" << path << "] even though it was in objset!");
						return false;
					}
				}
				result->signature = i->second->signature;
				result->th = i->second->th;
				return true;
			}
			return false;
		}

		// Fetches -the- uncached.
		bool fetch_object(data* d, const char* path, fetch_obj_result* result)
		{
			ObjMap::iterator i = d->objs.find(path);
			if (i == d->objs.end())
			{
				result->obj = 0;
				result->th = 0;
				return false;
			}

			object_entry* o = i->second;
			type_handler_i* th = i->second->th;
			if (o->node)
			{
				result->th = th;
				result->obj = th->alloc();
				th->fill_from_parsed(o->node, result->obj);
				signature::buffer buf;
				o->signature = signature::object(th, result->obj, buf);
				return true;
			}

			if (o->file->parsed)
			{
				APP_ERROR("File has parse data, but object has not?!");
				return false;
			}

			examine_object_file(d, o->file);

			ObjMap::iterator j = d->objs.find(path);
			if (j->second->node)
			{
				result->th = th;
				result->obj = th->alloc();
				th->fill_from_parsed(j->second->node, result->obj);
				signature::buffer buf;
				o->signature = signature::object(th, result->obj, buf);
				return true;
			}

			APP_ERROR("Could not read [" << path << "] from file [" << o->file->full_path << "]!");
			return false;
		}

		void fetch_object_free(fetch_obj_result* result)
		{

		}

		bool query_resource(data* d, const char *path, resource_info* result)
		{
			std::pair<ResMap::iterator, ResMap::iterator> range = d->resources.equal_range(path);
			for (ResMap::iterator i = range.first; i != range.second; i++)
			{
				if (!i->second.cached)
				{
					result->signature = i->second.signature;
					result->size = i->second.size;
					return true;
				}
			}
			return false;
		}

		resource_entry* find_res(data *d, const char* path, const char* signature)
		{
			std::pair<ResMap::iterator, ResMap::iterator> range = d->resources.equal_range(path);
			for (ResMap::iterator i = range.first; i != range.second; i++)
			{
				if (!strcmp(i->second.signature.c_str(), signature))
				{
					return &i->second;
				}
			}
			return 0;
		}

		size_t read_resource_range(data *d, const char* path, const char* signature, char* output, size_t beg, size_t end)
		{
			resource_entry* e = find_res(d, path, signature);
			APP_DEBUG("Reading range " << beg << " to " << end << " from " << path);
			if (e)
			{
				std::ifstream f(e->file->full_path.c_str(), std::ios::binary);
				if (f.good())
				{
					f.seekg(beg, std::ios::beg);
					f.read(output, end - beg);
					return (size_t)f.gcount();
				}
				APP_WARNING("Failed to load file [" << path << "]")
			}
			return 0;
		}

		size_t query_by_type(data* d, type_handler_i* th, const char** paths, size_t len)
		{
			size_t count = 0;
			ObjMap::iterator i = d->objs.begin();
			while (i != d->objs.end())
			{
				if (i->second->th != th)
				{
					++i;
					continue;
				}
				if (count < len)
				{
					paths[count] = i->first.c_str();
				}
				++count;
				++i;
			}
			return count;
		}

		bool fetch_resource(data* d, const char* path, const char* signature, fetch_res_result* result)
		{
			resource_entry* e = find_res(d, path, signature);
			if (e)
			{
				if (load_file(e->file->full_path.c_str(), &result->data, &result->size))
				{
					return true;
				}
				APP_WARNING("Could not fetch resource [" << path << "] actual[" << e->file->path << "]");
			}
			return false;
		}

		void fetch_resource_free(fetch_res_result* result)
		{
			delete[] result->data;
			result->data = 0;
		}

		bool uncache_object(data* dest, data* source, const char *path, const char *signature)
		{
			std::pair<CacheMap::iterator, CacheMap::iterator> range = source->obj_cache.equal_range(signature);
			for (ObjMap::iterator i = range.first; i != range.second; i++)
			{
				if (!strcmp(i->second->path.c_str(), path))
				{
					file_entry* fe;
					// TODO: Think this through!
					FileMap::iterator q = dest->file_map.find(i->second->file->full_path.c_str());
					if (q != dest->file_map.end())
					{
						fe = q->second;
						fe->signature = i->second->file->signature;
					}
					else
					{
						fe = new file_entry();
						fe->path = i->second->file->path;
						fe->full_path = i->second->file->full_path;
						fe->has_objects = true;
						fe->signature = i->second->file->signature;
						fe->parsed = 0;
						dest->files.push_back(fe);
						dest->file_map.insert(std::make_pair(i->second->file->full_path, fe));
					}

					object_entry *o = new object_entry();
					dest->all_objs.push_back(o);
					o->file = fe;
					o->node = 0;
					o->path = path;
					o->th = i->second->th;
					o->signature = signature;
					dest->objs.insert(std::make_pair(path, o));
					return true;
				}
			}
			return false;
		}

		bool store_object(data* d, const char *path, type_handler_i* th, instance_t obj, const char *signature)
		{
			std::string out_path(d->root);
			out_path.append("/objs/");
			out_path.append(path);
			out_path.append(".");
			out_path.append(signature);
			out_path.append(".json");
			putki::sstream ts;
			write::write_object_into_stream(ts, th, obj);
			sys::mk_dir_for_path(out_path.c_str());
			if (!sys::write_file(out_path.c_str(), ts.str().c_str(), (unsigned long)ts.str().size()))
			{
				return false;
			}

			std::string fn(path);
			fn.append(".");
			fn.append(signature);
			fn.append(".json");

			file_entry* fe;
			// TODO: Think this through!
			FileMap::iterator q = d->file_map.find(out_path.c_str());
			if (q != d->file_map.end())
			{
				fe = q->second;
			}
			else
			{
				fe = new file_entry();
				fe->path = fn;
				fe->full_path = out_path.c_str();
				fe->has_objects = true;
				fe->parsed = 0;
				d->files.push_back(fe);
				d->file_map.insert(std::make_pair(fe->full_path, fe));
			}

			sys::stat(fe->full_path.c_str(), &fe->info);

			// actually, could just take the 'signature' as we know it will be the same,
			// but for future compatibility if object signature computation changes...
			signature::buffer buf;
			fe->signature = signature::resource(ts.str().c_str(), ts.str().size(), buf);

			object_entry* o = new object_entry();
			o->file = fe;
			o->path = path;
			o->signature = signature;
			o->node = 0;
			o->th = th;
			d->all_objs.push_back(o);
			d->obj_cache.insert(std::make_pair(signature, o));
			return uncache_object(d, d, path, signature);
		}

		bool store_resource(data* d, const char *path, const char* data, size_t length)
		{
			char signature[64];
			char signature_string[64];
			md5_buffer(data, (unsigned int)length, signature);
			md5_sig_to_string(signature, signature_string, 64);

			std::string fn(path);
			std::string out_fn;
			size_t dot = fn.find_last_of('.');
			if (dot != std::string::npos)
			{
				out_fn.append(fn.substr(0, dot));
				out_fn.append("#");
				out_fn.append(signature_string);
				out_fn.append(fn.substr(dot, fn.size() - dot));
			}
			else
			{
				out_fn.append(fn);
				out_fn.append("#");
				out_fn.append(signature_string);
			}

			std::string out_path(d->root);
			out_path.append("/res/");
			out_path.append(out_fn);

			sys::mk_dir_for_path(out_path.c_str());
			if (!sys::write_file(out_path.c_str(), data, length))
			{
				return false;
			}

			examine_resource_file(out_path.c_str(), out_fn.c_str(), d);
			return uncache_resource(d, d, path, signature_string);
		}

		bool uncache_resource(data* dest, data* source, const char *path, const char *signature)
		{
			std::pair<ResMap::iterator, ResMap::iterator> range = source->resources.equal_range(path);
			for (ResMap::iterator i = range.first; i != range.second; i++)
			{
				if (i->second.cached && !strcmp(i->second.signature.c_str(), signature))
				{
					file_entry* fe = new file_entry();
					fe->full_path = i->second.file->full_path;
					fe->path = i->second.file->path;
					fe->signature = i->second.file->signature;
					fe->parsed = 0;
					dest->files.push_back(fe);
					resource_entry re;
					re.cached = false;
					re.path = path;
					re.file = fe;
					re.signature = signature;
					re.size = i->second.size;
					dest->resources.insert(std::make_pair(path, re));
					return true;
				}
			}
			return false;
		}
	}
}
