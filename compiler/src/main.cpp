#include <parser/typeparser.h>
#include <parser/treeparser.h>
#include <parser/resolve.h>

#include <putki/sys/files.h>

#include <iostream>
#include <sstream>
#include <fstream>

#include "inki-outki-generator/generator.h"
#include "inki-outki-generator/style.h"
#include "netki-generator/netki-generator.h"

#include "writetools/indentedwriter.h"
#include "writetools/save_stream.h"

void generate_project(putki::project *p)
{
	std::string out_base(p->start_path);
	out_base.append("/_gen");

	std::string outki_base(out_base + "/outki");
	std::string inki_base(out_base + "/inki");

	std::stringstream rt_blob_load_calls;
	std::stringstream rt_blob_load_header;

	std::stringstream bind_decl;
	std::stringstream bind_calls;

	std::stringstream inki_master, runtime_master;

	for (int i=0;i!=p->files.size();i++)
	{
		putki::parsed_file *pf = &p->files[i];

		std::string subpath = pf->sourcepath.substr(p->base_path.size()+1);
		std::string rt_path = outki_base + "/" + subpath;
		std::string inki_path = inki_base + "/" + subpath;

		// c++ runtime header
		std::stringstream rt_header;
		putki::write_runtime_header(pf, 0, putki::indentedwriter(rt_header));
		putki::save_stream(rt_path + ".h", rt_header);

		// c++ runtime implementation
		std::stringstream rt_impl;
		putki::write_runtime_impl(pf, 0, putki::indentedwriter(rt_impl));
		putki::save_stream(rt_path + ".cpp", rt_impl);

		// c++ runtime blob load calls
		putki::indentedwriter iw(rt_blob_load_calls);
		iw.indent(3);
		putki::write_runtime_blob_load_cases(pf, iw);
		rt_blob_load_header << "#include <outki/" << subpath << ".h>\n";

		// c++ inki header
		std::stringstream inki_header;
		putki::write_putki_header(pf, putki::indentedwriter(inki_header));
		putki::save_stream(inki_path + ".h", inki_header);

		// c++ runtime implementation
		std::stringstream inki_impl;
		putki::write_putki_impl(pf, putki::indentedwriter(inki_impl));
		putki::save_stream(inki_path + ".cpp", inki_impl);

		// bindings
		putki::write_bind_decl(pf, putki::indentedwriter(bind_decl));
		putki::write_bind_calls(pf, putki::indentedwriter(bind_calls));

		inki_master << "#include \"inki/" << subpath << ".cpp\"\n";
		runtime_master << "#include \"outki/" << subpath << ".cpp\"\n";
	}

	// bind calls
	{
		std::stringstream bind;
		bind << bind_decl.str() << std::endl;

		putki::indentedwriter iw(bind);
		iw.line() << "namespace inki";
		iw.line() << "{";
		iw.indent(1);
		iw.line() << "void bind_" << putki::to_c_struct_name(p->module_name) << "()";
		iw.line() << "{";
		iw.indent(1);
		iw.line();
		bind << bind_calls.str();
		iw.indent(-1);
		iw.line() << "}";
		iw.indent(-1);
		iw.line() << "}";
		putki::save_stream(inki_base + "/bind.cpp", bind);

		inki_master << "#include \"inki/bind.cpp\"\n";
	}

	// c++ runtime blob switcher
	{
		std::stringstream output;
		output << rt_blob_load_header.str();
		putki::indentedwriter iw(output);
		iw.line() << "#include <putki/blob.h>";
		iw.line() << "#include <putki/types.h>";
		iw.line();
		iw.line() << "namespace outki";
		iw.line() << "{";
		iw.indent(1);
		iw.line() << "char* post_blob_load_" << p->module_name << "(int type, putki::depwalker_i *ptr_reg, char *begin, char *end)";
		iw.line() << "{";
		iw.indent(1);
		iw.line() << "switch (type)";
		iw.line() << "{";
		output << rt_blob_load_calls.str();
		iw.line(1) << "default:";
		iw.line(2) << "return 0;";
		iw.line() << "}";
		iw.indent(-1);
		iw.line() << "}";
		iw.line();
		iw.line() << "void bind_" << p->module_name << "_loaders()";
		iw.line() << "{";
		iw.line(1) << "add_blob_loader(post_blob_load_" << p->module_name << ");";
		iw.line() << "}" << std::endl;
		iw.indent(-1);
		iw.line() << "}";
		putki::save_stream(outki_base + "/blobload.cpp", output);

		runtime_master << "#include \"outki/blobload.cpp\"\n";
	}

	putki::save_stream(out_base + "/" + p->module_name + "-inki-master.cpp", inki_master);
	putki::save_stream(out_base + "/" + p->module_name + "-outki-runtime-master.cpp", runtime_master);
}

int main (int argc, char *argv[])
{
	putki::grand_parse parse;
	if (!putki::parse_all_with_deps(&parse))
	{
		std::cerr << "Aborting on parse error." << std::endl;
		return -1;
	}

	putki::resolved_parse res;
	if (!putki::resolve_parse(&parse, &res))
	{
		std::cerr << "Aborting on resolve error." << std::endl;
		return -1;
	}

	for (int i=0;i!=parse.projects.size();i++)
	{
		generate_project(&parse.projects[i]);
		putki::build_netki_project(&parse.projects[i]);
	}

	return 0;
}
