#include <putki/builder/build.h>
#include <putki/builder/builder.h>
#include <putki/builder/package.h>
#include <putki/builder/builder2.h>

#include <inki/types/t1.h>

// generated.
namespace inki
{
	void bind_test_proj();
}

bool everything_builder(const putki::builder2::build_info* info)
{
	inki::everything* obj = (inki::everything*) info->object;

	inki::built_asset* built = putki::builder2::create_build_output<inki::built_asset>(info, "slask");
	built->build_config = info->build_config;
	built->other_data = obj->vt_inline.text;
	obj->built = built;

	return true;
}

void app_configure_builder(putki::builder2::data *builder)
{
	const int count = 1;
	putki::builder2::handler_info h[1] = {
		{ inki::everything::type_id(), "everything-builder", everything_builder, 0 }
	};
	putki::builder2::add_handlers(builder, &h[0], &h[count]);
}

void app_build_packages(putki::objstore::data *out, putki::build::packaging_config *pconf)
{
	{
		putki::package::data *pkg = putki::package::create(out);
		putki::package::add(pkg, "everything", true);
		putki::package::add(pkg, "triply-nested", true);
		putki::build::commit_package(pkg, pconf, "default.pkg");
	}
}

int run_putki_builder(int argc, char **argv);

int main(int argc, char **argv)
{
	inki::bind_test_proj();
	putki::builder2::set_builder_configurator(&app_configure_builder);
	putki::builder::set_packager(&app_build_packages);
	return run_putki_builder(argc, argv);
}
