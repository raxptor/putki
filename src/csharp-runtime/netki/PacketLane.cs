using System;

namespace Netki
{
	public delegate void PacketLaneOutput(Bitstream.Buffer send);

	public struct LanePacket
	{
		public DateTime Timestamp;
		public Bitstream.Buffer Buffer;
	}

	public interface PacketLane
	{		
		void Incoming(Bitstream.Buffer stream, DateTime timestamp);
		void Send(Bitstream.Buffer stream);
		bool Update(float dt, PacketLaneOutput outputFn, ref LanePacket incoming);
		float ComputePacketLoss();
	}
}