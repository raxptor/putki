#ifndef __PUTKI_BUILDER2_OBJSTORE_H__
#define __PUTKI_BUILDER2_OBJSTORE_H__

#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>

// Knows how to get parse nodes from path through some structure.
// This implementation only reads from .json files on disk.

namespace putki
{
	namespace objstore
	{
		struct data;
		data* open(const char *root_path);
		void free(data *d);

		struct fetch_result
		{
			parse::node* node;
			type_handler_i* th;
		};

		struct object_info
		{
			std::string signature;
			type_handler_i* th;
		};

		bool fetch_object(data* d, const char* path, const char* signature, fetch_result* result);
		void fetch_object_free(data* d, fetch_result* result);
		bool query_object(data* d, const char *path, object_info* result);

		// This function will destroy the object.
		bool store_object(data* d, const char *path, db::data* ref_source, type_handler_i* th, instance_t obj, const char *signature);
		bool transfer_and_uncache_into(data* dest, data* source, const char *path, const char *signature);
	}
}

#endif
