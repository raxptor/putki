#include "objloader.h"

#include <putki/sys/files.h>
#include <putki/builder/db.h>
#include <putki/builder/log.h>
#include <putki/builder/parse.h>
#include <putki/builder/typereg.h>

#include <string>
#include <map>
#include <vector>
#include <set>

namespace putki
{
	namespace objloader
	{
		struct data
		{
			objstore::data* store;
		};

		data* create(objstore::data* store)
		{
			data* d = new data();
			d->store = store;
			return d;
		}
		
		void free(data *d)
		{
			delete d;
		}

		namespace
		{
			struct loaded
			{
				type_handler_i* th;
				instance_t obj;
			};

			typedef std::map<std::string, loaded> LoadedMapT;

			struct resolve : public load_resolver_i
			{
				db::data* tmp_db;
				db::data* out_db;
				std::set<std::string> paths;
				std::string self_path;
				void resolve_pointer(instance_t *ptr, const char *path)
				{
					std::string actual = path;
					if (actual.size() > 0 && actual[0] == '#')
					{
						actual = self_path + actual;
					}
					putki::type_handler_i* th;
					if (!db::fetch(out_db, actual.c_str(), &th, ptr))
					{
						paths.insert(actual.c_str());
						*ptr = db::create_unresolved_pointer(tmp_db, actual.c_str());
						APP_WARNING("ptr[" << (*ptr) << "] = " << actual);
					}
				}
			};

			struct finalize : public depwalker_i
			{
				db::data* tmp_db;
				db::data* db;
				LoadedMapT* loaded;
				std::string self_path;
				
				bool pointer_pre(instance_t *ptr, const char *ptr_type_name)
				{
					if (!*ptr)
					{
						return true;
					}
					const char *path = db::is_unresolved_pointer(tmp_db, *ptr);
					if (!path)
					{
						APP_WARNING("ptr[" << ptr << "] already resolved?!?!");
						return true;
					}
					putki::type_handler_i* th;
					if (!db::fetch(db, path, &th, ptr))
					{
						LoadedMapT::iterator i = loaded->find(path);
						if (i == loaded->end())
						{
							*ptr = 0;
							APP_WARNING("Unresolved pointer to [" << path << "] in object [" << self_path << "], clearing pointer.");
						}
						else
						{
							APP_WARNING("Resolving co-loaded pointer to [" << path << "] in object [" << self_path << "].");
							*ptr = i->second.obj;
						}
					}
					return true;
				}
			};
		}

		struct load_data
		{
			data* d;
			db::data* tmp_db;
			db::data* out_db;
			LoadedMapT loaded;
		};

		bool process_load(load_data* ld, const char* path, type_handler_i** _th, instance_t* _obj, bool load_deps)
		{
			APP_DEBUG("Process load [" << path << "]");
			if (db::fetch(ld->out_db, path, _th, _obj))
			{
				return true;
			}

			LoadedMapT::iterator qi = ld->loaded.find(path);
			if (qi != ld->loaded.end())
			{
				return false;
			}

			objstore::object_info info;
			if (!objstore::query_object(ld->d->store, path, &info))
			{
				return false;
			}
			
			APP_DEBUG("\tprocess_load/fetch [" << path << "] sig = [" << info.signature << "]");
			objstore::fetch_obj_result fr;
			if (!objstore::fetch_object(ld->d->store, path, info.signature.c_str(), &fr))
			{
				return false;
			}

			APP_DEBUG("\tprocess_load/alloc&fill [" << path << "]");
			char self[1024];
			if (!db::base_asset_path(path, self, 1024))
			{
				strcpy(self, path);
			}

			resolve res;
			res.self_path = self;
			res.tmp_db = ld->tmp_db;
			res.out_db = ld->out_db;
			instance_t obj = fr.th->alloc();
			fr.th->fill_from_parsed(fr.node, obj, &res);

			loaded l;
			l.obj = obj;
			l.th = fr.th;
			ld->loaded.insert(std::make_pair(std::string(path), l));

			if (load_deps)
			{
				APP_DEBUG("\tprocess_load/load_deps(" << res.paths.size() << ") [" << path << "]");
				std::set<std::string>::iterator i = res.paths.begin();
				while (i != res.paths.end())
				{
					process_load(ld, i->c_str(), 0, 0, true);
					i++;
				}
			}

			if (_th && _obj)
			{
				*_th = fr.th;
				*_obj = obj;
			}
			
			objstore::fetch_object_free(&fr);
			return true;
		}

		bool load_into_nodeps(data* d, db::data* db, const char* path)
		{
			if (db::exists(db, path))
			{
				return true;
			}

			load_data ld;
			ld.d = d;
			ld.tmp_db = db;
			ld.out_db = db;

			type_handler_i* th;
			instance_t obj;
			if (!process_load(&ld, path, &th, &obj, false))
			{
				APP_WARNING("Failed to load object[" << path << "]!");
				return false;
			}

			db::insert(db, path, th, obj);
			return true;
		}

		bool load_into(data* d, db::data* db, const char* path)
		{
			if (db::exists(db, path))
			{
				return true;
			}

			load_data ld;
			ld.d = d;
			ld.tmp_db = db::create();
			ld.out_db = db;

			type_handler_i* th;
			instance_t obj;
			if (!process_load(&ld, path, &th, &obj, true))
			{
				APP_WARNING("Failed to load object[" << path << "]!");
				return false;
			}

			finalize fn;
			fn.db = db;
			fn.tmp_db = ld.tmp_db;
			fn.self_path = path;
			fn.loaded = &ld.loaded;
			th->walk_dependencies(obj, &fn, true);

			LoadedMapT::iterator qi = ld.loaded.begin();
			while (qi != ld.loaded.end())
			{
				db::insert(db, qi->first.c_str(), qi->second.th, qi->second.obj);
				++qi;
			}
			
			db::free(ld.tmp_db);

			return true;
		}
	}
}

