#include "write.h"

#include <iostream>
#include <fstream>
#include <vector>
#include <string>

#include <putki/builder/db.h>
#include <putki/sys/files.h>
#include <putki/sys/sstream.h>

namespace putki
{
	namespace write
	{
		struct auxwriter : public depwalker_i
		{
			std::vector<std::string> paths;
			std::vector<std::string> subpaths;
			db::data *ref_source;
			instance_t base, start;
			type_handler_i *th;

			virtual bool pointer_pre(instance_t *on, const char *ptr_type)
			{
				instance_t obj = *on;
				if (!obj) {
					return false;
				}

				// returning back to where we were.
				if (obj == start)
					return false;

				const char *path = db::pathof(ref_source, obj);
				if (path && db::is_aux_path_of(ref_source, base, path))
				{
					// std::cout << "Including aux object [" << path << "]" << std::endl;
					paths.push_back(path);

					auxwriter aw;
					if (db::fetch(ref_source, path, &aw.th, &aw.base))
					{
						aw.start = start;
						aw.ref_source = ref_source;
						aw.th->walk_dependencies(aw.base, &aw, false);

						for (unsigned int i=0; i<aw.paths.size(); i++)
							subpaths.push_back(aw.paths[i]);
						for (unsigned int i=0; i<aw.subpaths.size(); i++)
							subpaths.push_back(aw.subpaths[i]);
					}
					else
					{
						*on = 0;
					}
				}

				return true;
			}

			virtual void pointer_post(instance_t *on)
			{

			}
		};


		void write_object_into_stream(putki::sstream & out, db::data *ref_source, type_handler_i *th, instance_t obj)
		{
			out << "{\n";
			out << "	type: "<< json_str(th->name()) << ",\n";
			out << "	data: {\n";
			th->write_json(ref_source, obj, out, 1);
			out << "	}\n";
			out << "}\n";
		}

		bool write_object_to_fs(const char *basedir, const char *path, db::data *ref_source, type_handler_i *th, instance_t obj, char *fn_out)
		{
			std::string out_path(basedir);
			out_path.append("/");
			out_path.append(path);
			out_path.append(".json");
			putki::sstream ts;
			write::write_object_into_stream(ts, ref_source, th, obj);
			sys::mk_dir_for_path(out_path.c_str());
			return sys::write_file(out_path.c_str(), ts.str().c_str(), (unsigned long)ts.str().size());
		}

		namespace
		{
			static const char *hex = "0123456789abcdef";
		}

		void json_stringencode_byte_array(putki::sstream & out, std::vector<unsigned char> const &bytes)
		{
			for (unsigned int i=0;i<bytes.size();i++)
			{
				out << (char)('a' + ((bytes[i] >> 4) & 0xf));
				out << (char)('a' + ((bytes[i]) & 0xf));
			}
		}

		std::string json_str(const char *input)
		{
			if (!input) {
				return "\"\"";
			}

			const int len = (int)strlen(input);

			putki::sstream ss;
			ss << "\"";
			for (size_t i = 0; i != len; ++i) {
				char val = input[i];
				if (unsigned(val) < '\x20' || val == '\\' || val == '"') {
					char buf[16];
					buf[0] = '\\';
					buf[1] = 'u';
					for (int k=0;k<4;k++)
						buf[2+k] = hex[(val >> 4*(3-k)) & 0xf];
					buf[6] = 0;
					ss << buf;
				}
				else
				{
					ss << val;
				}
			}
			ss << "\"";
			return ss.c_str();
		}

		const char *json_indent(char *buf, int level)
		{
			int i;
			for (i=0; i<level; i++)
				buf[i] = '\t';
			buf[i] = 0;
			return buf;
		}
	}
}

