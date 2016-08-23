#pragma once

#include <putki/runtime.h>
#include <vector>
#include <set>
#include <string>

#include "ptr.h"

namespace putki
{
	typedef void* instance_t;
	typedef const char *type_t;
	namespace parse { struct node; }

	struct sstream;

	struct ptr_query_result
	{
		std::vector<ptr_raw*> pointers;
	};

	inline void ptr_add_to_query_result(ptr_query_result* result, ptr_raw* p)
	{
		result->pointers.push_back(p);
	}

	struct file_query_result
	{
		std::vector<std::string*> files;
	};

	inline void add_to_file_query(file_query_result* result, std::string* file)
	{
		// Need to think what to do about this.
		result->files.push_back(file);
	}

	struct type_handler_i
	{
		// info
		virtual const char *name() = 0;
		virtual type_handler_i* parent_type() = 0;

		virtual int id() = 0; // unique type id
		virtual bool in_output() = 0;

		// instantiate / destruct.
		virtual instance_t alloc() = 0;
		virtual instance_t clone(instance_t source) = 0;
		virtual void free(instance_t) = 0;

		// reading / writing
		virtual void fill_from_parsed(parse::node *pn, instance_t target) = 0;
		virtual void write_json(instance_t source, putki::sstream & out, int indent) = 0;
		virtual void query_pointers(instance_t source, ptr_query_result* result, bool skip_input_only, bool rtti_dispatch) = 0;

		// TODO: Decide what to do about this some day.
		virtual void query_files(putki::instance_t source, putki::file_query_result* result, bool skip_input_only, bool rtti_dispatch) = 0;
		virtual char* write_into_buffer(runtime::descptr rt, instance_t source, char *beg, char *end) = 0;
	};

	void typereg_init();
	void typereg_register(type_t, type_handler_i *dt);
	type_handler_i *typereg_get_handler(type_t);
	type_handler_i *typereg_get_handler(int type_id);
	type_handler_i *typereg_get_handler_by_index(unsigned int idx);
}
