#include <putki/builder/build.h>
#include <putki/builder/builder.h>
#include <putki/builder/package.h>
#include <putki/builder/log.h>

#include "putki/builder2/objstore.h"
#include "putki/builder2/objloader.h"
#include "putki/builder2/builder2.h"

namespace inki
{
	void bind_test_proj();
}

using namespace putki;

int main(int argc, char **argv)
{
	inki::bind_test_proj();

	set_loglevel(putki::LOG_DEBUG);
	objstore::data *input_store = objstore::open("data/");
	objstore::data *tmp_store = objstore::open("out/.tmp");
	objstore::data *built_store = objstore::open("out/.built");

	build_db::data* bdb = build_db::create(".builddb", false);

	builder2::config conf;
	conf.input = input_store;
	conf.temp = tmp_store;
	conf.built = built_store;
	conf.build_db = bdb;
	builder2::data* builder = builder2::create(&conf);
	
	builder2::add_build_root(builder, "triply-nested");
	builder2::do_build(builder);

	builder2::free(builder);
	build_db::release(bdb);
	objstore::free(input_store);
	objstore::free(tmp_store);
	objstore::free(built_store);
}
