using System;

namespace Netki
{
	public interface Packet
	{
		int GetTypeId();
		bool Write(Bitstream.Buffer buf);
		bool Read(Bitstream.Buffer buf);
	}

	public interface PacketHolder
	{
		Packet Box(int type_id);
	}
}
