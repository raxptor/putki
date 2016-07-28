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
			std::set<std::string> has_built;
		};

		struct data;
		data* create(config* conf);

		data* create(config* conf)
		{
			data* d = new data();
			d->conf = *conf;
			d->input_loader = objloader::create(conf->input);
			d->temp_loader = objloader::create(conf->temp);
			d->output = db::create();
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
			for (const handler_info* i=begin;i!=end;i++)
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

		void add_build_output(build_info* info, type_handler_i* th, instance_t object, const char *path)
		{
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
			d->to_build.push(path);
		}

		void do_build(data *d)
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
				d->has_built.insert(next);

				const char* path = next.c_str();

				objstore::object_info info;
				if (!objstore::query_object(d->conf.input, path, &info) && !objstore::query_object(d->conf.temp, path, &info))
				{
					APP_ERROR("Attempted to build object not in store! [" << path << "]");
					continue;
				}

				if (!objloader::load_into(d->input_loader, input, path) && !objloader::load_into(d->temp_loader, input, path))
				{
					APP_ERROR("Could not load object to build into db! [" << path << "]");
					continue;
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
				bi.type = th;
				bi.object = clone;
				bi.record = build_db::create_record(path, info.signature.c_str(), builder_name(d, info.th->id()).c_str());;
				bi.internal = &bii;

				bool has_error = false;
				std::pair<HandlerMapT::iterator, HandlerMapT::iterator> hs = d->handlers.equal_range(th->id());
				for (HandlerMapT::iterator i = hs.first; i != hs.second; i++)
				{
					bi.user_data = i->second.user_data;
					if (!i->second.fn(&bi))
					{
						has_error = true;
						break;
					}
				}

				// TODO: Enumerate struct instances here too.

				APP_DEBUG("Finished building [" << path << "] has_error = " << has_error);

				// Find out run-time dependencies.
				find_runtime_deps frd;
				frd.db = temp;
				frd.record = bi.record;
				th->walk_dependencies(clone, &frd, false);
				build_db::flush_log(frd.record);
				build_db::commit_record(d->conf.build_db, bi.record);

				db::insert(output, path, th, clone);
				for (std::set<std::string>::iterator i = frd.ptrs.begin(); i != frd.ptrs.end(); i++)
				{
					if (!d->has_built.count(*i))
					{
						d->to_build.push(*i);
					}
				}

				char buffer[64];
				objstore::store_object(d->conf.built, path, output, th, clone, db::signature(output, path, buffer));
			}

			db::free(input);
			db::free(temp);
			db::free(output);
		}
	}
}
