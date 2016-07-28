#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>
#include <putki/builder/db.h>
#include <putki/builder/build-db.h>
#include "builder2.h"
#include "objloader.h"

#include <map>
#include <set>
#include <queue>

namespace putki
{
	namespace builder2
	{
		struct build_info_internal
		{
			db::data* tmp_db;
			db::data* output_db;
			std::vector<std::string> outputs;
		};

		typedef std::multimap<int, handler_info> HandlerMapT;

		struct data
		{
			config conf;
			db::data* output;
			objloader::data* input_loader;
			objloader::data* temp_loader;
			HandlerMapT handlers;
			std::queue<std::string> to_build;
			std::set<std::string> has_added;
		};

		static configurator_fn s_config_fn = 0;

		void set_builder_configurator(configurator_fn configurator)
		{
			s_config_fn = configurator;
		}

		data* create(config* conf)
		{
			data* d = new data();
			d->conf = *conf;
			d->input_loader = objloader::create(conf->input);
			d->temp_loader = objloader::create(conf->temp);
			d->output = db::create();
			s_config_fn(d);
			return d;
		}

		void free(data *d)
		{
			objloader::free(d->input_loader);
			objloader::free(d->temp_loader);
			db::free(d->output);
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
			}
			return builder_name.empty() ? "default" : builder_name;
		}

		void add_build_output(const build_info* info, type_handler_i* th, instance_t object, const char *path)
		{
			info->internal->outputs.push_back(path);
			db::insert(info->internal->tmp_db, path, th, object);
		}

		struct find_runtime_deps : public depwalker_i
		{
			db::data* db;
			std::set<std::string> ptrs;
			build_db::record* record;

			bool pointer_pre(instance_t *ptr, const char *ptr_type_name)
			{
				if (!*ptr)
				{
					return false;
				}

				const char *path = db::pathof_including_unresolved(db, *ptr);
				if (!path)
				{
					RECORD_ERROR(record, "Post-build there was an unrecognized pointer!");
					return false;
				}

				ptrs.insert(path);
				RECORD_DEBUG(record, "dep:" << path);
				return false;
			}
		};

		void add_build_root(data *d, const char *path)
		{
			if (!d->has_added.count(path))
			{
				d->has_added.insert(path);
				d->to_build.push(path);
			}
		}

		bool fetch_cached(data* d, const char* path, objstore::object_info* info, const char* bname, db::data* input, db::data* temp, db::data* output)
		{
			build_db::record* find = build_db::find(d->conf.build_db, path, info->signature.c_str(), bname);
			if (!find)
			{
				return false;
			}

			int o = 0;
			while (true)
			{
				const char* out = build_db::enum_outputs(find, o);
				if (!out)
				{
					break;
				}
				const char* out_sig = get_output_signature(find, o);
				if (!objstore::transfer_and_uncache_into(d->conf.temp, d->conf.temp, out, out_sig))
				{
					// Is cleanup here actually needed? Probably not.
					APP_DEBUG("Could not uncache object " << out << " sig=" << out_sig);
					return false;
				}
				else
				{
					APP_DEBUG("Uncached tmp object " << out << " sig=" << build_db::get_signature(find));
				}
				++o;
			}

			if (!objstore::transfer_and_uncache_into(d->conf.built, d->conf.built, path, build_db::get_signature(find)))
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
					d->to_build.push(ptr);
					d->has_added.insert(ptr);
				}
			}
			return true;
		}

		void do_build(data *d, bool incremental)
		{
			db::data* input = db::create();
			db::data* temp = db::create(input);
			db::data* output = db::create(temp);

			while (true)
			{
				if (d->to_build.empty())
				{
					break;
				}

				std::string next = d->to_build.front();
				d->to_build.pop();

				const char* path = next.c_str();

				bool is_tmp_obj = false;
				objstore::object_info info;
				if (!objstore::query_object(d->conf.input, path, &info))
				{
					if (objstore::query_object(d->conf.temp, path, &info))
					{
						is_tmp_obj = true;
					}
					else
					{
						APP_ERROR("Attempted to build object not in store! [" << path << "]");
						continue;
					}
				}

				std::string bname = builder_name(d, info.th->id());
				if (incremental && fetch_cached(d, path, &info, bname.c_str(), input, temp, output))
				{
					continue;
				}

				if (is_tmp_obj)
				{
					if (!objloader::load_into(d->temp_loader, input, path))
					{
						APP_ERROR("Could not load tmp object to build into db! [" << path << "]");
						continue;
					}
				}
				else
				{
					if (!objloader::load_into(d->input_loader, input, path))
					{
						APP_ERROR("Could not load object to build into db! [" << path << "]");
						continue;
					}
				}

				type_handler_i* th;
				instance_t obj;
				if (!db::fetch(input, path, &th, &obj))
				{
					APP_ERROR("Fetch error! This should never happen.");
					continue;
				}

				instance_t clone = th->clone(obj);

				build_info_internal bii;
				bii.tmp_db = temp;

				build_info bi;
				bi.path = path;
				bi.build_config = d->conf.build_config;
				bi.type = th;
				bi.object = clone;
				bi.record = build_db::create_record(path, info.signature.c_str(), builder_name(d, info.th->id()).c_str());;
				bi.internal = &bii;

				bool has_error = false;
				std::pair<HandlerMapT::iterator, HandlerMapT::iterator> hs = d->handlers.equal_range(th->id());
				for (HandlerMapT::iterator i = hs.first; i != hs.second; i++)
				{
					bi.builder = i->second.name;
					bi.user_data = i->second.user_data;
					if (!i->second.fn(&bi))
					{
						has_error = true;
						break;
					}
				}

				// TODO: Enumerate struct instances here too and build them.
				APP_DEBUG("Finished building [" << path << "] has_error = " << has_error);

				for (size_t i = 0; i != bii.outputs.size(); i++)
				{
					char buffer[64];
					build_db::add_output(bi.record, bii.outputs[i].c_str(), bname.c_str(), db::signature(temp, bii.outputs[i].c_str(), buffer));
				}

				db::insert(output, path, th, clone);

				// Find out run-time dependencies.
				find_runtime_deps frd;
				frd.db = temp;
				frd.record = bi.record;
				th->walk_dependencies(clone, &frd, false);
				build_db::add_input_dependency(bi.record, path, info.signature.c_str());
				build_db::flush_log(frd.record);
				build_db::insert_metadata(frd.record, output, path);
				build_db::commit_record(d->conf.build_db, bi.record);

				// Handle outputs
				int o = 0;
				while (true)
				{
					const char* out = build_db::enum_outputs(bi.record, o++);
					if (!out)
					{
						break;
					}

					type_handler_i* th;
					instance_t obj;
					if (db::fetch(temp, out, &th, &obj))
					{
						char buffer[64];
						objstore::store_object(d->conf.temp, out, temp, th, obj, db::signature(temp, out, buffer));
					}
					else
					{
						APP_ERROR("enum_outputs gave path [" << out << "] but it did not exist in temp database!");
					}
				}

				// Add runtime dependencies.
				for (std::set<std::string>::iterator i = frd.ptrs.begin(); i != frd.ptrs.end(); i++)
				{
					if (!d->has_added.count(*i))
					{
						d->to_build.push(*i);
						d->has_added.insert(*i);
					}
				}

				char slask[64];
				APP_DEBUG("Source signature = " + info.signature + "  dest signature = " + db::signature(output, path, slask) + " org sig = ");
				
				// store_object takes ownership of objects
				char buffer[64];
				objstore::store_object(d->conf.built, path, output, th, clone, db::signature(output, path, buffer));
			}

			db::free_and_destroy_objs(input);
			db::free(temp);
			db::free(output);
		}
	}
}
