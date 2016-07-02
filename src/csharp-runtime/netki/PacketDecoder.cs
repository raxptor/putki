using System;

namespace Netki
{
	public struct DecodedPacket<HolderType> where HolderType : PacketHolder
	{
		public int type_id;
		public HolderType packet;
		public Packet GetPacket()
		{
			return packet.Box(type_id);	
		}
	}

	public interface PacketDecoder<HolderType> where HolderType : PacketHolder
	{
		// Returns number of bytes consumed. pkt.type_id >= 0 on success. pkt.type_id < 0 on failure.
		// TODO: Returns negative numbers for stream failure
		int Decode(byte[] data, int offset, int length, ref DecodedPacket<HolderType> pkt);
	}
}
