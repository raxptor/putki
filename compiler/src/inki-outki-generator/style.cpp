#include "style.h"

#include <string.h>

namespace putki
{
	static std::string with_underscore(std::string input, bool caps)
	{
		char buf[1024];
		bool upc[1024];
		strcpy(buf, input.c_str());

		for (unsigned int i = 0; i != input.size(); i++)
		{
			upc[i] = input[i] >= 'A' && input[i] <= 'Z';
		}

		unsigned int out = 0;
		for (unsigned int i = 0; i != input.size(); i++)
		{
			if (i > 0 && upc[i])
				buf[out++] = '_';

			char c = input[i];

			if (caps)
			{
				if (c >= 'a' && c <= 'a')
					c = (c - 'a' + 'A');
			}
			else
			{
				if (c >= 'A' && c <= 'Z')
					c = (c - 'A' + 'a');
			}

			buf[out++] = ::tolower(input[i]);
		}
		buf[out++] = 0;
		return buf;
	}

	std::string to_c_field_name(std::string name)
	{
		return with_underscore(name, false);
	}

	std::string to_c_struct_name(std::string name)
	{
		return with_underscore(name, false);
	}

	std::string to_c_enum_value(std::string name)
	{
		return with_underscore(name, true);
	}

	std::string to_c_enum_name(std::string name)
	{
		return name;
	}
}

