#include "signature.h"

#include <putki/sys/sstream.h>
#include <putki/builder/write.h>

extern "C"
{
	#include <md5/md5.h>
}

namespace putki
{
	namespace signature
	{
		const char *resource(const char*data, size_t size, buffer buf)
		{
			char signature[16];
			md5_buffer(data, (int)size, signature);
			md5_sig_to_string(signature, buf, 64);
			return buf;
		}
		const char *object(type_handler_i* th, instance_t obj, buffer buf)
		{
			putki::sstream ss;
			write::write_object_into_stream(ss, th, obj);

			char signature[16];
			md5_buffer(ss.c_str(), (unsigned int)ss.size(), signature);
			md5_sig_to_string(signature, buf, 64);
			return buf;
		}
	}
}

