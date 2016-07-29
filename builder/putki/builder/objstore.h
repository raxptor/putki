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
		data* open(const char *root_path);
		void free(data *d);

		struct fetch_obj_result
		{
			parse::node* node;
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
			std::string signature;
		};

		bool query_resource(data* d, const char *path, resource_info* result);
		bool fetch_resource(data* d, const char* path, const char* signature, fetch_res_result* result);
		void fetch_resource_free(fetch_res_result* result);
		
		bool query_object(data* d, const char *path, object_info* result);
		bool fetch_object(data* d, const char* path, const char* signature, fetch_obj_result* result);
		void fetch_object_free(fetch_obj_result* result);

		bool store_object(data* d, const char *path, type_handler_i* th, instance_t obj, const char *signature);
		bool store_resource(data* d, const char *path, const char* data, size_t length);
		
		bool uncache_object(data* dest, data* source, const char *path, const char *signature);
		bool uncache_resource(data* dest, data* source, const char *path, const char *signature);
	}
}

#endif
