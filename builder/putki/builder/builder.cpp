#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>
#include <putki/builder/signature.h>
#include <putki/builder/build-db.h>
#include <putki/builder/ptr.h>
#include "builder.h"

#include <map>
#include <set>
#include <queue>

namespace putki
{
	namespace builder
	{
		struct loaded
		{
			type_handler_i* th;
			instance_t obj;
			std::string signature;
		};

		typedef std::multimap<int, handler_info> HandlerMapT;
		typedef std::map<std::string, loaded> LoadedT;
		typedef std::vector<const char*> AllocStringsT;

		struct to_build_entry
		{
			std::string path;
			int domain;
		};

		struct data
		{
			config conf;
			HandlerMapT handlers;
			std::queue<to_build_entry> to_build;
			std::set<std::string> has_added;
			LoadedT loaded_input;
			LoadedT loaded_temp;
			LoadedT loaded_built;
			std::vector<const char*> str_allocs;
		};
		
		struct build_info_internal
		{
			data* d;
			std::vector<ptr_raw> outputs;
			ptr_context* ptr_ctx;
		};

		data* create(config* conf)
		{
			data* d = new data();
			d->conf = *conf;
			return d;
		}

		void free_strings(data* d)
		{
			for (size_t i = 0; i < d->str_allocs.size(); i++)
			{
				::free((void*)d->str_allocs[i]);
			}
			d->str_allocs.clear();
		}

		void clear_cache(LoadedT* cache)
		{
			LoadedT::iterator i = cache->begin();
			while (i != cache->end())
			{
				i->second.th->free(i->second.obj);
				++i;
			}
			cache->clear();
		}

		void clear(data* d)
		{
			clear_cache(&d->loaded_input);
			clear_cache(&d->loaded_temp);
			clear_cache(&d->loaded_built);
			free_strings(d);
			d->has_added.clear();
		}

		void free(data *d)
		{
			clear(d);
			delete d;
		}

		void add_handlers(data* d, const handler_info* begin, const handler_info* end)
		{
			for (const handler_info* i = begin; i != end; i++)
			{
				type_handler_i* th = typereg_get_handler(i->type_id);
				while (th)
				{
					d->handlers.insert(std::make_pair(th->id(), *i));
					th = th->parent_type();
				}
			}
		}

		std::string builder_name(data *d, int type_id)
		{
			std::string builder_name;
			std::pair<HandlerMapT::iterator, HandlerMapT::iterator> hs = d->handlers.equal_range(type_id);
			for (HandlerMapT::iterator i = hs.first; i != hs.second; i++)
			{
				if (!builder_name.empty())
					builder_name.append("&");

				builder_name.append(i->second.name);

				// Compute this once maybe. In code generator?
				type_handler_i* th = typereg_get_handler(i->type_id);
				if (th)
				{
					instance_t empty = th->alloc();
					signature::buffer buf;
					const char* sig = signature::object(th, empty, buf);
					builder_name.append("=" + std::string(sig, sig + 6))
				}
			}
			return builder_name.empty() ? "default" : builder_name;
		}

		void create_build_output(struct putki::builder::build_info const *info, struct putki::type_handler_i *th, char const *tag, struct putki::ptr_raw *ptr)
		{
			std::string actual(info->path);
			actual.append("-");
			actual.append(tag);

			RECORD_INFO(info->record, "Creating output object [" << actual << "] type=" << th->name());
			instance_t obj = th->alloc();
			ptr->path = strdup(actual.c_str());
			ptr->has_resolved = true;
			ptr->obj = obj;
			ptr->th = th;
			ptr->user_data = 1;
			ptr->ctx = info->internal->ptr_ctx;

			info->internal->d->str_allocs.push_back(ptr->path);
			info->internal->outputs.push_back(*ptr);
		}

		bool store_resource_path(const build_info* info, const char* path, const char* data, size_t size)
		{
			if (!objstore::store_resource(info->internal->d->conf.temp, path, data, size))
			{
				RECORD_ERROR(info->record, "Failed to store temp resource [" << path << "] size=" << size);
				return false;
			}
			objstore::resource_info ri;
			if (!objstore::query_resource(info->internal->d->conf.temp, path, &ri))
			{
				RECORD_ERROR(info->record, "Failed to query stored resource [" << path << "] size=" << size);
				return false;
			}

			build_db::add_output_resource(info->record, path, info->builder, ri.signature.c_str());
			return true;
		}
		
		std::string store_resource_tag(const build_info* info, const char* tag, const char* data, size_t size)
		{
			std::string actual(info->path);
			actual.append("-");
			actual.append(tag);
			return store_resource_path(info, actual.c_str(), data, size) ? actual : std::string("");
		}

		size_t read_resource_segment(const build_info* info, const char* path, char* output, size_t beg, size_t end)
		{
			objstore::resource_info ri;
			if (objstore::query_resource(info->internal->d->conf.temp, path, &ri))
			{
				return objstore::read_resource_range(info->internal->d->conf.temp, path, ri.signature.c_str(), output, beg, end);
			}
			if (objstore::query_resource(info->internal->d->conf.input, path, &ri))
			{
				return objstore::read_resource_range(info->internal->d->conf.input, path, ri.signature.c_str(), output, beg, end);
			}
			return 0;
		}

		bool fetch_resource(const build_info* info, const char* path, resource* resource)
		{
			objstore::resource_info ri;
			if (objstore::query_resource(info->internal->d->conf.temp, path, &ri))
			{
				if (objstore::fetch_resource(info->internal->d->conf.temp, path, ri.signature.c_str(), &resource->internal))
				{
					resource->signature = strdup(ri.signature.c_str());
					resource->data = resource->internal.data;
					resource->size = resource->internal.size;
					build_db::add_external_resource_dependency(info->record, path, resource->signature);
					return true;
				}
				else
				{
					build_db::add_external_resource_dependency(info->record, path, "file-not-found");
					APP_WARNING("Thought i could fetch resource [" << path << "] from tmp but then i couldn't!");
					return false;
				}
			}
			if (objstore::query_resource(info->internal->d->conf.input, path, &ri))
			{
				if (objstore::fetch_resource(info->internal->d->conf.input, path, ri.signature.c_str(), &resource->internal))
				{
					resource->signature = strdup(ri.signature.c_str());
					resource->data = resource->internal.data;
					resource->size = resource->internal.size;
					build_db::add_external_resource_dependency(info->record, path, resource->signature);
					return true;
				}
				else
				{
					APP_WARNING("Thought i could fetch resource [" << path << "] from input but then i couldn't!");
				}
			}
			build_db::add_external_resource_dependency(info->record, path, "file-not-found");
			return false;
		}
		
		void free_resource(resource* resource)
		{
			::free((void*)resource->signature);
			objstore::fetch_resource_free(&resource->internal);
		}

		void add_build_root(data *d, const char *path, int domain)
		{
			if (!d->has_added.count(path))
			{
				d->has_added.insert(path);

				to_build_entry tb;
				tb.path = path;
				tb.domain = domain;
				d->to_build.push(tb);
			}
		}

		struct free_deplist_obj
		{
			build_db::deplist* dl;
			~free_deplist_obj()
			{
				build_db::deplist_free(dl);
			}
		};
		
		bool fetch_cached(data* d, const char* path, objstore::object_info* info, const char* bname, build_db::InputDepSigs& sigs)
		{
			build_db::record* find = build_db::find_cached(d->conf.build_db, path, info->signature.c_str(), bname, sigs);
			if (!find)
			{
				return false;
			}

			build_db::deplist* dl = build_db::inputdeps_get(find);
			free_deplist_obj freeer;
			freeer.dl = dl;

			int di = 0;
			while (true)
			{
				const char* dep = build_db::deplist_path(dl, di);
				if (!dep)
				{
					break;
				}
				if (!deplist_is_external_resource(dl, di))
				{
					objstore::object_info dep_info;
					if (objstore::query_object(d->conf.input, dep, &dep_info))
					{
						if (strcmp(dep_info.signature.c_str(), build_db::deplist_signature(dl, di)))
						{
							APP_DEBUG("fetch_cached: obj-dep check for [" << dep << "] => " << dep_info.signature << " (record had " << build_db::deplist_signature(dl, di) << ")");
							sigs.insert(std::make_pair(std::string(dep), dep_info.signature));
							return fetch_cached(d, path, info, bname, sigs);
						}
					}
					else if (objstore::query_object(d->conf.temp, dep, &dep_info))
					{
						if (strcmp(dep_info.signature.c_str(), build_db::deplist_signature(dl, di)))
						{
							APP_DEBUG("fetch_cached: obj-dep check for [" << dep << "] => " << dep_info.signature << " (record had " << build_db::deplist_signature(dl, di) << ")");
							sigs.insert(std::make_pair(std::string(dep), dep_info.signature));
							return fetch_cached(d, path, info, bname, sigs);
						}
					}
					else
					{
						APP_DEBUG("fetch_cached => unknown input object [" << dep << "]");
						return false;
					}
				}
				else
				{
					objstore::resource_info res_info;
					if (objstore::query_resource(d->conf.input, dep, &res_info))
					{
						if (strcmp(res_info.signature.c_str(), build_db::deplist_signature(dl, di)))
						{
							APP_DEBUG("fetch_cached: res-dep check for [" << dep << "] => " << res_info.signature << " (record had " << build_db::deplist_signature(dl, di) << ")");
							sigs.insert(std::make_pair(std::string(dep), res_info.signature));
							return fetch_cached(d, path, info, bname, sigs);
						}
					}
					else if (objstore::query_resource(d->conf.temp, dep, &res_info))
					{
						if (strcmp(res_info.signature.c_str(), build_db::deplist_signature(dl, di)))
						{
							APP_DEBUG("fetch_cached: res-dep check for [" << dep << "] => " << res_info.signature << " (record had " << build_db::deplist_signature(dl, di) << ")");
							sigs.insert(std::make_pair(std::string(dep), res_info.signature));
							return fetch_cached(d, path, info, bname, sigs);
						}
					}
					else if (!strcmp(build_db::deplist_signature(dl, di), "file-not-found"))
					{
						APP_DEBUG("fetch_cached: obj still not exists");
					}
					else
					{
						APP_DEBUG("fetch_cached => [" << dep << "] does not exist any longer.");
						return false;
					}
				}
				++di;
			}

			APP_DEBUG("fetch_cached: I have a match on " << path << " for builder " << bname);
			build_db::InputDepSigs::iterator i = sigs.begin();
			while (i != sigs.end())
			{
				APP_DEBUG("fetch_cached:  filter[" << i->first << "] => [" << i->second << "]");
				++i;
			}

			int o = 0;
			while (true)
			{
				bool is_resource;
				const char* out = build_db::enum_outputs(find, o, &is_resource);
				if (!out)
				{
					break;
				}
				const char* out_sig = get_output_signature(find, o);

				if (is_resource)
				{
					if (!objstore::uncache_resource(d->conf.temp, out, out_sig))
					{
						// Is cleanup here actually needed? Probably not.
						APP_DEBUG("Could not uncache resource " << out << " sig=" << out_sig);
						return false;
					}
					else
					{
						APP_DEBUG("Uncached tmp resource " << out << " sig=" << build_db::get_signature(find));
					}
				}
				else
				{
					if (!objstore::uncache_object(d->conf.temp, out, out_sig))
					{
						// Is cleanup here actually needed? Probably not.
						APP_DEBUG("Could not uncache object " << out << " sig=" << out_sig);
						return false;
					}
					else
					{
						APP_DEBUG("Uncached tmp object " << out << " sig=" << build_db::get_signature(find));
					}
				}
				++o;
			}

			if (!objstore::uncache_object(d->conf.built, path, build_db::get_signature(find)))
			{
				APP_DEBUG("Could not uncache object " << path << " sig=" << build_db::get_signature(find));
				return false;
			}

			int p = 0;
			while (true)
			{
				const char* ptr = build_db::get_pointer(find, p++);
				if (!ptr)
				{
					break;
				}
				if (!d->has_added.count(ptr))
				{
					to_build_entry p;
					p.path = ptr;
					p.domain = 0;
					objstore::object_info oi;
					if (objstore::query_object(d->conf.temp, ptr, &oi))
					{
						p.domain = 1;
					}

					d->to_build.push(p);
					d->has_added.insert(ptr);
				}
			}

			build_db::commit_cached_record(d->conf.build_db, find);
			return true;
		}

		bool fetch_cached(data* d, const char* path, objstore::object_info* info, const char* bname)
		{
			build_db::InputDepSigs sigs;
			return fetch_cached(d, path, info, bname, sigs);
		}

		void fixup_pointers(data* d, type_handler_i* th, instance_t obj, ptr_context* context, const char* root_path)
		{
			ptr_query_result result;
			th->query_pointers(obj, &result, false, true);
			for (size_t i = 0; i < result.pointers.size(); i++)
			{
				ptr_raw *p = result.pointers[i];
				p->ctx = context;
				if (p->path == 0 || !p->path[0])
				{
					continue;
				}
				if (p->path[0] == '#')
				{
					std::string actual(root_path);
					size_t already = actual.find_last_of('#');
					if (already != std::string::npos)
					{
						actual.erase(actual.begin() + already, actual.end());
					}
					actual.append(p->path);
					p->path = strdup(actual.c_str());
					d->str_allocs.push_back(p->path);
				}
				
				objstore::object_info info;
				if (objstore::query_object(d->conf.temp, p->path, &info))
				{
					p->user_data = 1;
				}
				else
				{
					p->user_data = 0;
				}
			}
		}
		
		struct ptr_ctx_data
		{
			data* d;
			std::set<const char*> visited;
		};

		void ptr_resolve_internal(ptr_raw* ptr, bool allow_cache, objstore::object_info* info)
		{
			if (ptr->path == 0 || !ptr->path[0])
			{
				ptr->obj = 0;
				return;
			}

			ptr_ctx_data* pcd = (ptr_ctx_data*)ptr->ctx->user_data;
			data* d = pcd->d;
			objstore::data* store;
			LoadedT* cache;
			switch (ptr->user_data)
			{
			case 0:
				store = d->conf.input;
				cache = &d->loaded_input;
				break;
			case 1:
				store = d->conf.temp;
				cache = &d->loaded_temp;
				break;
			case 2:
				store = d->conf.built;
				cache = &d->loaded_built;
				break;
			default:
				APP_ERROR("Invalid ptr domain. I do not know from which store to get it.");
				ptr->obj = 0;
				break;
			}

			// TODO: Verify that pointers are compatible with what is actually being loaded.

			if (allow_cache)
			{
				LoadedT::iterator i = cache->find(ptr->path);
				if (i != cache->end())
				{
					ptr->obj = i->second.obj;
					ptr->th = i->second.th;
					info->signature = i->second.signature;
					info->th = i->second.th;
					return;
				}
			}

			if (!objstore::query_object(store, ptr->path, info))
			{
				APP_ERROR("Unable to resolve " << ptr->path);
				ptr->obj = 0;
				return;
			}

			objstore::fetch_obj_result result;
			if (!objstore::fetch_object(store, ptr->path, &result))
			{
				APP_ERROR("Unable to fetch " << ptr->path);
				ptr->obj = 0;
				return;
			}

			fixup_pointers(d, info->th, result.obj, ptr->ctx, ptr->path);

			if (allow_cache)
			{
				loaded l;
				l.obj = result.obj;
				l.th = info->th;
				l.signature = info->signature;
				cache->insert(std::make_pair(std::string(ptr->path), l));
			}

			ptr->th = info->th;
			ptr->obj = result.obj;
		}

		void ptr_resolve(ptr_raw* ptr)
		{
			objstore::object_info info;
			ptr_resolve_internal(ptr, true, &info);
		}

		void ptr_deref(const ptr_raw* ptr)
		{
			ptr_ctx_data* pcd = (ptr_ctx_data*) ptr->ctx->user_data;
			if (ptr->path != 0 && ptr->path[0] != 0)
			{
				pcd->visited.insert(ptr->path);
			}
		}

		void add_post_build_object(data* d, type_handler_i* th, instance_t obj, const char* path)
		{
			signature::buffer buf;
			const char* sig = signature::object(th, obj, buf);
			if (!objstore::store_object(d->conf.temp, path, th, obj, sig))
			{
				APP_WARNING("Failed to store post build object [" << path << "]");
			}
			add_build_root(d, path, 1);
		}

		void do_build(data *d, bool incremental)
		{
			ptr_ctx_data pcd;
			pcd.d = d;

			ptr_context pctx;
			pctx.user_data = (uintptr_t)&pcd;
			pctx.deref = ptr_deref;
			pctx.resolve = ptr_resolve;

			while (true)
			{
				if (d->to_build.empty())
				{
					break;
				}

				to_build_entry next = d->to_build.front();
				d->to_build.pop();

				const char* path = next.path.c_str();

				bool found = false;
				objstore::object_info info;
				switch (next.domain)
				{
					case 0: found = objstore::query_object(d->conf.input, path, &info); break;
					case 1: found = objstore::query_object(d->conf.temp, path, &info); break;
					case -1:
					{
						next.domain = 0;
						found = objstore::query_object(d->conf.input, path, &info);
						if (!found)
						{
							next.domain = 1;
							found = objstore::query_object(d->conf.temp, path, &info);
						}
						break;
					}
					default: break;
				}

				if (!found)
				{
					APP_ERROR("Could not find object [" << path << "] from domain " << next.domain);
					continue;
				}

				std::string bname = builder_name(d, info.th->id());
				if (incremental && fetch_cached(d, path, &info, bname.c_str()))
				{
					APP_DEBUG("Got cached object, no build needed.");
					continue;
				}

				ptr_raw source;
				source.ctx = &pctx;
				source.has_resolved = false;
				source.user_data = next.domain;
				source.path = path;

				APP_DEBUG("Processing object [" << next.path << "] domain=" << next.domain);

				ptr_resolve_internal(&source, false, &info);
				if (!source.obj)
				{
					APP_ERROR("Could not resolve ptr! [" << next.path << "] from domain " << next.domain);
					continue;
				}

				build_info_internal bii;
				bii.d = d;
				bii.ptr_ctx = &pctx;

				build_info bi;
				bi.path = path;
				bi.build_config = d->conf.build_config;
				bi.type = info.th;
				bi.object = source.obj;
				bi.record = build_db::create_record(path, info.signature.c_str(), builder_name(d, info.th->id()).c_str());;
				bi.internal = &bii;

				pcd.visited.clear();

				bool has_error = false;
				std::pair<HandlerMapT::iterator, HandlerMapT::iterator> hs = d->handlers.equal_range(info.th->id());
				for (HandlerMapT::iterator i = hs.first; i != hs.second; i++)
				{
					bi.builder = i->second.name;
					bi.user_data = i->second.user_data;
					RECORD_INFO(bi.record, "Invoking builder " << bi.builder << "...");
					if (!i->second.fn(&bi))
					{
						RECORD_ERROR(bi.record, "Error occured when building with builder " << bi.builder);
						has_error = true;
						break;
					}
				}
				
				if (hs.first == hs.second)
				{
					RECORD_INFO(bi.record, "Copying to output");
				}
				
				std::set<const char*> ignore;
				for (size_t i = 0; i != bii.outputs.size(); i++)
				{
					signature::buffer sigbuf;
					const char* sig = signature::object(bii.outputs[i].th, bii.outputs[i].obj, sigbuf);
					objstore::store_object(d->conf.temp, bii.outputs[i].path, bii.outputs[i].th, bii.outputs[i].obj, sig);
					build_db::add_output(bi.record, bii.outputs[i].path, bname.c_str(), sig);

					loaded le;
					le.th = bii.outputs[i].th;
					le.obj = bii.outputs[i].obj;
					le.signature = sig;
					d->loaded_temp.insert(std::make_pair(std::string(bii.outputs[i].path), le));
					ignore.insert(bii.outputs[i].path);
				}

				std::set<const char*>::iterator deps = pcd.visited.begin();
				while (deps != pcd.visited.end())
				{
					if (ignore.find(*deps) != ignore.end())
					{
						deps++;
						continue;
					}
					LoadedT::iterator i = d->loaded_temp.find(*deps);
					if (i != d->loaded_temp.end())
					{
						build_db::add_input_dependency(bi.record, *deps, i->second.signature.c_str());
					}
					else
					{
						i = d->loaded_input.find(*deps);
						if (i != d->loaded_input.end())
						{
							build_db::add_input_dependency(bi.record, *deps, i->second.signature.c_str());
						}
						else
						{
							APP_ERROR("Visited set contained entry " << *deps << " not in either input or temp!");
						}
					}
					deps++;
				}

				ptr_query_result ptrs;
				source.th->query_pointers(source.obj, &ptrs, true, true);

				// Clear pointers to non-existant runtime dependencies. Note this happens before signature computation step.
				for (size_t i = 0; i < ptrs.pointers.size();i++)
				{
					ptr_raw* p = ptrs.pointers[i];

					if (p->path != 0 && p->path[0])
					{
						objstore::object_info res;
						if (objstore::query_object(d->conf.input, p->path, &res) || objstore::query_object(d->conf.temp, p->path, &res))
						{
							continue;
						}
						APP_DEBUG("Clearing pointer to [" << p->path << "] because object could not be found. Will be null pointer.")
						p->path = 0;
						p->obj = 0;
					}
				}

				signature::buffer sigbuf;
				const char* sig = signature::object(source.th, source.obj, sigbuf);
			
				build_db::flush_log(bi.record);
				build_db::insert_metadata(bi.record, source.th, source.obj, source.path, sig);
				build_db::commit_record(d->conf.build_db, bi.record);

				// Add runtime dependencies.
				for (size_t i = 0; i < ptrs.pointers.size();i++)
				{
					ptr_raw* p = ptrs.pointers[i];
					if (p->path != 0 && p->path[0])
					{
						if (!d->has_added.count(p->path))
						{
							to_build_entry tb;
							tb.path = p->path;
							tb.domain = (int)p->user_data;
							d->to_build.push(tb);
							d->has_added.insert(p->path);
						}
					}
				}

				objstore::store_object(d->conf.built, path, info.th, source.obj, sig);

				source.th->free(source.obj);
			}
		}
	}
}
