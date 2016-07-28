#pragma once

#include <stdint.h>

namespace putki
{
	typedef void* instance_t;

	struct ptr_info;
	typedef void(*ptrwalker_callback)(ptr_info* info, void* user_data);
	typedef void(*ptrwalker_walker)(void* object, ptrwalker_callback callback, void* user_data);

	struct ptr_info
	{
		instance_t* ptr;
		ptrwalker_walker walker;
	};

	struct depwalker_i
	{
		virtual void pointer(instance_t *ptr) = 0;
	};

	typedef char* (*post_blob_load_fn)(void* data, char* beg, char* end);

	struct type_record
	{
		int id;
		unsigned int size;
		post_blob_load_fn post_blob_load;
		ptrwalker_walker walk_dependencies;
	};


	struct field_record
	{
		unsigned int type;
		unsigned int offset;
	};
	
	void insert_type_records(const type_record* begin, const type_record* end);
	const type_record* get_type_record(int type);
}
