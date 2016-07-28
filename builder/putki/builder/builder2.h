#ifndef __PUTKI_BUILDER2_H__
#define __PUTKI_BUILDER2_H__

#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>
#include <putki/builder/db.h>
#include <putki/builder/build-db.h>
#include "objstore.h"

namespace putki
{
	namespace builder2
	{
		struct data;

		struct build_info_internal;
		struct build_info
		{
			// instance being built.
			const char* path;
			const char* build_config;
			const char* builder;
			type_handler_i* type;
			instance_t object;

			// database
			build_db::record* record;

			// builder data
			void* user_data;

			// internal
			build_info_internal* internal;
		};

		typedef bool (*obj_handler_fn)(const build_info* info);
		typedef void(*configurator_fn)(data* target);

		struct handler_info
		{
			int type_id;
			const char* name;
			obj_handler_fn fn;
			void* user_data;
		};

		//
		struct config
		{
			build_db::data* build_db;
			objstore::data* input;
			objstore::data* temp;
			objstore::data* built;
			const char* build_config;
		};

		// Global configuration
		void set_builder_configurator(configurator_fn configurator);

		// Automatically creates and adds the object.
		template<typename T>
		T* create_build_output(const build_info* info, const char *path)
		{
			instance_t obj = T::th()->alloc();
			add_build_output(info, T::th(), obj, path);
			return (T*) obj;
		}

		void add_build_output(const build_info* info, type_handler_i* th, instance_t object, const char *path);

		// 
		data* create(config* conf);
		void free(data *d);

		//
		void add_handlers(data* d, const handler_info* begin, const handler_info* end);

		void add_build_root(data *d, const char *path);
		void do_build(data *d, bool incremental);
	}
}

#endif
