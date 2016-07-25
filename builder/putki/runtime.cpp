#include "runtime.h"

#include <cstring>
#include <cstdlib>

namespace putki
{
	namespace runtime
	{
		const char * desc_str(desc const * rt)
		{
			static char buf[256];
			switch (rt->platform)
			{
				case PLATFORM_32BIT:
					strcpy(buf, "x");
					break;
				case PLATFORM_64BIT:
					strcpy(buf, "x");
					break;
				case PLATFORM_CSHARP:
					strcpy(buf, "csharp");
					break;
				default:
					strcpy(buf, "unknown");
					break;
			}

			if (buf[0] == 'x')
			{
				if (rt->ptr_size == 8)
					strcat(buf, "64");
				else if (rt->ptr_size == 4)
					strcat(buf, "32");
				else if (rt->ptr_size == 2)
					strcat(buf, "16");
			}

			return buf;
		}

		const desc * get(unsigned int index)
		{
			// if you change this table, re-build the compiler and re-compile everything too.
			static const int count = 3;
			static const desc rtd[count] = {
				// ptrsize, arraysize, boolsize, enumsize, struct_align
				{PLATFORM_64BIT, 8, 4, 1, 4, 8},
				{PLATFORM_32BIT, 4, 4, 1, 4, 4},
				{PLATFORM_CSHARP, 2, 4, 1, 4, 1}
			};

			if (index < count) {
				return &rtd[index];
			}
			return 0;
		}

		const desc * running()
		{
			static desc rt;
			rt.platform = platform();
			rt.ptr_size = sizeof(void*);
			rt.bool_size = sizeof(bool);
			rt.struct_align = sizeof(void*);
			rt.enum_size = sizeof(int);
			return &rt;
		}
	}
}
