#include <outki/types/t1.h>
#include <outki/types/t2.h>
#include <outki/test_proj.h>

#include <putki/pkgmgr.h>
#include <putki/pkgloader.h>
#include <putki/liveupdate/liveupdate.h>
#include <putki/log/log.h>
#include <iostream>
#include <Windows.h>

int main()
{
	outki::bind_test_proj();
	putki::pkgmgr::loaded_package* pkg = putki::pkgloader::from_file("default.pkg");
	outki::everything* everything = (outki::everything*) putki::pkgmgr::resolve(pkg, "everything");

	putki::set_loglevel(putki::LOG_DEBUG);
	putki::liveupdate::init();
	putki::liveupdate::data* data = 0;

	while (true)
	{
		if (data && !putki::liveupdate::connected(data))
		{
			putki::liveupdate::disconnect(data);
			data = 0;
		}
		if (!data)
		{
			data = putki::liveupdate::connect();
		}

		if (LIVE_UPDATE(&everything))
		{
			std::cout << "Everything changed!\n" << std::endl;
		}

		std::cout << "vt inline text=" << everything->vt_inline.text << std::endl;
		
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
		Sleep(100);
		if (data)
		{
			putki::liveupdate::update(data);
		}

	}

	return 0;
}
