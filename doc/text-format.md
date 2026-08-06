Putki text format: string escaping
==================================

This is the normative definition of how string literals are escaped in putki's
text data files. It exists because there are several independent implementations
of the format (Rust `inki`, C# `mixki`, the Electron editor's JS, the C++
builder), and they historically disagreed in ways that silently corrupted data
rather than failing.

Escape set
----------

| Escape | Means | Emitted by writers |
|---|---|---|
| `\\` | backslash | yes |
| `\"` | double quote | yes |
| `\n` | newline | yes |
| `\t` | tab | yes |
| `\r` | newline (see below) | **no**, accepted on read only |
| `\uXXXX` | one byte, `XX <= FF` | **no**, accepted on read only |
| anything else | — | invalid, reject the file |

Everything outside this table is written literally as UTF-8. Non-ASCII text
needs no escaping.

Newlines
--------

Putki draws no distinction between carriage return and newline. A CRLF pair, a
lone CR, and a legacy `\r` escape all mean one newline, and readers normalise
them to `\n`. Writers only ever emit `\n`. There is no way to represent a
carriage return, by design.

Legacy `\uXXXX`
---------------

Older writers encoded non-ASCII by emitting one `\uXXXX` per **byte of the
UTF-8 encoding**, not per code point, and only values up to `00FF` were ever
produced. So `å` (U+00E5) appears as `Ã¥`, its two UTF-8 bytes — not
as `å`.

Readers must therefore decode `\uXXXX` into a single byte and reassemble the
byte sequence into text at the end. A run that does not reassemble into valid
UTF-8 is malformed and must be rejected, not silently replaced. In particular a
lone `å` is Latin-1, not UTF-8, and is invalid.

No writer emits this form any more, so it drains out of the data naturally as
files are re-saved. Readers should keep accepting it until old data is gone.

Invalid escapes are errors
--------------------------

An unrecognised escape must fail the parse. Guessing is what the older readers
did, and it is why the divergences below went unnoticed for years: the data
looked like it loaded.

Test vectors
------------

Every implementation should round-trip these. The Rust suite checks them in
`rust/putki-inki/tests/pipeline.rs` (`ESCAPE_VECTORS`); keep the two in sync.

| Value | Encoded |
|---|---|
| `plain` | `"plain"` |
| `with "quote"` | `"with \"quote\""` |
| `back\slash` | `"back\\slash"` |
| `trailing\` | `"trailing\\"` |
| `two\\slashes` | `"two\\\\slashes"` |
| `line`⏎`break` | `"line\nbreak"` |
| `tab`⇥`here` | `"tab\there"` |
| `quote"then\` | `"quote\"then\\"` |
| `\"` | `"\\\""` |
| `unicode: åäö` | `"unicode: åäö"` |
| *(empty)* | `""` |

Plus, on read only: `"Ã¥"` decodes to `å`; `"a\rb"` decodes to `a`⏎`b`;
and `"å"`, `"bad \q escape"`, `"short \u00"` and an unterminated literal
are all rejected.

Known divergences
-----------------

As of this writing only the Rust reader/writer implements the above. The others
need bringing in line against the vectors:

* **C# (`runtime/csharp/MicroJson.cs`)** — `DecodeString` handles `\uXXXX`, but
  for any other escape it appends nothing and advances only one position, so it
  never skips the escaped character. `\"` happens to come out right. `\\`
  decodes to the *empty string*, meaning a literal backslash cannot survive a
  round-trip. `\n` and `\t` decode to the letters `n` and `t`.
* **Electron editor (`electron/datawriter.js`, `dataparser.js`)** — the writer
  emits the right four escapes, but drops `\r` instead of normalising it, and
  carries a dead `hex` table from an abandoned `\uXXXX` encoder. The reader
  decodes only `\n`, via a regex, so it does not round-trip its own output for
  backslash or quote.
* **C++ (`builder/putki/builder/parse.cpp`)** — `get_value_string` decodes
  `\uXXXX` only and passes other escapes through with the backslash intact.
