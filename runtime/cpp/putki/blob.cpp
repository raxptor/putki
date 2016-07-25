#include "log/log.h"

namespace putki
{
	typedef unsigned short strsize_t;

	char* post_blob_load_string(const char **string, char* aux_beg, char* aux_end)
	{
		if (!aux_beg) {
			return 0;
		}

		unsigned int *lptr = (unsigned int*) string;
		unsigned int len = *lptr;

		*string = "<UNPACK FAIL>";

		if (aux_beg + len <= aux_end)
		{
			const char *last = aux_beg + len - 1;
			if (*last == 0x00) {
				*string = (const char*)aux_beg;
			}
			else{
				*string = "<UNPACK LAST NON-ZERO>";
			}

			aux_beg += len;
		}
		else
		{
			PTK_ERROR("Not enough bytes in stream");
			return 0;
		}
		return aux_beg;
	}
}

