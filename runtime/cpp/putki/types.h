#pragma once

#include <stdint.h>

namespace putki
{
	typedef void* instance_t;

	// Dependency walking
	struct depwalker_i
	{
		bool pointer_pre_filter(instance_t *ptr)
		{
			// TODO: build in cycle dodging here
			return pointer_pre(ptr);
		}
		virtual bool pointer_pre(instance_t *ptr) = 0;
		virtual void pointer_post(instance_t *ptr) = 0;
	};

	typedef char* (*post_blob_load_fn)(void* data, char* beg, char* end);
	typedef void (*walk_dependencies_fn)(void* data, depwalker_i* walker);
		
	struct type_record
	{
		int id;
		unsigned int size;
		post_blob_load_fn post_blob_load;
		walk_dependencies_fn walk_dependencies;
	};
	
	void insert_type_records(const type_record* begin, const type_record* end);
	const type_record* get_type_record(int type);
}


