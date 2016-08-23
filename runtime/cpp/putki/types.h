#pragma once

#include <stdint.h>

namespace putki
{
	typedef void* instance_t;

	struct resource_id
	{
		uintptr_t slot;
	};

	struct ptr_info;	

	typedef void(*objwalker_callback_ptr)(ptr_info* info, void* user_data);
	typedef void(*objwalker_callback_res)(resource_id* res, void* user_data);
	typedef void(*objwalker_walker)(void* object, objwalker_callback_ptr callback_ptr, objwalker_callback_res callback_file, void* user_data);

	struct ptr_info
	{
		instance_t* ptr;
		objwalker_walker walker;
	};
	
	typedef char* (*post_blob_load_fn)(void* data, char* beg, char* end);

	struct type_record
	{
		int id;
		unsigned int size;
		post_blob_load_fn post_blob_load;
		objwalker_walker walk_dependencies;
	};

	struct field_record
	{
		unsigned int type;
		unsigned int offset;
	};
	
	void insert_type_records(const type_record* begin, const type_record* end);
	const type_record* get_type_record(int type);
}
