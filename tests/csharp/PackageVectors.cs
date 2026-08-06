// Parses tests/fixtures/sample.pkg, which is emitted by the Rust writer and
// checked byte for byte by the Rust test. If this and that test both pass, the
// two implementations agree on the format.
//
//     mcs -out:packagetest.exe ../../runtime/csharp/Package.cs PackageVectors.cs
//     mono packagetest.exe ../fixtures/sample.pkg
//
// The fixture holds the types from the Rust pipeline test: Multi (id 2), which
// is one TestValues, and Pointer (id 3), which is a TestValues plus a pointer.

using System;
using System.Collections.Generic;
using System.IO;
using Putki;

namespace PutkiPackageVectors
{
	class TestValues
	{
		public int value1, value2;
	}

	class Multi
	{
		public TestValues contained;
	}

	class Pointer
	{
		public TestValues contained;
		public int __slot_next;
		public Pointer next;
	}

	class Loader : TypeLoader
	{
		public const int TYPE_MULTI = 2;
		public const int TYPE_POINTER = 3;

		static TestValues LoadTestValues(PackageReader r)
		{
			TestValues v = new TestValues();
			v.value1 = r.ReadInt32();
			v.value2 = r.ReadInt32();
			return v;
		}

		public object LoadFromPackage(int type, PackageReader reader)
		{
			switch (type)
			{
				case TYPE_MULTI:
				{
					Multi m = new Multi();
					m.contained = LoadTestValues(reader);
					return m;
				}
				case TYPE_POINTER:
				{
					Pointer p = new Pointer();
					p.contained = LoadTestValues(reader);
					p.__slot_next = reader.ReadSlotRef();
					return p;
				}
			}
			return null;
		}

		public object ResolveFromPackage(int type, object obj, Package pkg)
		{
			if (type == TYPE_POINTER)
			{
				Pointer p = (Pointer)obj;
				p.next = pkg.ResolveSlot<Pointer>(p.__slot_next);
			}
			return obj;
		}
	}

	class PackageVectors
	{
		static int failures = 0;

		static void Check(string what, object got, object want)
		{
			if (!object.Equals(got, want))
			{
				failures++;
				Console.WriteLine("FAIL " + what + "\n  got  " + got + "\n  want " + want);
			}
		}

		public static int Main(string[] args)
		{
			string path = args.Length > 0 ? args[0] : "../fixtures/sample.pkg";
			byte[] bytes = File.ReadAllBytes(path);

			Package pkg = new Package();
			Loader loader = new Loader();
			if (!pkg.LoadFromBytes(bytes, loader))
			{
				Console.WriteLine("FAIL package did not load");
				return 1;
			}

			Check("type name of 2", pkg.TypeName(2), "Multi");
			Check("type name of 3", pkg.TypeName(3), "Pointer");

			// The builders in the Rust test add 1000 and 2000 to the two fields.
			Multi multi = (Multi)pkg.Resolve("multi");
			if (multi == null)
			{
				Console.WriteLine("FAIL no object at path 'multi'");
				return 1;
			}
			Check("multi.value1", multi.contained.value1, 321 + 1000);
			Check("multi.value2", multi.contained.value2, 654 + 2000);

			Pointer ptr = (Pointer)pkg.Resolve("ptr");
			if (ptr == null)
			{
				Console.WriteLine("FAIL no object at path 'ptr'");
				return 1;
			}
			Check("ptr.value1", ptr.contained.value1, 1 + 1000);
			Check("ptr.value2", ptr.contained.value2, 2 + 2000);

			// And the pointer must have been followed to a real object.
			if (ptr.next == null)
			{
				failures++;
				Console.WriteLine("FAIL ptr.next did not resolve");
			}
			else
			{
				Check("ptr.next.value1", ptr.next.contained.value1, 222 + 1000);
				Check("ptr.next.value2", ptr.next.contained.value2, 333 + 2000);
			}

			// A truncated package must fail cleanly, not throw or read past the end.
			byte[] truncated = new byte[bytes.Length / 2];
			Array.Copy(bytes, truncated, truncated.Length);
			if (new Package().LoadFromBytes(truncated, loader))
			{
				failures++;
				Console.WriteLine("FAIL truncated package was accepted");
			}

			// So must one that is not a package at all.
			byte[] garbage = new byte[64];
			if (new Package().LoadFromBytes(garbage, loader))
			{
				failures++;
				Console.WriteLine("FAIL garbage was accepted as a package");
			}

			if (failures > 0)
			{
				Console.WriteLine("\n" + failures + " failure(s)");
				return 1;
			}
			Console.WriteLine("package vectors pass");
			return 0;
		}
	}
}
