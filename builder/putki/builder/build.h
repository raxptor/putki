#ifndef __putki_databuilder_lib__build__
#define __putki_databuilder_lib__build__

#include <putki/runtime.h>
#include <putki/builder/typereg.h>
#include <putki/builder/builder2.h>

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
	}
}

#endif
