//
//  build.cpp
//  putki-databuilder-lib
//
//  Created by Dan Nilsson on 5/28/13.
//
//

#include "build.h"

#include <putki/builder/typereg.h>
#include <putki/builder/package.h>
#include <putki/builder/write.h>
#include <putki/builder/build-db.h>
#include <putki/builder/log.h>
#include <putki/builder/objstore.h>
#include <putki/builder/builder.h>

#include <putki/sys/files.h>
#include <putki/sys/thread.h>

#include <algorithm>
#include <iostream>
#include <fstream>
#include <string>
#include <sstream>
#include <vector>
#include <set>

namespace putki
{
	namespace
	{
		const size_t xbufSize = 64 * 1024 * 1024;
		char xbuf[xbufSize];
	}

	namespace build
	{
		struct pkg_conf
		{
			package::data *pkg;
			std::string path;
			std::string final_path;
			std::string final_manifest_path;
			std::string ptr_file;
			std::string ptr_file_content;
		};

		struct packaging_config
		{
			std::string package_path;
			runtime::descptr rt;
			build_db::data *bdb;
			std::vector<pkg_conf> packages;
			bool make_patch;
		};

		static builder_setup_fn s_config_fn = 0;
		static packaging_fn s_packaging_fn = 0;
		static std::vector<postbuild_fn> s_postbuild_fns;

		void set_builder_configurator(builder_setup_fn configurator)
		{
			s_config_fn = configurator;
		}

		void set_packager(packaging_fn packaging)
		{
			s_packaging_fn = packaging;
		}

		void add_postbuild_fn(postbuild_fn fn)
		{
			s_postbuild_fns.push_back(fn);
		}

		void invoke_packager(putki::objstore::data* out, packaging_config* pconf)
		{
			s_packaging_fn(out, pconf);
		}

		void invoke_post_build(postbuild_info* info)
		{
			for (size_t i=0;i<s_postbuild_fns.size();i++)
			{
				s_postbuild_fns[i](info);
			}
		}

		void commit_package(putki::package::data *package, packaging_config *packaging, const char *out_path)
		{
			pkg_conf pk;
			
			bool make_patch = packaging->make_patch;
		
			// expect old packages to be there.
			pk.ptr_file = packaging->package_path + out_path + ".ptr";
			pk.ptr_file_content = out_path;
			pk.final_path = packaging->package_path + out_path;
			pk.final_manifest_path = packaging->package_path + out_path + ".manifest";

			std::vector<std::string> old_ones;
			if (make_patch)
			{
				for (int i=0;;i++)
				{
					sstream pkg_name;
					pkg_name << out_path;
					if (i > 0)
					{
						pkg_name << ".patch" << i;
					}
	
					std::string manifest_name(pkg_name.c_str());
					manifest_name.append(".manifest");
					
					pk.final_path = packaging->package_path + pkg_name.c_str();
					pk.final_manifest_path = packaging->package_path + manifest_name.c_str();

					std::ifstream f0(pk.final_path.c_str());
					std::ifstream f1(pk.final_manifest_path.c_str());
					
					// check for previous files.
					if (f0.good() && f1.good())
					{
						APP_DEBUG("Previous at " << pkg_name.c_str() << " and " << manifest_name);
						old_ones.push_back(pkg_name.c_str());
					}
					else
					{
						pk.ptr_file_content = pkg_name.c_str();
						break;
					}
				}
				
				if (old_ones.empty())
				{
					APP_DEBUG("Wanted to make patch for " << out_path << " but previous package does not exist!");
					make_patch = false;
				}
			}
			
			if (make_patch)
			{
				for (int i=(int)old_ones.size()-1;i>=0;i--)
				{
					package::add_previous_package(package, packaging->package_path.c_str(), old_ones[i].c_str());
				}
				APP_DEBUG("Registered package " << out_path << " and it will be a patch [" << pk.final_path << "] with " << old_ones.size() << " previous files to use");
			}

			pk.pkg = package;
			pk.path = out_path;
			packaging->packages.push_back(pk);
			APP_DEBUG("Registered package " << out_path);
		}

		void write_package(pkg_conf *pk, packaging_config *packaging)
		{
			APP_DEBUG("Saving package to [" << pk->final_path << "]...")

			sstream mf;
			long bytes_written = putki::package::write(pk->pkg, packaging->rt, xbuf, xbufSize, packaging->bdb, mf);

			APP_INFO("Wrote " << pk->final_path << " (" << bytes_written << ") bytes")

			putki::sys::mk_dir_for_path(pk->final_path.c_str());
			putki::sys::mk_dir_for_path(pk->final_manifest_path.c_str());

			std::ofstream pkg(pk->final_path.c_str(), std::ios::binary);
			pkg.write(xbuf, bytes_written);
			pkg.close();
			
			std::ofstream pkg_mf(pk->final_manifest_path.c_str(), std::ios::binary);
			pkg_mf.write(mf.c_str(), mf.size());
			pkg_mf.close();
			
			std::ofstream ptr(pk->ptr_file.c_str());
			ptr << pk->ptr_file_content;
			ptr.close();
		}

		void make_packages(runtime::descptr rt, const char* build_config, bool incremental, bool make_patch)
		{	
			char pfx[1024];
			sprintf(pfx, "out/%s-%s", runtime::desc_str(rt), build_config);

			size_t len = strlen(pfx);
			for (int i=0;i<len;i++)
				pfx[i] = ::tolower(pfx[i]);

			std::string prefix(pfx);
			objstore::data *input_store = objstore::open("data/", false);
			objstore::data *tmp_store = objstore::open((prefix + "/.tmp").c_str(), true);
			objstore::data *built_store = objstore::open((prefix + "/.built").c_str(), true);
			build_db::data* bdb = build_db::create((prefix + "/.builddb").c_str(), incremental);

			builder::config conf;
			conf.input = input_store;
			conf.temp = tmp_store;
			conf.built = built_store;
			conf.build_db = bdb;
			conf.build_config = build_config;
			builder::data* builder = builder::create(&conf);
			s_config_fn(builder);
	
			char pkg_path[1024];
			sprintf(pkg_path, "%s/packages/", prefix.c_str());
			packaging_config pconf;
			pconf.package_path = pkg_path;
			pconf.rt = rt;
			pconf.bdb = bdb;
			pconf.make_patch = make_patch;
			invoke_packager(built_store, &pconf);

			// Required assets
			std::set<std::string> req;
			for (size_t i=0;i!=pconf.packages.size();i++)
			{
				for (unsigned int j=0;;j++)
				{
					const char *path = package::get_needed_asset(pconf.packages[i].pkg, j);
					if (path)
						req.insert(path);
					else
						break;
				}
			}

			std::set<std::string>::iterator j = req.begin();
			while (j != req.end())
			{
				putki::builder::add_build_root(builder, j->c_str(), 0);
				j++;
			}

			putki::builder::do_build(builder, incremental);

			APP_INFO("Done building. Performing post-build steps.")

			postbuild_info pbi;
			pbi.input = input_store;
			pbi.temp = tmp_store;
			pbi.output = built_store;
			pbi.pconf = &pconf;
			pbi.builder = builder;
			invoke_post_build(&pbi);

			// Post-build step may create new packages, but it must make sure they get built too.
			// So it is up to the post_build_step to run add_build_root for the objects it would like
			// to package.
			putki::builder::do_build(builder, incremental);

			APP_INFO("Done post-build. Writing packages")

			for (unsigned int i=0;i!=pconf.packages.size();i++)
			{
				write_package(&pconf.packages[i], &pconf);
				putki::package::free(pconf.packages[i].pkg);
			}

			objstore::free(input_store);
			objstore::free(tmp_store);
			objstore::free(built_store);
			build_db::store(bdb);
			build_db::release(bdb);
		}
	}
}

