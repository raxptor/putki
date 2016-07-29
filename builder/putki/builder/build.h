#pragma once

#include <putki/runtime.h>
#include <putki/builder/typereg.h>
#include <putki/builder/objstore.h>

namespace putki
{
	namespace package { struct data; }
	namespace db { struct data; }
	namespace builder2 { struct data; }

	namespace build
	{
		struct packaging_config;
		void make_packages(runtime::descptr rt, const char* build_config, bool incremental, bool make_patch);
		void commit_package(putki::package::data *package, packaging_config *packaging, const char *out_path);

		typedef void(*builder_setup_fn)(builder2::data *builder);
		typedef void(*packaging_fn)(objstore::data *out, build::packaging_config *pconf);
		typedef void(*reporting_fn)(objstore::data *out, build::packaging_config *pconf);

		void set_builder_configurator(builder_setup_fn fn);
		void set_packager(packaging_fn fn);
		void set_reporting_fn(reporting_fn fn);

		void invoke_packager(objstore::data* built, build::packaging_config *pconf);
		void invoke_reporting(objstore::data* out, build::packaging_config *pconf);
	}
}
