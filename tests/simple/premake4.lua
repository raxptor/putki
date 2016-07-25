solution "Test"

    configurations {"Release"}
    platforms {"x64"}

    flags { "Symbols" }

    location "build"
    targetdir "build"

    defines("_CRT_SECURE_NO_WARNINGS")
    defines("BUILDER_DEFAULT_RUNTIME=x64")
    defines("LIVEUPDATE_ENABLE")
    defines("PUTKI_ENABLE_LOG")
    defines("KOSMOS_ENABLE_LOG")

    configuration {"linux", "gmake"}
        buildoptions {"-fPIC"}
        buildoptions ("-std=c++11")
    configuration {}

    ------------------------------------
    -- Putki must always come first   --
    ------------------------------------

    dofile "../../runtime/premake.lua"
	dofile "../../builder/premake.lua"

    project "test-putki-lib"
        language "C++"
        targetname "test-putki-lib"
        kind "StaticLib"
        putki_use_builder_lib()
        putki_typedefs_builder("src/types", true)

    project "test-databuilder"

        kind "ConsoleApp"
        language "C++"
        targetname "test-databuilder"

        files { "src/builder-main.cpp" }
        files { "src/builder/**.*" }
        links { "test-putki-lib" }
        includedirs { "src" }
        
        putki_use_builder_lib()
        putki_typedefs_builder("src/types", false)

    project "test-runtime"
        kind "ConsoleApp"
        language "C++"
        targetname "test-runtime"
        files { "src/runtime/main.cpp" }
        putki_use_runtime_lib()
        putki_typedefs_runtime("src/types", true)

