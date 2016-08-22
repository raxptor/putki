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
			int sent_version;
		};
		
		typedef std::map<std::string, edit> edits_t;
		
		struct ed_session
		{
			std::string name;
			
			sys::mutex mtx;
			sys::condition cond;
			
			int clients;
			bool finished;
			bool ready;
			
			edits_t edits;
			int num_edits;
		};
		
		struct data
		{
			sys::mutex mtx;
			sys::condition cond;
			std::vector<ed_session*> editors;
		};
		
		data *create()
		{
			return new data();
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
				if (!d->editors[i]->clients && d->editors[i]->finished)
				{
					APP_INFO("Cleaning up finished session " << d->editors[i]->name << " because no clients remain on it")
					delete d->editors[i];
					d->editors.erase(d->editors.begin() + i);
					i--;
				}
			}
		}

		// called on the server side from network connection.
		void editor_on_object(ed_session *session, const char *path, sstream *stream)
		{
			sys::scoped_maybe_lock lk(&session->mtx);

			APP_INFO("Got object [" << path << "] as follows. I had " << session->num_edits << " edits")
			APP_INFO(stream->c_str())
			
			edits_t::iterator i = session->edits.find(path);
			if (i != session->edits.end())
			{
				const char *data = stream->c_str();
				i->second.version++;
				i->second.data = std::string(data, data + stream->size());
			}
			else
			{
				const char *data = stream->c_str();
				// insert new edits
				edit e;
				e.version = 1;
				e.data = std::string(data, data + stream->size());
				session->edits.insert(edits_t::value_type(path, e));
			}
			
			session->num_edits++;
			session->cond.broadcast();
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
			ed->num_edits = 0;
			ed->clients = 0;
			
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
								sys::scoped_maybe_lock(&ed->mtx);
								ed->ready = true;
								ed->cond.broadcast();
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
								editor_on_object(ed, obj.c_str(), objbuf);
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
			ed->ready = true;
			delete ptr;

			clean_sessions(d);

			return 0;
		}

		void* client_thread(void *arg)
		{
			/*
			const char *sourcepath = ".";
			
			thr_info *ptr = (thr_info *) arg;
			data *d = ptr->d;
			
			// wait for session.
			ed_session *session = 0;

			APP_INFO("Client on socket " << ptr->socket << " connected")
			
			builder::data *builder = 0;
			runtime::descptr rt = 0;
			std::string config = "Default";

			//
			std::map<std::string, int> sent;
			
			int rd;
			int readpos = 0;
			int parsed = 0, scanned = 0;
			char buf[4096];
			while ((rd = read(ptr->socket, &buf[readpos], sizeof(buf)-readpos)) > 0)
			{
				// try to connect to session
				if (!session)
				{
					sys::scoped_maybe_lock dlk(&d->mtx);
					if (!d->editors.empty())
					{
						APP_INFO("Client on socket " << ptr->socket << " attached to session " << d->editors.back()->name)
						session = d->editors.back();
						session->clients++;
					}
				}

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
						
						if (!strcmp(cmd.c_str(), "poll") && builder && session)
						{
							sys::scoped_maybe_lock lk_(&session->mtx);
							edits_t::iterator e = session->edits.begin();
							while (e != session->edits.end())
							{
								if (sent[e->first] != e->second.version)
								{
									type_handler_i *th;
									instance_t obj;
									
									char main_object[1024];
									if (db::base_asset_path(e->first.c_str(), main_object, sizeof(main_object)))
									{
										// trigger a forced load so it isn't later, with our aux object getting overwritten.
										// if it is not loaded we would create a new object here.
										if (!db::fetch(input_db, main_object, &th, &obj))
										{
											e++;
											continue;
										}
									}
									else
									{
										if (!db::fetch(input_db, e->first.c_str(), &th, &obj))
										{
											APP_INFO("Adding new object " << e->first)
										}
									}
			
									char *tmp = strdup(e->second.data.c_str());
									if (!update_with_json(input_db, e->first.c_str(), tmp, (int)e->second.data.size()))
									{
										APP_WARNING("Json update failed. I am broken now and will exit");
									}
									::free(tmp);

									sent[e->first] = e->second.version;
									updated_objects.push_back(e->first);
									APP_INFO("Adding to build new version [" << e->first << "]");
								}
								++e;
							}
						}
						
						if (!strcmp(cmd.c_str(), "build") && args.size() > 0)
						{
							client_requests.push_back(args[0]);
						}
						
						if (!strcmp(cmd.c_str(), "init") && args.size() > 1)
						{
							// see what runtime it is.
							for (int i=0;; i++)
							{
								runtime::descptr p = runtime::get(i);
								if (!p) break;
								
								if (!strcmp(args[0].c_str(), runtime::desc_str(p)))
									rt = p;
							}
							
							if (!builder)
							{
								builder = builder::create(rt, sourcepath, false, args[1].c_str(), 3);
								if (builder)
								{
									builder::enable_liveupdate_builds(builder);
									APP_INFO("Client has builder " << args[0] << ":" << args[1] << ". Loading DB.")
									input_db = db::create(0, &in_db_mtx);
									tmp_db = db::create(input_db, &tmp_db_mtx);
									load_tree_into_db(builder::obj_path(builder), input_db);
									APP_DEBUG("DB loaded with " << db::size(input_db) << " entries")
									
									db::enable_erase_on_overwrite(tmp_db);
								}
							}
						}
						parsed = scanned + 1;
					}
				}
				
				std::set<std::string> buildset;

				for (int i=0;i!=client_requests.size();i++)
				{
					const char *orgpath = client_requests[i].c_str();

					char main_object[1024];
					if (!db::base_asset_path(orgpath, main_object, sizeof(main_object)))
						strcpy(main_object, orgpath);

					APP_DEBUG("Client requested [" << client_requests[i] << "], for which [" << orgpath << "] will be built.");
					buildset.insert(main_object);
				}
				
				// this array will be empty till there are builders.
				for (int k=0;k!=updated_objects.size();k++)
				{
					// find all objects that need a rebuild because of this update.
					build_db::data *bdb = builder::get_build_db(builder);
					build_db::deplist *dl = build_db::deplist_get(bdb, updated_objects[k].c_str());
					for (int i=-1;; i++)
					{
						const char *path = (i == -1) ? updated_objects[k].c_str() : build_db::deplist_entry(dl, i);
						if (!path) {
							break;
						}

						// main object always -1
						if (i != -1 && !strcmp(path, updated_objects[k].c_str()))
							continue;
					
						type_handler_i *th;
						instance_t obj;
						if (db::fetch(input_db, path, &th, &obj) && !th->in_output())
						{
							// skip this, not in output.
							continue;
						}
				
						buildset.insert(path);
						
						if (!dl)
						{
							APP_WARNING("deplist_get(build, '" << updated_objects[k] << "') returned 0!")
							break;
						}
					}
				}
				
				std::set<std::string>::iterator bq = buildset.begin();
				while (bq != buildset.end())
				{
					std::string tobuild = *(bq++);
					APP_DEBUG("Building [" << tobuild << "]...")

					char b[1024];
					if (db::is_aux_path(tobuild.c_str()))
					{
						if (db::base_asset_path(tobuild.c_str(), b, 1024))
						{
							if (buildset.count(b))
							{
								APP_DEBUG("Main object already in set, skipping")
								continue;
							}
							
							APP_DEBUG("...actually " << b)
							tobuild = b;
						}
					}

					// walk up the parent chain of this and find out where we need to start to make this asset.
					while (true)
					{
						if (db::exists(tmp_db, tobuild.c_str(), true) || db::exists(input_db, tobuild.c_str(), true))
							break;
	
						build_db::record *rec = build_db::find(builder::get_build_db(builder), tobuild.c_str());
						if (!rec || !build_db::get_parent(rec))
							break;

						tobuild = build_db::get_parent(rec);
					}

					// load asset into source db if missing.
					if (!db::exists(tmp_db, tobuild.c_str(), true) && !db::exists(input_db, tobuild.c_str(), true))
					{
						APP_WARNING("This object cannot be built!")
						continue;
					}

					build::resolve_object(input_db, tobuild.c_str());
					
					db::data *output_db = db::create(tmp_db, &out_db_mtx);
					
					builder::build_source_object(builder, input_db, tmp_db, output_db, tobuild.c_str());
					build::post_build_ptr_update(input_db, output_db);
					build::post_build_ptr_update(tmp_db, output_db);

					package::data *pkg = package::create(output_db);
					package::add(pkg, tobuild.c_str(), true);
					
					// monster buffer.. make bigger?
					const unsigned int sz = 256*1024*1024;
					char *buf = new char[sz];
					
					putki::sstream mf;
					long bytes = package::write(pkg, rt, buf, sz, builder::get_build_db(builder), mf);
					
					APP_INFO("Package is " << bytes << " bytes")

					if (send(ptr->socket, buf, bytes, 0) != bytes)
					{
						// broken pipe
						APP_INFO("Failed to write all data, socket was closed?")
						close(ptr->socket);
						ptr->socket = -1;
						break;
					}

					delete [] buf;
					
					db::free_and_destroy_objs(output_db);
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

			if (session)
			{
				sys::scoped_maybe_lock dlk(&d->mtx);
				session->clients--;
				dlk.unlock();
				clean_sessions(d);
			}

			if (ptr->socket != -1)
				close(ptr->socket);
			
			db::free_and_destroy_objs(input_db);
			db::free_and_destroy_objs(tmp_db);
	
			delete ptr;
			*/
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
