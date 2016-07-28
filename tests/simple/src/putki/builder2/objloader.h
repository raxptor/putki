#ifndef __PUTKI_BUILDER2_OBJLOADER_H__
#define __PUTKI_BUILDER2_OBJLOADER_H__

#include <putki/builder/typereg.h>
#include <putki/builder/parse.h>
#include <putki/builder/db.h>
#include "objstore.h"

namespace putki
{
	namespace objloader
	{
		struct data;
		data* create(objstore::data* store);
		void free(data *d);
		bool load_into(data* d, db::data* db, const char* path);
	}
}

#endif
