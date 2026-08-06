// Reader for the binary package format written by the Rust pipeline.
// doc/package-format.md is the normative definition; keep the two in step.

using System;
using System.Collections.Generic;
using System.Text;

namespace Putki
{
	// Implemented by generated code.
	public interface TypeLoader
	{
		object ResolveFromPackage(int type, object obj, Putki.Package pkg);
		object LoadFromPackage(int type, Putki.PackageReader reader);
	}

	public class PackageFormatException : Exception
	{
		public PackageFormatException(string why) : base(why) { }
	}

	// Reads the primitive encoding described in doc/package-format.md. All
	// little-endian, no alignment, everything inline.
	public class PackageReader
	{
		static UTF8Encoding enc = new UTF8Encoding();
		byte[] data;
		int pos;

		public PackageReader(byte[] _data)
		{
			data = _data;
			pos = 0;
		}

		public PackageReader(byte[] _data, int _pos)
		{
			data = _data;
			pos = _pos;
		}

		public int GetPosition()
		{
			return pos;
		}

		public void SeekTo(int _pos)
		{
			Need(_pos - pos);
			pos = _pos;
		}

		// Package data is not trusted: a truncated file must fail cleanly rather
		// than read past the end.
		void Need(int bytes)
		{
			if (bytes < 0 || pos + bytes > data.Length || pos + bytes < pos)
				throw new PackageFormatException("package ended mid-field");
		}

		public byte ReadByte()
		{
			Need(1);
			return data[pos++];
		}

		public bool ReadBool()
		{
			return ReadByte() != 0;
		}

		public ushort ReadUInt16()
		{
			Need(2);
			ushort v = (ushort)(data[pos] | (data[pos + 1] << 8));
			pos += 2;
			return v;
		}

		public int ReadInt32()
		{
			Need(4);
			int v = data[pos] | (data[pos + 1] << 8) | (data[pos + 2] << 16) | (data[pos + 3] << 24);
			pos += 4;
			return v;
		}

		public uint ReadUInt32()
		{
			return (uint)ReadInt32();
		}

		// 8 byte field carrying a 32 bit value; a non-zero high word means the
		// file is not what we think it is.
		public int ReadUSize()
		{
			uint low = ReadUInt32();
			if (ReadUInt32() != 0)
				throw new PackageFormatException("oversized length field");
			if (low > int.MaxValue)
				throw new PackageFormatException("length does not fit in an int");
			return (int)low;
		}

		public float ReadFloat()
		{
			Need(4);
			float f = System.BitConverter.ToSingle(data, pos);
			pos += 4;
			return f;
		}

		// usize length then that many raw UTF-8 bytes, not NUL terminated.
		public string ReadString()
		{
			int len = ReadUSize();
			Need(len);
			string s = enc.GetString(data, pos, len);
			pos += len;
			return s;
		}

		// Pointer field: -1 is null, otherwise a zero-based slot index.
		public int ReadSlotRef()
		{
			return ReadInt32();
		}
	}

	public class Package
	{
		const uint MAGIC = 0x494B5450;   // 'PTKI'
		const uint VERSION = 1;
		const int HEADER_FIXED = 16;

		const int FLAG_PATH = 1;
		const int FLAG_INTERNAL = 2;

		public class Slot
		{
			public string path;
			public object inst;
			public int type;
			public int flags;
			public int begin;
			public int end;
		};

		public Slot[] m_slots;
		Dictionary<int, string> m_typeNames;
		List<Package> m_extRefs = null;
		List<string> m_unresolved = null;
		bool m_gotUnresolved = false;
		Dictionary<object, string> m_paths;

		public string TypeName(int type)
		{
			string name;
			if (m_typeNames != null && m_typeNames.TryGetValue(type, out name))
				return name;
			return "<unknown>";
		}

		public List<string> TryResolveWithRefs(List<Package> extRefs, TypeLoader loader)
		{
			m_extRefs = extRefs;
			m_unresolved = new List<string>();
			for (int i = 0;i < m_slots.Length;i++)
			{
				m_slots[i].inst = loader.ResolveFromPackage(m_slots[i].type, m_slots[i].inst, this);
			}
			List<string> r = m_unresolved;
			m_unresolved = null;
			m_extRefs = null;
			return r;
		}

		public string RootObjPath()
		{
			return m_slots[0].path;
		}

		// Slot references are as they appear in the file: -1 is null, anything
		// else is a zero-based index.
		public Type ResolveSlot<Type>(int index)
		{
			if (index < 0 || index >= m_slots.Length)
			{
				return default(Type);
			}
			// Resolve by path instead when we are cross-referencing packages and
			// the object has one.
			if (m_extRefs != null && m_slots[index].path != null && m_slots[index].path.Length > 0)
			{
				return (Type)Resolve(m_slots[index].path);
			}
			return (Type)m_slots[index].inst;
		}

		public string PathOf(object obj)
		{
			string path;
			if (m_paths != null && m_paths.TryGetValue(obj, out path))
				return path;
			return null;
		}

		public object Resolve(string path)
		{
			if (m_extRefs != null)
			{
				foreach (Package p in m_extRefs)
				{
					foreach (Slot s in p.m_slots)
					{
						if (s.inst != null && s.path == path)
						{
							return s.inst;
						}
					}
				}
			}
			foreach (Slot s in m_slots)
			{
				if (s.inst != null && s.path == path)
				{
					return s.inst;
				}
			}
			if (path == "")
			{
				return null;
			}
			m_gotUnresolved = true;
			if (m_unresolved != null)
			{
				m_unresolved.Add(path);
			}
			return null;
		}

		public bool LoadFromBytes(byte[] data, TypeLoader loader)
		{
			try
			{
				return Parse(data, loader);
			}
			catch (PackageFormatException)
			{
				m_slots = null;
				return false;
			}
		}

		bool Parse(byte[] data, TypeLoader loader)
		{
			PackageReader rdr = new PackageReader(data);

			if (rdr.ReadUInt32() != MAGIC)
				return false;
			if (rdr.ReadUInt32() != VERSION)
				return false;

			int headerSize = rdr.ReadUSize();
			if (headerSize < HEADER_FIXED || headerSize > data.Length)
				return false;

			int numTypes = rdr.ReadUSize();
			m_typeNames = new Dictionary<int, string>();
			for (int i = 0;i < numTypes;i++)
			{
				int id = rdr.ReadUSize();
				m_typeNames[id] = rdr.ReadString();
			}

			int numSlots = rdr.ReadUSize();
			m_slots = new Slot[numSlots];
			for (int i = 0;i < numSlots;i++)
			{
				Slot s = new Slot();
				s.flags = (int)rdr.ReadUInt32();
				if ((s.flags & ~(FLAG_PATH | FLAG_INTERNAL)) != 0)
					return false;
				if ((s.flags & FLAG_PATH) != 0)
					s.path = rdr.ReadString();
				s.type = rdr.ReadUSize();
				s.begin = rdr.ReadUSize();
				s.end = rdr.ReadUSize();
				if (s.end < s.begin || s.begin < headerSize || s.end > data.Length)
					return false;
				m_slots[i] = s;
			}

			// Payloads sit at absolute offsets, so each one is read from its own
			// position rather than assuming they follow the header in order.
			for (int i = 0;i < numSlots;i++)
			{
				if ((m_slots[i].flags & FLAG_INTERNAL) == 0)
					continue;
				PackageReader slotReader = new PackageReader(data, m_slots[i].begin);
				m_slots[i].inst = loader.LoadFromPackage(m_slots[i].type, slotReader);
			}

			for (int i = 0;i < numSlots;i++)
			{
				if ((m_slots[i].flags & FLAG_INTERNAL) == 0)
					continue;
				m_slots[i].inst = loader.ResolveFromPackage(m_slots[i].type, m_slots[i].inst, this);
			}

			m_paths = new Dictionary<object, string>();
			for (int i = 0;i < numSlots;i++)
			{
				if (m_slots[i].inst != null && m_slots[i].path != null)
				{
					m_paths[m_slots[i].inst] = m_slots[i].path;
				}
			}
			return !m_gotUnresolved;
		}

		public void Release()
		{
			m_slots = null;
		}
	}
}
