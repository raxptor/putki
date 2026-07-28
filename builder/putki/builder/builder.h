#ifndef __PUTKI_BUILDER_H__
#define __PUTKI_BUILDER_H__

#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>
#include <putki/builder/build-db.h>
#include <putki/builder/build.h>
#include "objstore.h"

namespace putki
{
	namespace builder
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

		// Automatically creates and adds the object.
		template<typename T>
		ptr<T> create_build_output(const build_info* info, const char *path)
		{
			ptr<T> new_ptr;
			create_build_output(info, T::th(), path, &new_ptr._ptr);
			return new_ptr;
		}

		struct resource
		{
			const char* signature;
			const char* data;
			size_t size;
			objstore::fetch_res_result internal;
		};

		// Automatically adds to input dependency.
		bool fetch_resource(const build_info* info, const char* path, resource* resource);
		void free_resource(resource* resource);
		size_t read_resource_segment(const build_info* info, const char* path, char* output, size_t beg, size_t end);

		// Creates path based on input name and 'tag'. Returns path.
		std::string store_resource_tag(const build_info* info, const char* tag, const char* data, size_t size);

		// Stores by path specified in 'path'
		bool store_resource_path(const build_info* info, const char* path, const char* data, size_t size);

		// Creates path based on input name and 'tag'
		void create_build_output(const build_info* info, type_handler_i* th, const char *tag, ptr_raw* ptr);

		void add_post_build_object(data* d, type_handler_i* th, instance_t obj, const char* path);

		//
		data* create(config* conf);
		void clear(data *d);
		void free(data *d);

		void add_handlers(data* d, const handler_info* begin, const handler_info* end);

		// domain = 0 means input. domain = 1 is temp objects.
		void add_build_root(data *d, const char *path, int domain=0);
		void do_build(data *d, bool incremental);
	}
}

#endif
