#include "types.h"
#include <map>

namespace putki
{	
	std::map<int, type_record> type_records;

	void insert_type_records(const type_record* begin, const type_record* end)
	{
		for (const type_record* i = begin; i != end; i++)
		{
			type_records.insert(std::make_pair(i->id, *i));
		}
	}

	const type_record* get_type_record(int type)
	{
		std::map<int, type_record>::iterator i = type_records.find(type);
		if (i != type_records.end())
		{
			return &i->second;
		}
		return 0;
	}
}
