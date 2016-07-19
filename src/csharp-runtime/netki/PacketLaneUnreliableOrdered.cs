using System;
using System.Collections.Generic;

namespace Netki
{
	public class PacketLaneUnreliableOrdered
	{
		Bitstream.Buffer[] _recv = new Bitstream.Buffer[256];
		List<Bitstream.Buffer> _send = new List<Bitstream.Buffer>();
		DateTime[] _timestamp = new DateTime[256];
		byte _recvHead = 0;
		byte _recvTail = 0;
		byte _sendPos = 0;
		int _recvTotal = 0;
		int _recvGaps = 0;
		BufferFactory _bufferFactory;

		public PacketLaneUnreliableOrdered(BufferFactory factory)
		{
			for (int i = 0; i < 256; i++)
				_recv[i] = new Bitstream.Buffer();
			_bufferFactory = factory;
		}
			
		public void Incoming(Bitstream.Buffer stream, DateTime timestamp)
		{
			byte seq = (byte)Bitstream.ReadBits(stream, 8);

			if (stream.error != 0)
			{
				Error("Broken packet 1");
				return;
			}

			byte diff = (byte)(seq - _recvTail);
			if (diff > 200)
			{
				// too old.
				return;
			}
			else
			{
				// update head if newer.
				byte newHead = (byte)(seq + 1);
				diff = (byte)(newHead - _recvHead);
				if (diff < 100)
					_recvHead = newHead;
			}
	
			Bitstream.SyncByte(stream);
			Bitstream.Copy(_recv[seq], stream);
			_timestamp[seq] = timestamp;
		}

		public void Send(Bitstream.Buffer stream)
		{
			Bitstream.Buffer buf = _bufferFactory.GetBuffer(stream.bytesize + 8);
			Bitstream.PutBits(buf, 8, _sendPos++);
			Bitstream.Insert(buf, stream);
            Bitstream.SyncByte(buf);
            buf.Flip();
			_send.Add(buf);
		}

		public bool Update(float dt, PacketLaneOutput outputFn, ref LanePacket incoming)
		{
			while (_recvTail != _recvHead)
			{
				if (_recv[_recvTail].buf != null)
				{
					incoming.Buffer = new Bitstream.Buffer();
					Bitstream.Copy(incoming.Buffer, _recv[_recvTail]);
					_recv[_recvTail++].buf = null;
					_recvTotal++;
					return true;
				}
				else
				{
					_recvGaps++;
					_recvTotal++;
					_recvTail++;
				}
			}

			if (dt >= 0)
			{
				foreach (Bitstream.Buffer b in _send)
				{
					outputFn(b);
				}
				_send.Clear();
			}

			return false;
		}

		public float ComputePacketLoss()
		{
			if (_recvTotal < 10)
				return 0.0f;

			return (float)_recvGaps / (float)(_recvTotal);
		}

		public void Error(string desc)
		{
			Console.WriteLine("PacketLaneUnreliableOrdered: " + desc);
		}
	}
}