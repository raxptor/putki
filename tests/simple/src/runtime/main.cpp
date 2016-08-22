#include <outki/types/t1.h>
#include <outki/types/t2.h>
#include <outki/test_proj.h>

#include <putki/pkgmgr.h>
#include <putki/pkgloader.h>

int main()
{
	outki::bind_test_proj();
	putki::pkgmgr::loaded_package* pkg = putki::pkgloader::from_file("default.pkg");
	outki::everything* everything = (outki::everything*) putki::pkgmgr::resolve(pkg, "everything");

	for (unsigned int i = 0; i < everything->root_structs_size; i++)
	{
		outki::root_struct* rs = everything->root_structs[i];
		if (!rs)
		{
			continue;
		}
		switch (rs->rtti_type_id())
		{
			case outki::sub_sub_sub_struct1::TYPE_ID:
			{
				outki::sub_sub_sub_struct1* s = (outki::sub_sub_sub_struct1*)(rs);
				break;
			}
			case outki::sub_sub_struct1::TYPE_ID:
			{
				outki::sub_sub_struct1* s = (outki::sub_sub_struct1*)(rs);
				break;
			}
			case outki::sub_struct1::TYPE_ID:
			{
				outki::sub_struct1* s = (outki::sub_struct1*)(rs);
				break;
			}
		}
	}

	return 0;
}
