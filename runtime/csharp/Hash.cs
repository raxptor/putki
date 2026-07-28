using System.Collections.Generic;

namespace Putki
{
	public struct Hash
	{
		public ulong Value;

		public static Hash Construct(string source)
		{
			Store(source);
			return new Hash()
			{
				Value = Compute(source)
			};
		}

		public bool Empty()
		{
			return Value == 0;
		}

		public override bool Equals(object b)
		{
			return Value == ((Hash)b).Value;
		}

		public override int GetHashCode()
		{
			return (int)Value;
		}

		public override string ToString()
		{
			string s;
			if (s_source.TryGetValue(Value, out s))
				return "#" + s;
			return "???-hash-" + Value;
		}

		[System.Diagnostics.Conditional("DEBUG")]
		static void Store(string source)
		{
			ulong hash = Compute(source);
			string s;
			if (s_source.TryGetValue(hash, out s))
			{
				if (s_source.ContainsKey(hash) && s_source[hash] != source)
					throw new System.Exception("Hash collision on [" + source + "] with [" + s_source[hash] + "]!");
			}
			else
			{
				s_source.Add(hash, source);
			}
		}

		public static ulong Compute(string input)
		{
			ulong hashedValue = 3074457345618258791ul;
			for (int i = 0; i < input.Length; i++)
			{
				hashedValue += input[i];
				hashedValue *= 3074457345618258799ul;
			}
			return hashedValue;
		}

		static Dictionary<ulong, string> s_source = new Dictionary<ulong, string>();
		public static bool operator ==(Hash lhs, Hash rhs)
		{
			return lhs.Value == rhs.Value;
		}
		public static bool operator !=(Hash lhs, Hash rhs)
		{
			return lhs.Value != rhs.Value;
		}
	}
}