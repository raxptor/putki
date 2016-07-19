using System;

namespace Netki
{
	public delegate void PacketLaneOutput(Bitstream.Buffer send);

	public struct LanePacket
	{
		public DateTime Timestamp;
		public Bitstream.Buffer Buffer;
	}

	public interface BufferFactory
	{
		Bitstream.Buffer GetBuffer(uint minSize);
		void ReturnBuffer(Bitstream.Buffer buf);
	}
}