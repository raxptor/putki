#include "write.h"

#include <iostream>
#include <fstream>
#include <vector>
#include <string>

#include <putki/builder/typereg.h>
#include <putki/sys/files.h>
#include <putki/sys/sstream.h>

namespace putki
{
	namespace write
	{
		void write_object_into_stream(putki::sstream & out, type_handler_i *th, instance_t obj)
		{
			out << "{\n";
			out << "	type: "<< json_str(th->name()) << ",\n";
			out << "	data: {\n";
			th->write_json(obj, out, 1);
			out << "	}\n";
			out << "}\n";
		}

		namespace
		{
			static const char *hex = "0123456789abcdef";
		}

		void json_stringencode_byte_array(putki::sstream & out, std::vector<unsigned char> const &bytes)
		{
			size_t i;
			const size_t blk = 8;
			for (i=0;(i+blk)<=bytes.size();i+=blk)
			{
				uint64_t* src = (uint64_t*) &bytes[i];
				uint64_t a = *src;
				uint64_t b = *src;
				const uint64_t a32 = ('a' << 24) | ('a' << 16) | ('a' << 8) | 'a';
				const uint64_t a64 = (a32 << 32) | a32;
				a = a & 0xf0f0f0f0f0f0f0f0;
				b = b & 0x0f0f0f0f0f0f0f0f;
				uint64_t res_a = (a >> 4) + a64;
				uint64_t res_b = b + a64;
				const char *high = (const char*) &res_a;
				const char *low = (const char*) &res_b;
				char* write = out.append_block(2*blk);
				for (int j=0;j<blk;j++)
				{
					write[2*j] = high[j];
					write[2*j+1] = low[j];
				}
			}
			for (;i<bytes.size();i++)
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

