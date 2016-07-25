using System;
using System.Net;
using System.Net.Sockets;
using System.Collections.Generic;

namespace Netki
{
	// Returns buffer for reading the next time.
	public delegate byte[] OnDatagramDelegate(byte[] data, uint length, ulong EndPoint);

	public class PacketDatagramServer
	{
		private Socket _listener;
		private OnDatagramDelegate _pkt;
		private int _port;
		private string _host;

		public PacketDatagramServer(OnDatagramDelegate pkt)
		{
			_host = "localhost";
			_pkt = pkt;
		}

        public void Terminate()
        {
            _listener.Close();
        }

		private void NextRead()
		{
            
		}

		public int GetPort()
		{
			return _port;
		}

		public string GetHost()
		{
			return _host;
		}

		private void ReadLoop()
		{
			EndPoint ep = new IPEndPoint(IPAddress.Any, 0);
			try
			{
				byte[] _recvBuf = new byte[4096];
    			while (true)
    			{
					try
					{
	    				int bytes = _listener.ReceiveFrom(_recvBuf, ref ep);
	    				if (bytes > 0)
	    				{
	    					IPEndPoint ipep = (IPEndPoint)ep;
	    					byte[] addr = ipep.Address.GetAddressBytes();
	    					ulong port = (uint)ipep.Port;
	    					ulong addr_portion = ((ulong)addr[3] << 24) | ((ulong)addr[2] << 16) | ((ulong)addr[1] << 8) | (ulong)addr[0];
	    					ulong endpoint = addr_portion | (port << 32);
	    					_recvBuf = _pkt(_recvBuf, (uint)bytes, endpoint);
						}
					}
					catch (SocketException)
					{
						// Ignore and continue...
					}
    			}
			}

			catch (ObjectDisposedException)
			{

			}
		}

		public void Send(byte[] data, int offset, int length, ulong endpoint)
		{
			ulong ep = endpoint >> 32;
			int port = (int)(ep & 0xffff);
			IPAddress p = new IPAddress((long)(endpoint & 0xffffffff));
			IPEndPoint ipep = new IPEndPoint(p, port);
			try {
				int ret = _listener.SendTo(data, offset, length, 0, ipep);
				if (ret != length)
				{
					Console.WriteLine("Send failed");
				}
			}
			catch (Exception e)
			{
				Console.WriteLine("Exception during send " + e.ToString());
			}
		}

		public void Start(int port = 0)
		{
			IPEndPoint localEP = new IPEndPoint(0, 0);
			_listener = new Socket(localEP.Address.AddressFamily, SocketType.Dgram, ProtocolType.Udp);
			_listener.Bind(localEP);

			System.Threading.Thread th = new System.Threading.Thread(ReadLoop);
			th.Start();

			_port = ((IPEndPoint)_listener.LocalEndPoint).Port;
		}
	}
}
