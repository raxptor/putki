#ifndef __PUTKI_BUILDER2_OBJSTORE_H__
#define __PUTKI_BUILDER2_OBJSTORE_H__

#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>
#include <string>

// Knows how to get parse nodes from path through some structure.
// This implementation only reads from .json files on disk.

namespace putki
{
	namespace objstore
	{
		struct data;
		data* open(const char *root_path, bool is_cache);
		void free(data *d);

		struct fetch_obj_result
		{
			instance_t obj;
			type_handler_i* th;
		};

		struct fetch_res_result
		{
			const char* data;
			size_t size;
		};

		struct object_info
		{
			std::string signature;
			type_handler_i* th;
		};

		struct resource_info
		{
			std::string path;
			std::string signature;
			size_t size;
			void* handle;
		};

		enum fetch_mode
		{
			FETCH_READONLY,
			FETCH_WRITABLE
		};

		bool fetch_resource(data* d, const char* path, const char* signature, fetch_res_result* result);
		void fetch_resource_free(fetch_res_result* result);

		bool query_object(data* d, const char *path, object_info* result);

		bool query_resource(data* d, const char* path, resource_info* result);
		size_t read_resource_range(data *d, const char* path, const char* signature, char* output, size_t beg, size_t end);

		bool fetch_object(data* d, const char* path, fetch_obj_result* result);

		bool store_object(data* d, const char *path, type_handler_i* th, instance_t obj, const char *signature);
		bool store_resource(data* d, const char *path, const char* data, size_t length);

		bool uncache_object(data* dest, data* source, const char *path, const char *signature);
		bool uncache_resource(data* dest, data* source, const char *path, const char *signature);

		size_t query_by_type(data* d, type_handler_i* th, const char** paths, size_t len);
	}
}

#endif
