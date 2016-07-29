#pragma once

#include <putki/runtime.h>
#include <vector>

#include "ptr.h"

namespace putki
{
	typedef void* instance_t;
	typedef const char *type_t;

	struct i_field_desc
	{
		const char *name;
	};

	namespace parse { struct node; }
	namespace db { struct data; }

	struct sstream;

	struct depwalker_i
	{
		// dodge cycles
		depwalker_i();
		virtual ~depwalker_i();
		struct visited_set;
		visited_set *_visited;
		bool pointer_pre_filter(instance_t *on, const char *ptr_type_name);

		void reset_visited();

		// pre descending into pointer.
		virtual bool pointer_pre(instance_t *on, const char *ptr_type_name) = 0;
	
		// after visited it.
		virtual void pointer_post(instance_t *on) { }; // post descending into pointer.
	};

	struct ptr_query_result
	{
		std::vector<ptr_raw*> pointers;
	};

	inline void ptr_add_to_query_result(ptr_query_result* result, ptr_raw* p)
	{
		result->pointers.push_back(p);
	}

	struct type_handler_i
	{
		// info
		virtual const char *name() = 0;
		virtual type_handler_i *parent_type() = 0;
		
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

		virtual char* write_into_buffer(runtime::descptr rt, instance_t source, char *beg, char *end) = 0;

		// recurse down and report all pointers
		virtual void walk_dependencies(instance_t source, depwalker_i *walker, bool traverseChildren, bool skipInputOnly = false, bool rttiDispatch = false) { }
	};

	void typereg_init();
	void typereg_register(type_t, type_handler_i *dt);
	type_handler_i *typereg_get_handler(type_t);
	type_handler_i *typereg_get_handler(int type_id);
	type_handler_i *typereg_get_handler_by_index(unsigned int idx);
}
