Putki binary package format
===========================

Normative definition of the binary package format, version 1. Written by the
Rust pipeline (`rust/putki-inki/src/pipeline/writer.rs`) and read by the Rust
runtime (`rust/putki-outki/src/outki/`) and the C# runtime
(`runtime/csharp/Package.cs`).

This replaces the older C++ builder format, which had a `PKTP` magic, 32-bit
sizes and `int16` type ids. That format is gone along with the C++ tree; it
survives only in git history.

Assumed invariant
-----------------

**Packages are written by the same build of the binaries that load them.**

Type ids come from the compiler and are assigned by declaration order, so
adding, removing or reordering a typedef renumbers them. There is deliberately
no support for remapping old ids to new ones. Ship packages with the executable
that built them; if that ever stops being true, this format needs a type
remapping table and this document needs revisiting.

The `version` field exists so that a mismatch fails loudly instead of being
misparsed.

Primitive encoding
------------------

All little-endian. There is no alignment or padding anywhere.

| Type | Encoding |
|---|---|
| `u32` | 4 bytes |
| `usize` | 8 bytes; low 4 bytes carry the value, high 4 bytes are written as zero |
| `string` | `usize` byte length, then that many raw UTF-8 bytes, **not** NUL terminated |

`usize` is a 64-bit field with a 32-bit range. Readers should reject a non-zero
high word rather than silently truncate.

Layout
------

```
header:
    magic          u32      0x494B5450  ('PTKI' little-endian)
    version        u32      1
    header_size    usize    total bytes of the header, i.e. offset of the data section
    type_count     usize
    types[type_count]:
        type_id    usize    compiler-assigned id, matches the generated TYPE_ID
        name       string   type name, for diagnostics only
    slot_count     usize
    slots[slot_count]:
        flags      u32      see below
        path       string   present only if flags & HAS_PATH
        type_id    usize    compiler-assigned id of the object in this slot
        begin      usize    absolute file offset of the slot payload
        end        usize    absolute file offset one past the payload
data:
    slot payloads, concatenated, in slot order
```

`begin` and `end` are offsets from the start of the file, not from the start of
the data section.

Flags
-----

| Flag | Value | Meaning |
|---|---|---|
| `HAS_PATH` | 1 | the slot carries a path string |
| `INTERNAL` | 2 | the payload is in this file's data section |

Other values are reserved and a reader should reject them. The old format's
`EXTERNAL`, `UNRESOLVED` and `RESOURCE` flags have no equivalent yet.

Payload encoding
----------------

Slot payloads are written by the generated `BinSaver` implementations. Every
field is stored **inline, in declaration order**, with no padding, no alignment
and no out-of-line area. (The old C++ format kept strings and arrays in a
separate "aux" region and pointed at them; this format does not.)

| Field type | Encoding |
|---|---|
| `s32` / `int` | `i32`, 4 bytes |
| `u32` / `uint` | 4 bytes |
| `byte` | 1 byte |
| `bool` | 1 byte, 0 or 1 |
| `float` | `f32` bits, 4 bytes |
| `enum` | `i32` of the enum's value, 4 bytes |
| `string`, `text`, `file`, `path`, `hash` | `string` (usize length + inline bytes) |
| `ptr` | `i32` slot reference |
| struct instance | that struct's fields, inline, recursively |
| any array | `usize` element count, then that many elements inline |

Note `hash` is stored as a string, not a hash value.

Polymorphic types
-----------------

A type that is an rtti root, or has children, is written as a `u16` variant
index followed by that variant's fields. The index is the position of the
concrete type in the parent's child list as the compiler emits it, **not** the
type id.

This is the one place where the format leans on the generated code of a
specific language matching the generator's ordering, and it is the part most
likely to need attention when porting a reader.

Slot references
---------------

Pointer fields are stored as a signed 32-bit slot reference: `-1` is null, and
`0..slot_count-1` indexes `slots`. Readers add one internally so that zero can
mean null, but that is an implementation detail and does not appear in the file.

Type checking
-------------

A reader that is asked for type `T` at a given slot must compare the slot's
`type_id` against `T`'s generated id and fail on a mismatch, rather than
reinterpreting the payload. The type table is there so the error can name both
types.

An id of `0` means "unknown" and is what a hand-written or not-yet-regenerated
type descriptor reports; readers skip the check in that case rather than
rejecting it.

Validation a reader must do
---------------------------

Package data is not trusted input; a truncated or corrupt file must produce an
error, never a panic, an out-of-bounds read, or a huge allocation.

* `magic` matches and `version` is understood.
* `header_size` is at least the size of the fixed header and no larger than the
  file.
* every `usize` has a zero high word.
* every string length fits within the remaining header bytes.
* `begin <= end`, and both lie within the file.
* `type_id` on a slot appears in the type table.
