#pragma once

#include <putki/runtime.h>
#include <putki/builder/typereg.h>
#include <putki/builder/objstore.h>

namespace putki
{
	namespace package { struct data; }
	namespace db { struct data; }
	namespace builder { struct data; }

	namespace build
	{
		struct packaging_config;
		void make_packages(runtime::descptr rt, const char* build_config, bool incremental, bool make_patch);
		void commit_package(putki::package::data *package, packaging_config *packaging, const char *out_path);

		typedef void(*builder_setup_fn)(builder::data *builder);
		typedef void(*packaging_fn)(objstore::data *out, build::packaging_config *pconf);
        
        struct postbuild_info
        {
            objstore::data* input;
            objstore::data* temp;
            objstore::data* output;
            builder::data* builder;
            build::packaging_config* pconf;
        };
        
		typedef void(*postbuild_fn)(postbuild_info* info);

		void set_builder_configurator(builder_setup_fn fn);
		void set_packager(packaging_fn fn);
		void add_postbuild_fn(postbuild_fn fn);
	}
}
