PUTKI_PATH = path.getdirectory(_SCRIPT) .. "/../"
BUILDER_PATH = path.getdirectory(_SCRIPT)
ZLIB_PATH = PUTKI_PATH .. "/external/libz"
ZLIB_INCLUDES = { ZLIB_PATH }
PUTKI_LIB_INCLUDES = { BUILDER_PATH }
PUTKI_LIB_LINKS = { "putki-lib", "jsmn", "libz" }

function putki_use_builder_lib()
	includedirs ( PUTKI_LIB_INCLUDES )
	links (PUTKI_LIB_LINKS)
	configuration {"windows"}
		links {"ws2_32"}
	configuration {"gmake", "linux"}
		links {"pthread"}
	configuration {}
end

function putki_typedefs_builder(path, use_impls, pathbase)
	if pathbase == nil then
		pathbase = "."
	end
	includedirs (pathbase .. "/_gen/cpp")
	files { pathbase .. "/" .. path .. "/**.typedef" }
	if use_impls == true then
		files { pathbase .. "/_gen/cpp/inki/inki-master.cpp", pathbase .. "/_gen/cpp/inki/**.h" }
	end
end

function make_putki_lib(name)


end

defines { "JSMN_PARENT_LINKS" }

configuration {"windows"}
	defines {"USE_WINSOCK"}
	
if os.get() == "bsd" or os.get() == "linux" then
	table.insert(PUTKI_LIB_LINKS, "pthread")
end

dofile "../external/libz/premake.lua"

project "jsmn"

	kind "StaticLib"
	targetname "jsmn"
	language "C++"
	files { "../external/jsmn/*.cpp", "../external/jsmn/*.h"}

project "putki-lib"

	language "C++"
	targetname "putki-lib"

	kind "StaticLib"
	files { "**.cpp" }
	files { "**.h" }
	files { "**.c" }
		
	includedirs { ".", "../external"}

	links {"jsmn"}
	links {"libz"}

	configuration {"windows"}
		links {"ws2_32"}
	configuration {"gmake", "linux"}
		links {"pthread"}

if os.get() == "windows" and false then
	project "putki-runtime-csharp"
		kind "SharedLib"
		language "C#"
		targetname "putki-runtime-csharp"
		files { "csharp/**.cs"}
		links { "System" }
end
	