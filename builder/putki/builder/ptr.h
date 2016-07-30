#pragma once

#include <putki/runtime.h>
#include <stdint.h>
#include <string.h>

namespace putki
{
	typedef void* instance_t;
	struct type_handler_i;

	typedef const char *type_t;
	struct ptr_raw;

	// Should always write obj. Should write th when mem != 0
	typedef void (*ptr_resolve_fn)(ptr_raw* ptr);
	typedef void (*ptr_on_deref_fn)(const ptr_raw* ptr);

	struct ptr_context
	{
		ptr_resolve_fn resolve;
		ptr_on_deref_fn deref;
		intptr_t user_data;
	};

	struct ptr_raw
	{
		const char* path;
		intptr_t user_data;
		instance_t obj;
		type_handler_i* th;
		ptr_context* ctx;
		bool has_resolved;
	};

	inline instance_t ptr_get(ptr_raw* ptr)
	{
		ptr_context* ctx = ptr->ctx;
		if (!ptr->has_resolved)
		{
            if (!ctx)
            {
                // dumb user created pointer.
                return 0;
            }
			ctx->resolve(ptr);
			ptr->has_resolved = true;
		}
		if (ctx->deref)
		{
			ctx->deref(ptr);
		}
		return ptr->obj;
	}

	template<typename InkiT>
	struct ptr
	{
		ptr_raw _ptr;
		void init(type_handler_i* th, const char* path)
		{
			_ptr.th = th;
			_ptr.path = path;
			_ptr.user_data = 0;
			_ptr.has_resolved = false;
			_ptr.obj = 0;
			_ptr.ctx = 0;
		}
		void set_context(ptr_context* ctx) { _ptr.ctx = ctx; }
		void set_path(const char* path) { _ptr.path = path; _ptr.has_resolved = false; }
		const char* path() { return _ptr.path;  }
		intptr_t& user_data() { return _ptr.user_data; };
		InkiT* get() const
		{
			return (InkiT*) ptr_get(const_cast<ptr_raw*>(&_ptr)); 
		}
		InkiT operator*() const { return *get(); }
		InkiT* operator->() const { return get(); }
		operator bool const() { return get() != 0; }
		template<typename SourceT>
		ptr(const ptr<SourceT>& source)
		{
			*this = source;
		}
		ptr()
		{
			memset(&_ptr, 0x00, sizeof(_ptr));
		}
		template<typename SourceT>
		ptr<InkiT>& operator=(const ptr<SourceT>& source)
		{
			_ptr.obj = static_cast<SourceT*>(source._ptr.obj);
			_ptr.th = source._ptr.th;
			_ptr.path = source._ptr.path;
			_ptr.user_data = source._ptr.user_data;
			_ptr.ctx = source._ptr.ctx;
			_ptr.has_resolved = source._ptr.has_resolved;
			return *this;
		}
	};
}

