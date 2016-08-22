namespace putki
{
	namespace db { struct data; }

	namespace liveupdate
	{
		enum {
			EDITOR_PORT      = 5555,
			CLIENT_PORT	     = 5556
		};
	
		struct data;
		data *create();
		void free(data *);
		void run_server(data *d);
	}
}