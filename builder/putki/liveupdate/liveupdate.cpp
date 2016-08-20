#include <putki/liveupdate/liveupdate.h>

#include <putki/builder/builder.h>
#include <putki/builder/log.h>
#include <putki/builder/build.h>
#include <putki/builder/write.h>
#include <putki/builder/build-db.h>
#include <putki/builder/package.h>
#include <putki/builder/log.h>
#include <putki/builder/parse.h>
#include <putki/sys/thread.h>
#include <putki/sys/sstream.h>
#include <putki/sys/socket.h>

#include <iostream>
#include <string>
#include <vector>
#include <cstdio>
#include <map>
#include <set>

namespace putki
{

	namespace liveupdate
	{
		sock_t skt_listen(int port)
		{
			sock_t s = socket(AF_INET, SOCK_STREAM, 0);
			if (s < 0)
			{
				std::cerr << "Could not open listening socket" << std::endl;
				return 0;
			}

			int optval = 1;
			setsockopt(s, SOL_SOCKET, SO_REUSEADDR, &optval, sizeof(optval));

			sockaddr_in srv;
			srv.sin_family = AF_INET;
			srv.sin_addr.s_addr = INADDR_ANY;
			srv.sin_port = htons(port);
			if (bind(s, (sockaddr*)&srv, sizeof(srv)) < 0)
			{
				std::cerr << "Binding failed." << std::endl;
				close(s);
				return -1;
			}

			listen(s, 10);
			return s;
		}
		
		sock_t skt_accept(sock_t lp)
		{
			sockaddr_in client;
			socklen_t sz = sizeof(client);
			return accept(lp, (sockaddr*)&client, &sz);
		}
		
		struct edit
		{
			std::string data;
			int version;
		};

		typedef std::map<std::string, int> seen_edits_map;

		struct builder_info
		{
			runtime::descptr rt;
			std::string config_name;
			putki::builder::config config;
			putki::builder::data* builder;
			seen_edits_map seen_edits;
			int num_edits;

			sys::mutex build_mtx;
			
			sys::condition queue_cond;
			std::set<std::string> queue;

			sys::condition num_cond;
			int num_builds;

			sys::thread* thread;
		};
		
		typedef std::map<std::string, edit> edits_map;

		struct ed_session
		{
			std::string name;
			bool finished;
			bool ready;
		};
		
		struct data
		{
			sys::mutex mtx;
			sys::condition cond;
			std::vector<ed_session*> editors;

			sys::mutex builders_mtx;
			std::vector<builder_info*> builders;
			// 
			sys::mutex edits_mtx;
			edits_map edits;
			int num_edits;
		};
		
		data *create()
		{
			data* d = new data();
			d->num_edits = 0;
			return d;
		}
		
		void free(data *d)
		{
			delete d;
		}
		
		struct thr_info
		{
			data *d;
			sock_t socket;
		};

		void clean_sessions(data *d)
		{
			sys::scoped_maybe_lock lk(&d->mtx);
			for (int i=0;i!=d->editors.size();i++)
			{
				if (d->editors[i]->finished)
				{
					APP_INFO("Cleaning up finished editor session " << d->editors[i]->name)
					delete d->editors[i];
					d->editors.erase(d->editors.begin() + i);
					i--;
				}
			}
		}

		// called on the server side from network connection.
		void editor_on_object(data* d, const char *path, sstream *stream)
		{
			APP_INFO("Got object [" << path << "]")
			sys::scoped_maybe_lock lk(&d->edits_mtx);

			edits_map::iterator i = d->edits.find(path);
			if (i != d->edits.end())
			{
				const char *data = stream->c_str();
				i->second.version++;
				i->second.data = std::string(data, data + stream->size());
			}
			else
			{
				const char *data = stream->c_str();
				edit e;
				e.version = 1;
				e.data = std::string(data, data + stream->size());
				d->edits.insert(edits_map::value_type(path, e));
			}
			d->num_edits++;
		}
		
		void* editor_client_thread(void *arg)
		{
			thr_info *ptr = (thr_info *) arg;
			
			sstream name;
			name << "psd-" << (ptr->socket) << ptr;
			
			APP_INFO("Editor session created on socket " << ptr->socket << " with name " << name.c_str())
			
			data *d = ptr->d;
			
			ed_session *ed = new ed_session();
			ed->finished = false;
			ed->ready = false;

			sys::scoped_maybe_lock dlk(&d->mtx);
			d->editors.push_back(ed);
			d->cond.broadcast();
			dlk.unlock();
			
			char buf[65536];
			
			sstream tmp;
			sstream *objbuf = 0;
			std::string obj;
			
			int rd;
			int readpos = 0;
			int parsed = 0, scanned = 0;
			while ((rd = read(ptr->socket, &buf[readpos], sizeof(buf)-readpos)) > 0)
			{
				readpos += rd;
				
				for (;scanned!=readpos;scanned++)
				{
					if (buf[scanned] == 0xd || buf[scanned] == 0xa)
					{
						buf[scanned] = 0x0;
						const char *line = &buf[parsed];
						APP_DEBUG("Editor:" << line);

						if (!strcmp(line, "<keepalive>"))
						{
							continue;
						}
						
						// ignore empty lines
						if (scanned == parsed)
						{
							parsed++;
							continue;
						}
						
						if (!objbuf)
						{
							if (!strcmp(line, "<ready>"))
							{
								APP_INFO("Editor session is ready")
							}
							else
							{
								obj = line;
								objbuf = &tmp;
								APP_INFO("Receiving object [" << obj << "]")
							}
						}
						else
						{
							if (!strcmp(line, "<end>"))
							{
								editor_on_object(d, obj.c_str(), objbuf);
								objbuf->clear();
								objbuf = 0;
							}
							else
							{
								(*objbuf) << line << "\n";
							}
						}
						parsed = scanned + 1;
					}
				}
				
				for (int i=parsed;i!=readpos;i++)
				{
					buf[i-parsed] = buf[i];
				}
				
				scanned -= parsed;
				readpos -= parsed;
				parsed = 0;
				
				if (readpos == sizeof(buf))
				{
					APP_INFO("Read buffer overflowed. Malfunction in communication.");
					break;
				}
			}
			
			APP_INFO("Editor session " << name.c_str() << " finished.")
			close(ptr->socket);
			ed->finished = true;
			delete ptr;
			clean_sessions(d);
			return 0;
		}

		void* builder_thread(void *arg)
		{
			builder_info* info = (builder_info*)arg;

			sys::scoped_maybe_lock lk_(&info->build_mtx);

			APP_INFO("Doing initial incremental build without writing packages...");
			build::add_build_roots(info->builder, &info->config, info->rt, info->config_name.c_str());
			putki::builder::do_build(info->builder, true);
			APP_INFO("Done building. Performing post-build steps.");

			// TODO: Fix post build things someow 
			/*
			build::postbuild_info pbi;
			pbi.input = info->config.input;
			pbi.temp = info->config.temp;
			pbi.output = info->config.built;
			pbi.pconf = &info->config;
			pbi.builder = bld;
			invoke_post_build(&pbi);
			putki::builder::do_build(bld, incremental);
			*/

			while (true)
			{
				sys::scoped_maybe_lock lk_(&info->build_mtx);
				while (info->queue.empty())
				{
					info->queue_cond.wait(&info->build_mtx);
				}

				for (std::set<std::string>::iterator i = info->queue.begin(); i != info->queue.end(); i++)
				{
					builder::add_build_root(info->builder, i->c_str(), -1);
				}
				builder::do_build(info->builder, true);
			}
		}

		void* client_thread(void *arg)
		{
			seen_edits_map seen;
			thr_info *ptr = (thr_info *) arg;
			data *d = ptr->d;
			builder_info* builder = 0;

			int rd;
			int readpos = 0;
			int parsed = 0, scanned = 0;
			char buf[4096];
			while ((rd = read(ptr->socket, &buf[readpos], sizeof(buf)-readpos)) > 0)
			{
				std::vector<std::string> client_requests;
				std::vector<std::string> updated_objects;
				
				readpos += rd;
				for (;scanned!=readpos;scanned++)
				{
					if (buf[scanned] == 0xd || buf[scanned] == 0xa)
					{
						buf[scanned] = 0x0;

						// ignore empty lines
						if (scanned == parsed)
						{
							parsed++;
							continue;
						}
						
						std::string cmd(buf + parsed, buf + scanned);
						std::string argstring;

						std::vector<std::string> args;
						size_t del = cmd.find_first_of(' ');
						if (del != std::string::npos)
						{
							argstring = cmd.substr(del+1, cmd.size() - del);
							cmd = cmd.substr(0, del);
						}

						while (!argstring.empty())
						{
							del = argstring.find_first_of(' ');
							if (del == std::string::npos)
							{
								args.push_back(argstring);
								break;
							}
							
							args.push_back(argstring.substr(0, del));
							argstring.erase(0, del + 1);
						}
						
						if (!strcmp(cmd.c_str(), "poll"))
						{
							// See what objects have been changed but this client does not known of;
							// put them in updated_objects.
							sys::scoped_maybe_lock lk_(&d->edits_mtx);
							edits_map::iterator e = d->edits.begin();
							while (e != d->edits.end())
							{
								if (seen[e->first] != e->second.version)
								{
									updated_objects.push_back(e->first);
									seen[e->first] = e->second.version;
								}
								++e;
							}
						}
						
						if (!strcmp(cmd.c_str(), "get") && args.size() > 0)
						{
							client_requests.push_back(args[0]);
						}
						
						if (!strcmp(cmd.c_str(), "init") && args.size() > 1)
						{
							// see what runtime it is.
							runtime::descptr rt = 0;
							for (int i = 0;; i++)
							{
								runtime::descptr p = runtime::get(i);
								if (!p)
								{
									break;
								}
								if (!strcmp(args[0].c_str(), runtime::desc_str(p)))
								{
									rt = p;
								}
							}

							if (rt)
							{
								if (!builder)
								{
									sys::scoped_maybe_lock lk_(&d->builders_mtx);
									if (!builder && !rt)
									{
										for (size_t i = 0; i != d->builders.size(); i++)
										{
											if (d->builders[i]->rt == rt && !strcmp(d->builders[i]->config_name.c_str(), args[1].c_str()))
											{
												APP_INFO("Found a builder for client.");
												builder = d->builders[i];
												break;
											}
										}
									}
									if (!builder)
									{
										APP_INFO("Could not find builder for client. Making one");
										builder_info* info = new builder_info;
										info->rt = rt;
										info->config_name = args[1].c_str();
										info->num_edits = 0;
										build::init_builder_configuration(&info->config, rt, args[1].c_str(), true);
										info->builder = build::create_and_config_builder(&info->config);
										info->thread = sys::thread_create(builder_thread, (void*)info);
										d->builders.push_back(info);
										builder = info;


									}
								}
							}
							else
							{
								APP_INFO("Could not find runtime desc for [" << args[0] << "]!");
							}
						}
						parsed = scanned + 1;
					}
				}

				if (!builder)
				{
					continue;
				}
				
				std::set<std::string> get_set;
				for (int i=0;i!=client_requests.size();i++)
				{
					get_set.insert(client_requests[i]);
				}

				for (int i=0;i!=updated_objects.size(); i++)
				{
					get_set.insert(updated_objects[i]);
					build_db::deplist* list = deplist_get(builder->config.build_db, updated_objects[i].c_str());
					for (int j=0;;j++)
					{
						const char *path = build_db::deplist_entry(list, j);
						if (!path) 
						{
							break;
						}
						get_set.insert(path);
					}
					build_db::deplist_free(list);
				}

				// Now get_set contains all the objects that need to be sent to the client. 
				// Either he requested them, or they were changed by a connected editor.
				sys::scoped_maybe_lock lk(&builder->build_mtx);
				int num_builds = builder->num_builds;
				for (std::set<std::string>::iterator i = get_set.begin(); i != get_set.end(); i++)
				{
					builder->queue.insert(*i);
				}

				builder->queue_cond.broadcast();

				// Once num_builds go up, the builder thread will have built everything in the queue.
				while (builder->num_builds == num_builds)
				{
					builder->num_cond.wait(&builder->build_mtx);
				}

				package::data *pkg = package::create(builder->config.built);
				for (std::set<std::string>::iterator i = get_set.begin(); i != get_set.end(); i++)
				{
					package::add(pkg, i->c_str(), true);
				}

				const unsigned int sz = 256*1024*1024;
				char *buf = new char[sz];
				putki::sstream mf;
				long bytes = package::write(pkg, builder->rt, buf, sz, builder->config.build_db, mf);

				APP_INFO("Package is " << bytes << " bytes")
				int result = send(ptr->socket, buf, bytes, 0);
				delete[] buf;

				if (result != bytes)
				{
					// broken pipe
					APP_INFO("Failed to write all data, socket was closed?")
					close(ptr->socket);
					ptr->socket = -1;
					break;
				}

				if (ptr->socket == -1)
					break;
			
				for (int i=parsed;i!=readpos;i++)
					buf[i-parsed] = buf[i];
				
				scanned -= parsed;
				readpos -= parsed;
				parsed = 0;
				
				if (readpos == sizeof(buf))
				{
					APP_INFO("Read buffer overflowed. Malfunction in communication.");
					break;
				}
			}

			APP_INFO("Client session ended")

			if (ptr->socket != -1)
				close(ptr->socket);

			return 0;
		}
		
		
		void* editor_listen_thread(void *arg)
		{
			APP_INFO("Live update: waiting for editors...");

			data *d = (data *)arg;
			sock_t s = skt_listen(EDITOR_PORT);

			while (s != -1)
			{
				thr_info *inf = new thr_info();
				inf->socket = skt_accept(s);
				if (inf->socket <= 0)
				{
					delete inf;
					break;
				}
				inf->d = d;
				sys::thread_create(&editor_client_thread, inf);
			}
			return 0;
		}
		
		void* client_listen_thread(void *arg)
		{
			APP_INFO("Live update: waiting for clients...");

			data *d = (data *)arg;
			sock_t s = skt_listen(CLIENT_PORT);

			while (s != -1)
			{
				thr_info *inf = new thr_info();
				inf->socket = skt_accept(s);
				if (inf->socket <= 0)
				{
					delete inf;
					break;
				}
				inf->d = d;
				sys::thread_create(&client_thread, inf);
			}
			return 0;
		}
		
		void run_server(data *d)
		{
			signal(SIGPIPE, SIG_IGN);
			sys::thread *thr0 = sys::thread_create(editor_listen_thread, d);
			sys::thread *thr1 = sys::thread_create(client_listen_thread, d);
			sys::thread_join(thr0);
			sys::thread_join(thr1);
		}
	}
}
