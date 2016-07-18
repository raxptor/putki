using System;

namespace Netki
{
	public static class BitstreamFast
	{
		public class Buffer
		{
			public byte[] buf;
			public int bytesize;
			public int bitsize;
			public int bytepos;
			public int bitpos;
			public int error;

			public void Reset()
			{
				bitpos = bytepos = bitsize = 0;
				bytesize = buf.Length;
				error =0;
			}

			public int BitsLeft()
			{
				return 8*bytesize + bitsize - 8*bytepos - bitpos;
			}

			public static Buffer Make(byte[] buffer)
			{
				Buffer b = new Buffer();
				b.buf = buffer;
				b.bytesize = buffer.Length;
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

			BitstreamFast.Buffer tmp = new BitstreamFast.Buffer();
			Copy(tmp, source);

			while (tmp.BitsLeft() > 0)
			{
				if (tmp.bitpos > 0)
				{
					int bits = 8 - tmp.bitpos;
					if (bits > tmp.BitsLeft())
						bits = tmp.BitsLeft();
					BitstreamFast.PutBits(dest, bits, BitstreamFast.ReadBits(tmp, bits));
				}
				if (tmp.BitsLeft() > 32)
					BitstreamFast.PutBits(dest, 32, BitstreamFast.ReadBits(tmp, 32));

				int left = tmp.BitsLeft();
				if (left >= 8)
					BitstreamFast.PutBits(dest, 8, BitstreamFast.ReadBits(tmp, 8));
				else if (left > 0)
					BitstreamFast.PutBits(dest, left, BitstreamFast.ReadBits(tmp, left));
			}
			return true;
		}

		class CheatEntry
		{
			public CheatEntry(byte _byteofs, byte _count, byte _s0, byte _m0, byte _s1, byte _m1, byte _s2, byte _m2, byte _s3, byte _m3, byte _s4, byte _m4)
			{
				byteofs = _byteofs;
				count = _count;
				s0 = _s0;
				m0 = _m0;
				s1 = _s1;
				m1 = _m1;
				s2 = _s2;
				m2 = _m2;
				s3 = _s3;
				m3 = _m3;
				s4 = _s4;
				m4 = _m4;
			}
			public byte byteofs, count;
			public byte s0, m0;
			public byte s1, m1;
			public byte s2, m2;
			public byte s3, m3;
			public byte s4, m4;
		}

		static CheatEntry[] CheatTable = new CheatEntry[] {new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,0,0,0,0,0,0,0,0,0,0,0),new CheatEntry(0,1,0,1,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,2,0,0,0,0,0,0,0,0),new CheatEntry(0,1,2,4,0,0,0,0,0,0,0,0),new CheatEntry(0,1,3,8,0,0,0,0,0,0,0,0),new CheatEntry(0,1,4,16,0,0,0,0,0,0,0,0),new CheatEntry(0,1,5,32,0,0,0,0,0,0,0,0),new CheatEntry(0,1,6,64,0,0,0,0,0,0,0,0),new CheatEntry(1,1,7,128,0,0,0,0,0,0,0,0),new CheatEntry(0,1,0,3,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,6,0,0,0,0,0,0,0,0),new CheatEntry(0,1,2,12,0,0,0,0,0,0,0,0),new CheatEntry(0,1,3,24,0,0,0,0,0,0,0,0),new CheatEntry(0,1,4,48,0,0,0,0,0,0,0,0),new CheatEntry(0,1,5,96,0,0,0,0,0,0,0,0),new CheatEntry(1,1,6,192,0,0,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,1,0,0,0,0,0,0),new CheatEntry(0,1,0,7,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,14,0,0,0,0,0,0,0,0),new CheatEntry(0,1,2,28,0,0,0,0,0,0,0,0),new CheatEntry(0,1,3,56,0,0,0,0,0,0,0,0),new CheatEntry(0,1,4,112,0,0,0,0,0,0,0,0),new CheatEntry(1,1,5,224,0,0,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,1,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,3,0,0,0,0,0,0),new CheatEntry(0,1,0,15,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,30,0,0,0,0,0,0,0,0),new CheatEntry(0,1,2,60,0,0,0,0,0,0,0,0),new CheatEntry(0,1,3,120,0,0,0,0,0,0,0,0),new CheatEntry(1,1,4,240,0,0,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,1,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,3,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,7,0,0,0,0,0,0),new CheatEntry(0,1,0,31,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,62,0,0,0,0,0,0,0,0),new CheatEntry(0,1,2,124,0,0,0,0,0,0,0,0),new CheatEntry(1,1,3,248,0,0,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,1,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,3,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,7,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,15,0,0,0,0,0,0),new CheatEntry(0,1,0,63,0,0,0,0,0,0,0,0),new CheatEntry(0,1,1,126,0,0,0,0,0,0,0,0),new CheatEntry(1,1,2,252,0,0,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,1,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,3,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,7,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,15,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,31,0,0,0,0,0,0),new CheatEntry(0,1,0,127,0,0,0,0,0,0,0,0),new CheatEntry(1,1,1,254,0,0,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,1,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,3,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,7,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,15,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,31,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,63,0,0,0,0,0,0),new CheatEntry(1,1,0,255,0,0,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,1,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,3,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,7,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,15,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,31,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,63,0,0,0,0,0,0),new CheatEntry(1,2,7,128,1,127,0,0,0,0,0,0),new CheatEntry(1,2,0,255,8,1,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,3,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,7,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,15,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,31,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,63,0,0,0,0,0,0),new CheatEntry(1,2,6,192,2,127,0,0,0,0,0,0),new CheatEntry(2,2,7,128,1,255,0,0,0,0,0,0),new CheatEntry(1,2,0,255,8,3,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,7,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,15,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,31,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,63,0,0,0,0,0,0),new CheatEntry(1,2,5,224,3,127,0,0,0,0,0,0),new CheatEntry(2,2,6,192,2,255,0,0,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,1,0,0,0,0),new CheatEntry(1,2,0,255,8,7,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,15,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,31,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,63,0,0,0,0,0,0),new CheatEntry(1,2,4,240,4,127,0,0,0,0,0,0),new CheatEntry(2,2,5,224,3,255,0,0,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,1,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,3,0,0,0,0),new CheatEntry(1,2,0,255,8,15,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,31,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,63,0,0,0,0,0,0),new CheatEntry(1,2,3,248,5,127,0,0,0,0,0,0),new CheatEntry(2,2,4,240,4,255,0,0,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,1,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,3,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,7,0,0,0,0),new CheatEntry(1,2,0,255,8,31,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,63,0,0,0,0,0,0),new CheatEntry(1,2,2,252,6,127,0,0,0,0,0,0),new CheatEntry(2,2,3,248,5,255,0,0,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,1,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,3,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,7,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,15,0,0,0,0),new CheatEntry(1,2,0,255,8,63,0,0,0,0,0,0),new CheatEntry(1,2,1,254,7,127,0,0,0,0,0,0),new CheatEntry(2,2,2,252,6,255,0,0,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,1,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,3,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,7,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,15,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,31,0,0,0,0),new CheatEntry(1,2,0,255,8,127,0,0,0,0,0,0),new CheatEntry(2,2,1,254,7,255,0,0,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,1,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,3,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,7,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,15,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,31,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,63,0,0,0,0),new CheatEntry(2,2,0,255,8,255,0,0,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,1,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,3,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,7,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,15,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,31,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,63,0,0,0,0),new CheatEntry(2,3,7,128,1,255,9,127,0,0,0,0),new CheatEntry(2,3,0,255,8,255,16,1,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,3,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,7,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,15,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,31,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,63,0,0,0,0),new CheatEntry(2,3,6,192,2,255,10,127,0,0,0,0),new CheatEntry(3,3,7,128,1,255,9,255,0,0,0,0),new CheatEntry(2,3,0,255,8,255,16,3,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,7,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,15,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,31,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,63,0,0,0,0),new CheatEntry(2,3,5,224,3,255,11,127,0,0,0,0),new CheatEntry(3,3,6,192,2,255,10,255,0,0,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,1,0,0),new CheatEntry(2,3,0,255,8,255,16,7,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,15,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,31,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,63,0,0,0,0),new CheatEntry(2,3,4,240,4,255,12,127,0,0,0,0),new CheatEntry(3,3,5,224,3,255,11,255,0,0,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,1,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,3,0,0),new CheatEntry(2,3,0,255,8,255,16,15,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,31,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,63,0,0,0,0),new CheatEntry(2,3,3,248,5,255,13,127,0,0,0,0),new CheatEntry(3,3,4,240,4,255,12,255,0,0,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,1,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,3,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,7,0,0),new CheatEntry(2,3,0,255,8,255,16,31,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,63,0,0,0,0),new CheatEntry(2,3,2,252,6,255,14,127,0,0,0,0),new CheatEntry(3,3,3,248,5,255,13,255,0,0,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,1,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,3,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,7,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,15,0,0),new CheatEntry(2,3,0,255,8,255,16,63,0,0,0,0),new CheatEntry(2,3,1,254,7,255,15,127,0,0,0,0),new CheatEntry(3,3,2,252,6,255,14,255,0,0,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,1,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,3,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,7,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,15,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,31,0,0),new CheatEntry(2,3,0,255,8,255,16,127,0,0,0,0),new CheatEntry(3,3,1,254,7,255,15,255,0,0,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,1,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,3,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,7,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,15,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,31,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,63,0,0),new CheatEntry(3,3,0,255,8,255,16,255,0,0,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,1,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,3,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,7,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,15,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,31,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,63,0,0),new CheatEntry(3,4,7,128,1,255,9,255,17,127,0,0),new CheatEntry(3,4,0,255,8,255,16,255,24,1,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,3,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,7,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,15,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,31,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,63,0,0),new CheatEntry(3,4,6,192,2,255,10,255,18,127,0,0),new CheatEntry(4,4,7,128,1,255,9,255,17,255,0,0),new CheatEntry(3,4,0,255,8,255,16,255,24,3,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,7,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,15,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,31,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,63,0,0),new CheatEntry(3,4,5,224,3,255,11,255,19,127,0,0),new CheatEntry(4,4,6,192,2,255,10,255,18,255,0,0),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,1),new CheatEntry(3,4,0,255,8,255,16,255,24,7,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,15,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,31,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,63,0,0),new CheatEntry(3,4,4,240,4,255,12,255,20,127,0,0),new CheatEntry(4,4,5,224,3,255,11,255,19,255,0,0),new CheatEntry(4,5,6,192,2,255,10,255,18,255,26,1),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,3),new CheatEntry(3,4,0,255,8,255,16,255,24,15,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,31,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,63,0,0),new CheatEntry(3,4,3,248,5,255,13,255,21,127,0,0),new CheatEntry(4,4,4,240,4,255,12,255,20,255,0,0),new CheatEntry(4,5,5,224,3,255,11,255,19,255,27,1),new CheatEntry(4,5,6,192,2,255,10,255,18,255,26,3),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,7),new CheatEntry(3,4,0,255,8,255,16,255,24,31,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,63,0,0),new CheatEntry(3,4,2,252,6,255,14,255,22,127,0,0),new CheatEntry(4,4,3,248,5,255,13,255,21,255,0,0),new CheatEntry(4,5,4,240,4,255,12,255,20,255,28,1),new CheatEntry(4,5,5,224,3,255,11,255,19,255,27,3),new CheatEntry(4,5,6,192,2,255,10,255,18,255,26,7),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,15),new CheatEntry(3,4,0,255,8,255,16,255,24,63,0,0),new CheatEntry(3,4,1,254,7,255,15,255,23,127,0,0),new CheatEntry(4,4,2,252,6,255,14,255,22,255,0,0),new CheatEntry(4,5,3,248,5,255,13,255,21,255,29,1),new CheatEntry(4,5,4,240,4,255,12,255,20,255,28,3),new CheatEntry(4,5,5,224,3,255,11,255,19,255,27,7),new CheatEntry(4,5,6,192,2,255,10,255,18,255,26,15),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,31),new CheatEntry(3,4,0,255,8,255,16,255,24,127,0,0),new CheatEntry(4,4,1,254,7,255,15,255,23,255,0,0),new CheatEntry(4,5,2,252,6,255,14,255,22,255,30,1),new CheatEntry(4,5,3,248,5,255,13,255,21,255,29,3),new CheatEntry(4,5,4,240,4,255,12,255,20,255,28,7),new CheatEntry(4,5,5,224,3,255,11,255,19,255,27,15),new CheatEntry(4,5,6,192,2,255,10,255,18,255,26,31),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,63),new CheatEntry(4,4,0,255,8,255,16,255,24,255,0,0),new CheatEntry(4,5,1,254,7,255,15,255,23,255,31,1),new CheatEntry(4,5,2,252,6,255,14,255,22,255,30,3),new CheatEntry(4,5,3,248,5,255,13,255,21,255,29,7),new CheatEntry(4,5,4,240,4,255,12,255,20,255,28,15),new CheatEntry(4,5,5,224,3,255,11,255,19,255,27,31),new CheatEntry(4,5,6,192,2,255,10,255,18,255,26,63),new CheatEntry(4,5,7,128,1,255,9,255,17,255,25,127)};

		public static void PutBitsU(Buffer buf, int bits, UInt32 value)
		{
			CheatEntry ce = CheatTable[bits * 8 + buf.bitpos];
			byte first = (byte)(((byte)value << ce.s0) & ce.m0);
			int dpos = buf.bytepos;
			byte[] data = buf.buf;
			byte old = 0;
			if (buf.bitpos != 0)
				old = data[dpos];
			buf.bitpos = (int)((buf.bitpos + bits) & 7);
			buf.bytepos = buf.bytepos + ce.byteofs;
			data[dpos] = (byte)(old | first);
			if (ce.count == 1)
				return;
			data[dpos+1] = (byte)((value >> ce.s1) & ce.m1);
			if (ce.count == 2)
				return;
			data[dpos+2] = (byte)((value >> ce.s2) & ce.m2);
			if (ce.count == 3)
				return;
			data[dpos+3] = (byte)((value >> ce.s3) & ce.m3);
			if (ce.count == 4)
				return;
			data[dpos+4] = (byte)((value >> ce.s4) & ce.m4);
		}

		public static bool PutBits(Buffer buf, int bits, UInt32 value)
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
                BitstreamFast.PutBits(buf, 8, data[i]);
            return buf.error != 0;
		}
	
		public static void PutCompressedUint(Buffer buf, uint value)
		{
			int bits = 0, prefixes = 0;
			
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
				PutBits(buf, prefixes, 0);
			if (prefixes != 6)
				PutBits(buf, 1, 1);
			if (bits > 0)
				PutBits(buf, bits, value);
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
		
		public static UInt32 ReadBits(Buffer buf, int bits)
		{
			if (buf.BitsLeft() < bits)
			{
				buf.error = 1;
				return 0;
			}

			CheatEntry ce = CheatTable[bits * 8 + buf.bitpos];
			uint value = 0;

			int dpos = buf.bytepos;
			byte[] data = buf.buf;
			buf.bitpos = (int)((buf.bitpos + bits) & 7);
			buf.bytepos = buf.bytepos + ce.byteofs;

			value = ((uint)data[dpos] & ce.m0) >> ce.s0;
			if (ce.count == 1)
				return value;
			value = value | ((uint)(data[dpos+1] & ce.m1) << ce.s1);
			if (ce.count == 2)
				return value;
			value = value | ((uint)(data[dpos+2] & ce.m2) << ce.s2);
			if (ce.count == 3)
				return value;
			value = value | ((uint)(data[dpos+3] & ce.m3) << ce.s3);
			if (ce.count == 4)
				return value;
			return value | ((uint)(data[dpos+4] & ce.m4) << ce.s4);
		}

		public static byte[] ReadBytes(Buffer buf, int count)
		{
			byte[] dst = new byte[count];
            for (int i=0;i<count;i++)
                dst[i] = (byte)BitstreamFast.ReadBits(buf, 8);
			return dst;
		}

		public static void PutStringDumb(Buffer buf, string value)
		{
			if (value == null)
			{
				BitstreamFast.PutCompressedInt(buf, -1);
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