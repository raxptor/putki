Type definition files
=====================

```
TestStruct
{
	string CamelCase
}
```

Use CamelCase for your struct names and names. In C# generated code they
will appear as-is. In C/C++ they will go lower_case instead.


Struct in a struct
==================

This is fine, and there are three possibilities. Pointers, aux-pointers and
just putting the object inside. The big difference is how the data is laid
out in your data files, rather than how you access the data from code.

```
OtherStruct
{
	int SuperValue = 3
}

TestStruct
{
	ptr OtherStruct Data1
	auxptr OtherStruct Data2
	OtherStruct Data3
}
```

The generated C# code for dealing with these structs will look like this.

```
public class TestStruct
{
	OtherStruct Data1;
	OtherStruct Data2;
	OtherSTruct Data3;
}
```

In C++, the first two will be actual pointers, and the last one won't be.
To see the useful difference, observe the example data file below. It
contains a TestStruct like above.

```
{
	"type": "TestStruct",
	"data": {
		"Data1": "other-1",
		"Data2": "#693hq",
		"Data3": {
			"SuperValue": 3
		}
	},
	"aux": [
		{
			"ref": "#693hq",
			"type": "OtherStruct",
			"data": {
				"SuperValue": 2
			}
		}
	]
}
```

The Data1 field refers to a different file altogether (/other-1.json) which
would then contain the OtherStruct. Data2 is an auxptr so the actual data is
contained within the same file in the 'aux' category. 

This helps reducing file spam on the disk, but functionally works just the
same as a regular reference. You can tell it is an aux-reference by the hash
mark in front of the path.

The last field is simply nested in.

All these pointer variants can also be used in array format.

```
TestStruct
{
	ptr[] OtherStruct Data1
	auxptr[] OtherStruct Data2
	OtherStruct[] Data3
}
```

And then you get arrays.

