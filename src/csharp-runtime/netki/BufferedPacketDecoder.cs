using System;

namespace Netki
{
	public delegate void OnPacketDelegate<HolderType>(ref DecodedPacket<HolderType> packet) where HolderType : PacketHolder;

	public class BufferedPacketDecoder<HolderType> : PacketDecoder<HolderType> where HolderType : PacketHolder
	{
		byte[] _data;
		PacketDecoder<HolderType> _decoder;
		int _parsepos, _readpos;
		bool _error = false;

		public BufferedPacketDecoder(int bufsize, PacketDecoder<HolderType> decoder)
		{
			_data = new byte[bufsize];
			_decoder = decoder;
			_parsepos = 0;
			_readpos = 0;
		}

		public int Decode(byte[] data, int offset, int length, ref DecodedPacket<HolderType> pkt)
		{
			int ret;

			// When data exists in queue, add on and attempt decode.
			if (_readpos > 0)
			{
				if (!Save(data, offset, length))
				{
					_error = true;
					pkt.type_id = -1;
					return length;
				}

				ret = DoDecode(_data, _parsepos, _readpos - _parsepos, ref pkt);
				if (ret > 0)
					OnParsed(ret);
				return length;
			}

			// No data in queue; attempt decode directly in buffer
			ret = DoDecode(data, offset, length, ref pkt);
			if (pkt.type_id < 0)
			{
				// No decode yet. Consume what it wants and store the rest.
				if (!Save(data, offset + ret, length - ret))
				{
					pkt.type_id = -1;
					_error = true;
				}
				return length;
			}

			return ret;
		}

		public void OnParsed(int bytes)
		{
			_parsepos += bytes;
			if (_parsepos == _readpos)
			{
				_readpos = 0;
				_parsepos = 0;
			}
		}

		DecodedPacket<HolderType> pktHolder;

		public void OnStreamData(byte[] data, int offset, int length, OnPacketDelegate<HolderType> handler)
		{
			while (true)
			{
				int ret = Decode(data, offset, length, ref pktHolder);

				offset += ret;
				length -= ret;
				if (pktHolder.type_id < 0)
					break;
				handler(ref pktHolder);
			}
		}

		public bool HasError()
		{
			return _error;
		}

		public bool Save(byte[] data, int offset, int length)
		{
			if (_readpos + length > _data.Length)
				return false;

			for (int i = 0; i < length; i++)
			{
				_data[_readpos + i] = data[offset + i];
			}
			_readpos += length;
			return true;
		}

		public int DoDecode(byte[] data, int offset, int length, ref DecodedPacket<HolderType> pkt)
		{
			return _decoder.Decode(data, offset, length, ref pkt);
		}
	}
}
