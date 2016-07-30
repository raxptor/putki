#include "objstore.h"

#include <putki/sys/files.h>
#include <putki/builder/signature.h>
#include <putki/builder/log.h>
#include <putki/builder/parse.h>
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
			}
			parse::data* parsed;
			std::string path;
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

		typedef std::multimap<std::string, resource_entry> ResMap;

		struct data
		{
			std::string root;
			std::vector<file_entry*> files;
			std::vector<object_entry*> all_objs;
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
			*outSize = (size_t) size;
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
				delete [] bytes;
				return signature_string;
			}
		}

		void examine_object_file(const char *fullname, const char *name, void *userptr)
		{
			data* d = (data *)userptr;
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

			std::string fn2 = fn.substr(0, pos);
			std::string objname;
			bool is_cache = false;

			size_t sig = fn2.find_last_of('.');
			if (sig != std::string::npos)
			{
				objname = fn.substr(0, sig);
				is_cache = true;
			}
			else
			{
				objname = fn2;
			}

			parse::data *pd = parse::parse(fullname);
			if (!pd)
			{
				APP_INFO("Parse error in file [" << fullname << "]");
				return;
			}

			file_entry* fe = new file_entry();
			fe->path = fullname;
			fe->parsed = pd;
			d->files.push_back(fe);

			parse::node *root = parse::get_root(pd);
			std::string objtype = parse::get_value_string(parse::get_object_item(root, "type"));
			type_handler_i *th = typereg_get_handler(objtype.c_str());
			if (th)
			{
				instance_t obj = th->alloc();
				th->fill_from_parsed(parse::get_object_item(root, "data"), obj);

				object_entry* e = new object_entry();
				e->file = fe;
				e->path = objname;

				signature::buffer sigbuf;
				e->signature = signature::object(th, obj, sigbuf);
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
				th->free(obj);
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
						instance_t obj = th->alloc();
						th->fill_from_parsed(parse::get_object_item(aux_obj, "data"), obj);
						object_entry* e = new object_entry();
						d->all_objs.push_back(e);
						e->file = fe;
						e->path = auxpath;
						e->node = parse::get_object_item(aux_obj, "data");
						e->th = th;
						signature::buffer sigbuf;
						e->signature = signature::object(th, obj, sigbuf);
						if (is_cache)
						{
							d->obj_cache.insert(std::make_pair(e->signature, e));
						}
						else
						{
							d->objs.insert(std::make_pair(auxpath, e));
						}
						th->free(obj);
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
			fe->path = fullname;
			d->files.push_back(fe);

			resource_entry e;
			e.file = fe;
			e.path = fn;
			e.cached = cached;
			e.signature = file_signature(fullname, &e.size);
			d->resources.insert(std::make_pair(fn, e));
		}

		data* open(const char *root_path)
		{
			data* d = new data();
			d->root = root_path;
			sys::search_tree((d->root + "/objs").c_str(), examine_object_file, d);
			sys::search_tree((d->root + "/res").c_str(), examine_resource_file, d);

			ObjMap::iterator i = d->objs.begin();
			while (i != d->objs.end())
			{
				APP_DEBUG("obj [" << i->first << "] sig=" << i->second->signature << " file=" << i->second->file->path);
				++i;
			}
			ResMap::iterator j = d->resources.begin();
			while (j != d->resources.end())
			{
				APP_DEBUG("res [" << j->first << "] sig=" << j->second.signature << " file=" << j->second.file->path);
				++j;
			}
			return d;
		}

		void free(data *d)
		{
			for (size_t i=0;i<d->files.size();i++)
			{
				if (d->files[i]->parsed)
				{
					parse::free(d->files[i]->parsed);
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
				return true;
			}

			file_entry* f = o->file;
			if (f->parsed)
			{
				APP_ERROR("File has parse data, but object has not?!");
				return false;
			}

			f->parsed = parse::parse(f->path.c_str());
			if (!f->parsed)
			{
				APP_INFO("Parse error in file [" << path << "]");
				return false;
			}

			parse::node *root = parse::get_root(f->parsed);
			i->second->node = parse::get_object_item(root, "data");
			result->th = th;
			result->obj = th->alloc();
			th->fill_from_parsed(o->node, result->obj);
			return true;
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
				std::ifstream f(e->file->path.c_str(), std::ios::binary);
				if (f.good())
				{
					f.seekg(beg, std::ios::beg);
					f.read(output, end - beg);
					return f.gcount();
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
				if (load_file(e->file->path.c_str(), &result->data, &result->size))
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
					file_entry* f = new file_entry();
					f->path = i->second->file->path;
					dest->files.push_back(f);
					object_entry *o = new object_entry();
					dest->all_objs.push_back(o);
					o->file = f;
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

			file_entry* fe = new file_entry();
			fe->path = out_path.c_str();

			d->files.push_back(fe);
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
				out_fn.append(fn.substr(dot, fn.size()-dot));
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
					fe->path = i->second.file->path;
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
