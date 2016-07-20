using System;
using System.Collections.Generic;

namespace Netki
{
	public static class Bitstream
	{
		public class Buffer
		{
			public byte[] buf;
			public uint bytesize;
			public uint bytepos;
			public byte bitsize;
			public byte bitpos;
			public byte error;
			public uint userdata;

			public void Reset()
			{
				error = bitpos = bitsize = 0;
				bytepos = 0;
				bytesize = (uint)buf.Length;
				userdata = 0;
				error = 0;
			}

			public uint BitsLeft()
			{
				return 8*bytesize + bitsize - 8*bytepos - bitpos;
			}

			public static Buffer Make(byte[] buffer)
			{
				Buffer b = new Buffer();
				b.buf = buffer;
				b.bytesize = (uint)buffer.Length;
				return b;
			}

			public void Flip()
			{
				error = 0;
				bytesize = bytepos;
				bitsize = bitpos;
				bytepos = 0;
				bitpos = 0;
			}
		}

		public static void Copy(Buffer dst, Buffer src)
		{
			dst.buf = src.buf;
			dst.bytesize = src.bytesize;
			dst.bitsize = src.bitsize;
			dst.bytepos = src.bytepos;
			dst.bitpos = src.bitpos;
			dst.error = src.error;
		}

		public static bool Insert(Buffer dest, Buffer source)
		{
			if (dest.BitsLeft() < source.BitsLeft())
				return false;

			Bitstream.Buffer tmp = new Bitstream.Buffer();
			Copy(tmp, source);

			while (tmp.BitsLeft() > 0)
			{
				if (tmp.bitpos > 0)
				{
					uint bits = (uint)(8 - tmp.bitpos);
					if (bits > tmp.BitsLeft())
					{
						bits = tmp.BitsLeft();
					}
					Bitstream.PutBits(dest, bits, Bitstream.ReadBits(tmp, bits));
				}
				if (tmp.BitsLeft() > 32)
					Bitstream.PutBits(dest, 32, Bitstream.ReadBits(tmp, 32));

				uint left = tmp.BitsLeft();
				if (left >= 8)
				{
					Bitstream.PutBits(dest, 8, Bitstream.ReadBits(tmp, 8));
				}
				else if (left > 0)
				{
					Bitstream.PutBits(dest, left, Bitstream.ReadBits(tmp, left));
				}
			}
			return true;
		}

		class CheatEntry
		{
			public CheatEntry(byte _byteofs, byte _bitpos, byte _count, byte _s0, byte _s1, byte _s2, byte _s3, byte _s4, byte _mfirst, byte _mlast)
			{
				byteofs = _byteofs;
				bitpos = _bitpos;
				count = _count;
				s0 = _s0;
				s1 = _s1;
				s2 = _s2;
				s3 = _s3;
				s4 = _s4;
				mfirst = _mfirst;
				mlast = _mlast;
			}
			public byte byteofs, bitpos, count;
			public byte s0, s1;
			public byte s2, s3;
			public byte s4;
			public byte mfirst, mlast;
		}

		static CheatEntry[] CheatTable = new CheatEntry[] {new CheatEntry(0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,1,0,0,0,0,0,0,0,0),new CheatEntry(0,2,0,0,0,0,0,0,0,0),new CheatEntry(0,3,0,0,0,0,0,0,0,0),new CheatEntry(0,4,0,0,0,0,0,0,0,0),new CheatEntry(0,5,0,0,0,0,0,0,0,0),new CheatEntry(0,6,0,0,0,0,0,0,0,0),new CheatEntry(0,7,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,0,0,0,0,0,1,1),new CheatEntry(0,2,1,1,0,0,0,0,2,2),new CheatEntry(0,3,1,2,0,0,0,0,4,4),new CheatEntry(0,4,1,3,0,0,0,0,8,8),new CheatEntry(0,5,1,4,0,0,0,0,16,16),new CheatEntry(0,6,1,5,0,0,0,0,32,32),new CheatEntry(0,7,1,6,0,0,0,0,64,64),new CheatEntry(1,0,1,7,0,0,0,0,128,128),new CheatEntry(0,2,1,0,0,0,0,0,3,3),new CheatEntry(0,3,1,1,0,0,0,0,6,6),new CheatEntry(0,4,1,2,0,0,0,0,12,12),new CheatEntry(0,5,1,3,0,0,0,0,24,24),new CheatEntry(0,6,1,4,0,0,0,0,48,48),new CheatEntry(0,7,1,5,0,0,0,0,96,96),new CheatEntry(1,0,1,6,0,0,0,0,192,192),new CheatEntry(1,1,2,7,1,0,0,0,128,1),new CheatEntry(0,3,1,0,0,0,0,0,7,7),new CheatEntry(0,4,1,1,0,0,0,0,14,14),new CheatEntry(0,5,1,2,0,0,0,0,28,28),new CheatEntry(0,6,1,3,0,0,0,0,56,56),new CheatEntry(0,7,1,4,0,0,0,0,112,112),new CheatEntry(1,0,1,5,0,0,0,0,224,224),new CheatEntry(1,1,2,6,2,0,0,0,192,1),new CheatEntry(1,2,2,7,1,0,0,0,128,3),new CheatEntry(0,4,1,0,0,0,0,0,15,15),new CheatEntry(0,5,1,1,0,0,0,0,30,30),new CheatEntry(0,6,1,2,0,0,0,0,60,60),new CheatEntry(0,7,1,3,0,0,0,0,120,120),new CheatEntry(1,0,1,4,0,0,0,0,240,240),new CheatEntry(1,1,2,5,3,0,0,0,224,1),new CheatEntry(1,2,2,6,2,0,0,0,192,3),new CheatEntry(1,3,2,7,1,0,0,0,128,7),new CheatEntry(0,5,1,0,0,0,0,0,31,31),new CheatEntry(0,6,1,1,0,0,0,0,62,62),new CheatEntry(0,7,1,2,0,0,0,0,124,124),new CheatEntry(1,0,1,3,0,0,0,0,248,248),new CheatEntry(1,1,2,4,4,0,0,0,240,1),new CheatEntry(1,2,2,5,3,0,0,0,224,3),new CheatEntry(1,3,2,6,2,0,0,0,192,7),new CheatEntry(1,4,2,7,1,0,0,0,128,15),new CheatEntry(0,6,1,0,0,0,0,0,63,63),new CheatEntry(0,7,1,1,0,0,0,0,126,126),new CheatEntry(1,0,1,2,0,0,0,0,252,252),new CheatEntry(1,1,2,3,5,0,0,0,248,1),new CheatEntry(1,2,2,4,4,0,0,0,240,3),new CheatEntry(1,3,2,5,3,0,0,0,224,7),new CheatEntry(1,4,2,6,2,0,0,0,192,15),new CheatEntry(1,5,2,7,1,0,0,0,128,31),new CheatEntry(0,7,1,0,0,0,0,0,127,127),new CheatEntry(1,0,1,1,0,0,0,0,254,254),new CheatEntry(1,1,2,2,6,0,0,0,252,1),new CheatEntry(1,2,2,3,5,0,0,0,248,3),new CheatEntry(1,3,2,4,4,0,0,0,240,7),new CheatEntry(1,4,2,5,3,0,0,0,224,15),new CheatEntry(1,5,2,6,2,0,0,0,192,31),new CheatEntry(1,6,2,7,1,0,0,0,128,63),new CheatEntry(1,0,1,0,0,0,0,0,255,255),new CheatEntry(1,1,2,1,7,0,0,0,254,1),new CheatEntry(1,2,2,2,6,0,0,0,252,3),new CheatEntry(1,3,2,3,5,0,0,0,248,7),new CheatEntry(1,4,2,4,4,0,0,0,240,15),new CheatEntry(1,5,2,5,3,0,0,0,224,31),new CheatEntry(1,6,2,6,2,0,0,0,192,63),new CheatEntry(1,7,2,7,1,0,0,0,128,127),new CheatEntry(1,1,2,0,8,0,0,0,255,1),new CheatEntry(1,2,2,1,7,0,0,0,254,3),new CheatEntry(1,3,2,2,6,0,0,0,252,7),new CheatEntry(1,4,2,3,5,0,0,0,248,15),new CheatEntry(1,5,2,4,4,0,0,0,240,31),new CheatEntry(1,6,2,5,3,0,0,0,224,63),new CheatEntry(1,7,2,6,2,0,0,0,192,127),new CheatEntry(2,0,2,7,1,0,0,0,128,255),new CheatEntry(1,2,2,0,8,0,0,0,255,3),new CheatEntry(1,3,2,1,7,0,0,0,254,7),new CheatEntry(1,4,2,2,6,0,0,0,252,15),new CheatEntry(1,5,2,3,5,0,0,0,248,31),new CheatEntry(1,6,2,4,4,0,0,0,240,63),new CheatEntry(1,7,2,5,3,0,0,0,224,127),new CheatEntry(2,0,2,6,2,0,0,0,192,255),new CheatEntry(2,1,3,7,1,9,0,0,128,1),new CheatEntry(1,3,2,0,8,0,0,0,255,7),new CheatEntry(1,4,2,1,7,0,0,0,254,15),new CheatEntry(1,5,2,2,6,0,0,0,252,31),new CheatEntry(1,6,2,3,5,0,0,0,248,63),new CheatEntry(1,7,2,4,4,0,0,0,240,127),new CheatEntry(2,0,2,5,3,0,0,0,224,255),new CheatEntry(2,1,3,6,2,10,0,0,192,1),new CheatEntry(2,2,3,7,1,9,0,0,128,3),new CheatEntry(1,4,2,0,8,0,0,0,255,15),new CheatEntry(1,5,2,1,7,0,0,0,254,31),new CheatEntry(1,6,2,2,6,0,0,0,252,63),new CheatEntry(1,7,2,3,5,0,0,0,248,127),new CheatEntry(2,0,2,4,4,0,0,0,240,255),new CheatEntry(2,1,3,5,3,11,0,0,224,1),new CheatEntry(2,2,3,6,2,10,0,0,192,3),new CheatEntry(2,3,3,7,1,9,0,0,128,7),new CheatEntry(1,5,2,0,8,0,0,0,255,31),new CheatEntry(1,6,2,1,7,0,0,0,254,63),new CheatEntry(1,7,2,2,6,0,0,0,252,127),new CheatEntry(2,0,2,3,5,0,0,0,248,255),new CheatEntry(2,1,3,4,4,12,0,0,240,1),new CheatEntry(2,2,3,5,3,11,0,0,224,3),new CheatEntry(2,3,3,6,2,10,0,0,192,7),new CheatEntry(2,4,3,7,1,9,0,0,128,15),new CheatEntry(1,6,2,0,8,0,0,0,255,63),new CheatEntry(1,7,2,1,7,0,0,0,254,127),new CheatEntry(2,0,2,2,6,0,0,0,252,255),new CheatEntry(2,1,3,3,5,13,0,0,248,1),new CheatEntry(2,2,3,4,4,12,0,0,240,3),new CheatEntry(2,3,3,5,3,11,0,0,224,7),new CheatEntry(2,4,3,6,2,10,0,0,192,15),new CheatEntry(2,5,3,7,1,9,0,0,128,31),new CheatEntry(1,7,2,0,8,0,0,0,255,127),new CheatEntry(2,0,2,1,7,0,0,0,254,255),new CheatEntry(2,1,3,2,6,14,0,0,252,1),new CheatEntry(2,2,3,3,5,13,0,0,248,3),new CheatEntry(2,3,3,4,4,12,0,0,240,7),new CheatEntry(2,4,3,5,3,11,0,0,224,15),new CheatEntry(2,5,3,6,2,10,0,0,192,31),new CheatEntry(2,6,3,7,1,9,0,0,128,63),new CheatEntry(2,0,2,0,8,0,0,0,255,255),new CheatEntry(2,1,3,1,7,15,0,0,254,1),new CheatEntry(2,2,3,2,6,14,0,0,252,3),new CheatEntry(2,3,3,3,5,13,0,0,248,7),new CheatEntry(2,4,3,4,4,12,0,0,240,15),new CheatEntry(2,5,3,5,3,11,0,0,224,31),new CheatEntry(2,6,3,6,2,10,0,0,192,63),new CheatEntry(2,7,3,7,1,9,0,0,128,127),new CheatEntry(2,1,3,0,8,16,0,0,255,1),new CheatEntry(2,2,3,1,7,15,0,0,254,3),new CheatEntry(2,3,3,2,6,14,0,0,252,7),new CheatEntry(2,4,3,3,5,13,0,0,248,15),new CheatEntry(2,5,3,4,4,12,0,0,240,31),new CheatEntry(2,6,3,5,3,11,0,0,224,63),new CheatEntry(2,7,3,6,2,10,0,0,192,127),new CheatEntry(3,0,3,7,1,9,0,0,128,255),new CheatEntry(2,2,3,0,8,16,0,0,255,3),new CheatEntry(2,3,3,1,7,15,0,0,254,7),new CheatEntry(2,4,3,2,6,14,0,0,252,15),new CheatEntry(2,5,3,3,5,13,0,0,248,31),new CheatEntry(2,6,3,4,4,12,0,0,240,63),new CheatEntry(2,7,3,5,3,11,0,0,224,127),new CheatEntry(3,0,3,6,2,10,0,0,192,255),new CheatEntry(3,1,4,7,1,9,17,0,128,1),new CheatEntry(2,3,3,0,8,16,0,0,255,7),new CheatEntry(2,4,3,1,7,15,0,0,254,15),new CheatEntry(2,5,3,2,6,14,0,0,252,31),new CheatEntry(2,6,3,3,5,13,0,0,248,63),new CheatEntry(2,7,3,4,4,12,0,0,240,127),new CheatEntry(3,0,3,5,3,11,0,0,224,255),new CheatEntry(3,1,4,6,2,10,18,0,192,1),new CheatEntry(3,2,4,7,1,9,17,0,128,3),new CheatEntry(2,4,3,0,8,16,0,0,255,15),new CheatEntry(2,5,3,1,7,15,0,0,254,31),new CheatEntry(2,6,3,2,6,14,0,0,252,63),new CheatEntry(2,7,3,3,5,13,0,0,248,127),new CheatEntry(3,0,3,4,4,12,0,0,240,255),new CheatEntry(3,1,4,5,3,11,19,0,224,1),new CheatEntry(3,2,4,6,2,10,18,0,192,3),new CheatEntry(3,3,4,7,1,9,17,0,128,7),new CheatEntry(2,5,3,0,8,16,0,0,255,31),new CheatEntry(2,6,3,1,7,15,0,0,254,63),new CheatEntry(2,7,3,2,6,14,0,0,252,127),new CheatEntry(3,0,3,3,5,13,0,0,248,255),new CheatEntry(3,1,4,4,4,12,20,0,240,1),new CheatEntry(3,2,4,5,3,11,19,0,224,3),new CheatEntry(3,3,4,6,2,10,18,0,192,7),new CheatEntry(3,4,4,7,1,9,17,0,128,15),new CheatEntry(2,6,3,0,8,16,0,0,255,63),new CheatEntry(2,7,3,1,7,15,0,0,254,127),new CheatEntry(3,0,3,2,6,14,0,0,252,255),new CheatEntry(3,1,4,3,5,13,21,0,248,1),new CheatEntry(3,2,4,4,4,12,20,0,240,3),new CheatEntry(3,3,4,5,3,11,19,0,224,7),new CheatEntry(3,4,4,6,2,10,18,0,192,15),new CheatEntry(3,5,4,7,1,9,17,0,128,31),new CheatEntry(2,7,3,0,8,16,0,0,255,127),new CheatEntry(3,0,3,1,7,15,0,0,254,255),new CheatEntry(3,1,4,2,6,14,22,0,252,1),new CheatEntry(3,2,4,3,5,13,21,0,248,3),new CheatEntry(3,3,4,4,4,12,20,0,240,7),new CheatEntry(3,4,4,5,3,11,19,0,224,15),new CheatEntry(3,5,4,6,2,10,18,0,192,31),new CheatEntry(3,6,4,7,1,9,17,0,128,63),new CheatEntry(3,0,3,0,8,16,0,0,255,255),new CheatEntry(3,1,4,1,7,15,23,0,254,1),new CheatEntry(3,2,4,2,6,14,22,0,252,3),new CheatEntry(3,3,4,3,5,13,21,0,248,7),new CheatEntry(3,4,4,4,4,12,20,0,240,15),new CheatEntry(3,5,4,5,3,11,19,0,224,31),new CheatEntry(3,6,4,6,2,10,18,0,192,63),new CheatEntry(3,7,4,7,1,9,17,0,128,127),new CheatEntry(3,1,4,0,8,16,24,0,255,1),new CheatEntry(3,2,4,1,7,15,23,0,254,3),new CheatEntry(3,3,4,2,6,14,22,0,252,7),new CheatEntry(3,4,4,3,5,13,21,0,248,15),new CheatEntry(3,5,4,4,4,12,20,0,240,31),new CheatEntry(3,6,4,5,3,11,19,0,224,63),new CheatEntry(3,7,4,6,2,10,18,0,192,127),new CheatEntry(4,0,4,7,1,9,17,0,128,255),new CheatEntry(3,2,4,0,8,16,24,0,255,3),new CheatEntry(3,3,4,1,7,15,23,0,254,7),new CheatEntry(3,4,4,2,6,14,22,0,252,15),new CheatEntry(3,5,4,3,5,13,21,0,248,31),new CheatEntry(3,6,4,4,4,12,20,0,240,63),new CheatEntry(3,7,4,5,3,11,19,0,224,127),new CheatEntry(4,0,4,6,2,10,18,0,192,255),new CheatEntry(4,1,5,7,1,9,17,25,128,1),new CheatEntry(3,3,4,0,8,16,24,0,255,7),new CheatEntry(3,4,4,1,7,15,23,0,254,15),new CheatEntry(3,5,4,2,6,14,22,0,252,31),new CheatEntry(3,6,4,3,5,13,21,0,248,63),new CheatEntry(3,7,4,4,4,12,20,0,240,127),new CheatEntry(4,0,4,5,3,11,19,0,224,255),new CheatEntry(4,1,5,6,2,10,18,26,192,1),new CheatEntry(4,2,5,7,1,9,17,25,128,3),new CheatEntry(3,4,4,0,8,16,24,0,255,15),new CheatEntry(3,5,4,1,7,15,23,0,254,31),new CheatEntry(3,6,4,2,6,14,22,0,252,63),new CheatEntry(3,7,4,3,5,13,21,0,248,127),new CheatEntry(4,0,4,4,4,12,20,0,240,255),new CheatEntry(4,1,5,5,3,11,19,27,224,1),new CheatEntry(4,2,5,6,2,10,18,26,192,3),new CheatEntry(4,3,5,7,1,9,17,25,128,7),new CheatEntry(3,5,4,0,8,16,24,0,255,31),new CheatEntry(3,6,4,1,7,15,23,0,254,63),new CheatEntry(3,7,4,2,6,14,22,0,252,127),new CheatEntry(4,0,4,3,5,13,21,0,248,255),new CheatEntry(4,1,5,4,4,12,20,28,240,1),new CheatEntry(4,2,5,5,3,11,19,27,224,3),new CheatEntry(4,3,5,6,2,10,18,26,192,7),new CheatEntry(4,4,5,7,1,9,17,25,128,15),new CheatEntry(3,6,4,0,8,16,24,0,255,63),new CheatEntry(3,7,4,1,7,15,23,0,254,127),new CheatEntry(4,0,4,2,6,14,22,0,252,255),new CheatEntry(4,1,5,3,5,13,21,29,248,1),new CheatEntry(4,2,5,4,4,12,20,28,240,3),new CheatEntry(4,3,5,5,3,11,19,27,224,7),new CheatEntry(4,4,5,6,2,10,18,26,192,15),new CheatEntry(4,5,5,7,1,9,17,25,128,31),new CheatEntry(3,7,4,0,8,16,24,0,255,127),new CheatEntry(4,0,4,1,7,15,23,0,254,255),new CheatEntry(4,1,5,2,6,14,22,30,252,1),new CheatEntry(4,2,5,3,5,13,21,29,248,3),new CheatEntry(4,3,5,4,4,12,20,28,240,7),new CheatEntry(4,4,5,5,3,11,19,27,224,15),new CheatEntry(4,5,5,6,2,10,18,26,192,31),new CheatEntry(4,6,5,7,1,9,17,25,128,63),new CheatEntry(4,0,4,0,8,16,24,0,255,255),new CheatEntry(4,1,5,1,7,15,23,31,254,1),new CheatEntry(4,2,5,2,6,14,22,30,252,3),new CheatEntry(4,3,5,3,5,13,21,29,248,7),new CheatEntry(4,4,5,4,4,12,20,28,240,15),new CheatEntry(4,5,5,5,3,11,19,27,224,31),new CheatEntry(4,6,5,6,2,10,18,26,192,63),new CheatEntry(4,7,5,7,1,9,17,25,128,127)};

		// Without any error checking
		public static void PutBitsU(Buffer buf, uint bits, uint value)
		{
			CheatEntry ce = CheatTable[bits * 8 + buf.bitpos];
			uint dpos = buf.bytepos;
			byte[] data = buf.buf;
			
			if (buf.bitpos != 0)
				data[dpos] = (byte)(data[dpos] | ((value << ce.s0) & ce.mfirst));
			else
				data[dpos] = (byte)((value << ce.s0) & ce.mfirst);

			buf.bitpos = ce.bitpos;
			buf.bytepos = dpos + ce.byteofs;

			if (ce.count == 2)
			{
				data[dpos+1] = (byte)((value >> ce.s1) & ce.mlast);
			}
			else if (ce.count == 3)
			{
				data[dpos+1] = (byte)(value >> ce.s1);
				data[dpos+2] = (byte)((value >> ce.s2) & ce.mlast);
			}
			else if (ce.count == 4)
			{
				data[dpos+1] = (byte)(value >> ce.s1);
				data[dpos+2] = (byte)(value >> ce.s2);
				data[dpos+3] = (byte)((value >> ce.s3) & ce.mlast);
			}
			else if (ce.count == 5)
			{
				data[dpos+1] = (byte)(value >> ce.s1);
				data[dpos+2] = (byte)(value >> ce.s2);
				data[dpos+3] = (byte)(value >> ce.s3);
				data[dpos+4] = (byte)((value >> ce.s4) & ce.mlast);
			}
		}

		public static bool PutBits(Buffer buf, uint bits, uint value)
		{
			if (buf.BitsLeft() < bits)
			{
				buf.error = 1;
				return false;
			}

			PutBitsU(buf, bits, value);
			return true;
		}

		public static bool PutBytes(Buffer buf, byte[] data)
		{
            for (int i=0;i<data.Length;i++)
                Bitstream.PutBits(buf, 8, data[i]);
            return buf.error != 0;
		}
	
		public static void PutCompressedUint(Buffer buf, uint value)
		{
			uint bits = 0;
			uint prefixes = 0;
			
			if (value == 0xffffffff)
			{
				prefixes = 6;
				bits = 0;
			}
			else if (value > 0xffff) 
			{
				bits = 32;
				prefixes = 5;
			}
			else if (value > 0xfff)
			{
				bits = 16;
				prefixes = 4;
			}
			else if (value > 0xff)
			{
				bits = 12;
				prefixes = 3;
			}
			else if (value > 0xf)
			{
				bits = 8;
				prefixes = 2;
			}
			else if (value > 0)
			{
				bits = 4;
				prefixes = 1;
			}
			
			if (prefixes > 0)
			{
				PutBits(buf, prefixes, 0);
			}
			if (prefixes != 6)
			{
				PutBits(buf, 1, 1);
			}
			if (bits > 0)
			{
				PutBits(buf, bits, value);
			}
		}

		public static uint ReadCompressedUint(Buffer buf)
		{
			if (ReadBits(buf, 1) == 1)
				return 0;
			if (ReadBits(buf, 1) == 1)
				return ReadBits(buf, 4);
			if (ReadBits(buf, 1) == 1)
				return ReadBits(buf, 8);
			if (ReadBits(buf, 1) == 1)
				return ReadBits(buf, 12);
			if (ReadBits(buf, 1) == 1)
				return ReadBits(buf, 16);
			if (ReadBits(buf, 1) == 1)
				return ReadBits(buf, 32);
			return 0xffffffff;
		}

		public static void PutCompressedInt(Buffer buf, int value)
		{
			if (value < 0)
			{
				PutBits(buf, 1, 1);
				PutCompressedUint(buf, (uint)(-value));
			}
			else
			{
				PutBits(buf, 1, 0);
				PutCompressedUint(buf, (uint)value);
			}
		}

		public static int ReadCompressedInt(Buffer buf)
		{
			if (ReadBits(buf, 1) == 0)
				return (int)ReadCompressedUint(buf);
			else
		        return (int)-ReadCompressedUint(buf);
		}
		
		public static float ReadFloat(Buffer buf)
		{
			if (buf.BitsLeft () < 32) {
				return 0;
			}
						
			byte[] tmp = new byte[4];
			for (int i = 0; i != 4; i++) {
				tmp[i] = (byte) ReadBits(buf, 8);
			}

			float[] f = new float[1];
			System.Buffer.BlockCopy(tmp, 0, f, 0, 4);
			return f[0];
		}
		
		public static void PutFloat(Buffer buf, float value)
		{
			float[] f = new float[1] { value };
			byte[] v = new byte[4];
			f[0] = value;
			System.Buffer.BlockCopy(f, 0, v, 0, 4);
			for (int i = 0; i != 4; i++)
				PutBits(buf, 8, v[i]);
		}
		
		public static UInt32 ReadBits(Buffer buf, uint bits)
		{
			if (buf.BitsLeft() < bits)
			{
				buf.error = 1;
				return 0;
			}
			CheatEntry ce = CheatTable[bits * 8 + buf.bitpos];
			uint dpos = buf.bytepos;
			byte[] data = buf.buf;
			buf.bitpos = ce.bitpos;
			buf.bytepos = dpos + ce.byteofs;
			uint value = ((uint)data[dpos] & ce.mfirst) >> ce.s0;
			if (ce.count == 2)
			{ 
				return value | ((uint)(data[dpos+1] & ce.mlast) << ce.s1);
			}
			else if (ce.count == 3)
			{ 
				return value | ((uint)(data[dpos+1]) << ce.s1)
				             | ((uint)(data[dpos+2] & ce.mlast) << ce.s2);
			}
			else if (ce.count == 4)
			{ 
				return value | ((uint)(data[dpos+1]) << ce.s1)
				             | ((uint)(data[dpos+2]) << ce.s2)
				             | ((uint)(data[dpos+3] & ce.mlast) << ce.s3);
			}
			else if (ce.count == 5)
			{ 
				return value | ((uint)(data[dpos+1]) << ce.s1)
				             | ((uint)(data[dpos+2]) << ce.s2)
				             | ((uint)(data[dpos+3]) << ce.s3)
				             | ((uint)(data[dpos+4] & ce.mlast) << ce.s4);
			}
			return value;
		}

		public static byte[] ReadBytes(Buffer buf, int count)
		{
			byte[] dst = new byte[count];
            for (int i=0;i<count;i++)
                dst[i] = (byte)Bitstream.ReadBits(buf, 8);
			return dst;
		}

		public static void PutStringDumb(Buffer buf, string value)
		{
			if (value == null)
			{
				Bitstream.PutCompressedInt(buf, -1);
				return;
			}

			byte[] bytes = System.Text.UTF8Encoding.UTF8.GetBytes(value);
			PutCompressedInt(buf, bytes.Length);
			PutBytes(buf, bytes);
		}

		public static string ReadStringDumb(Buffer buf)
		{
			int len = ReadCompressedInt(buf);
			if (len > 65536)
			{
				buf.error = 4;
				return null;
			}
			if (len < 0)
			{
				return null;
			}

			byte[] data = ReadBytes(buf, len);
			if (data == null || buf.error != 0)
			{
				return null;
			}
			return System.Text.UTF8Encoding.UTF8.GetString(data);
		}

        public static void SyncByte(Buffer buf)
        {
            if (buf.bitpos > 0)
            {
                buf.bitpos = 0;
                buf.bytepos++;
            }
        }
    }
}