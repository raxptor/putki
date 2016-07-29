#pragma once
#include <putki/runtime.h>

namespace putki
{
	typedef void* instance_t;
	typedef const char *type_t;

	struct ptr_context;

	struct ptr_raw
	{
		int type_id;
		const char* path;
		intptr_t user_data;
		bool has_resolved;
		instance_t* mem;
		ptr_context* ctx;
	};

	void ptr_mark_visited(ptr_raw* p);

	// queries
	struct ptr_query_result;
	void ptr_add_to_query_result(ptr_query_result* result, ptr_raw* p);

	// action.
	instance_t ptr_get(ptr_raw* p);

	template<typename InkiT>
	struct ptr
	{
		private:
			data_ptr _ptr;
		public:
			void set_path(const char* path);
			intptr_t& user_data() { return _ptr.user_data; };
			const InkiT* get() { return ptr_get(const_cast<ptr_raw*>(&_ptr)); }
			const InkiT& operator*() { return ptr_get(const_cast<ptr_raw*>(&_ptr)); }
			const InkiT* operator->() const { return ptr_get(const_cast<ptr_raw*>(&_ptr)); }
			const InkiT& bool { return ptr_get(const_cast<ptr_raw*>(&_ptr)) != 0; }
	};
}

