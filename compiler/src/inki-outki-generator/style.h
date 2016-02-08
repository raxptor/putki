#ifndef __PUTKI_GENERATOR_STYLE_H__
#define __PUTKI_GENERATOR_STYLE_H__

#include <string>

namespace putki
{
	std::string to_c_field_name(std::string name);
	std::string to_c_struct_name(std::string name);
	std::string to_c_enum_name(std::string name);
	std::string to_c_enum_value(std::string name);	
}


#endif
