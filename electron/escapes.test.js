// Run with: node escapes.test.js
//
// Normative escaping vectors from doc/text-format.md. The Rust suite checks the
// same table in rust/putki-inki/tests/pipeline.rs (ESCAPE_VECTORS); keep the
// three in sync.

var escapes = require('./escapes.js');

// [raw value, encoded form]
var VECTORS = [
    ["plain", '"plain"'],
    ['with "quote"', '"with \\"quote\\""'],
    ["back\\slash", '"back\\\\slash"'],
    ["trailing\\", '"trailing\\\\"'],
    ["two\\\\slashes", '"two\\\\\\\\slashes"'],
    ["line\nbreak", '"line\\nbreak"'],
    ["tab\there", '"tab\\there"'],
    ['quote"then\\', '"quote\\"then\\\\"'],
    ['\\"', '"\\\\\\""'],
    ["unicode: åäö", '"unicode: åäö"'],
    ["", '""'],
];

var failures = 0;

function check(what, got, want)
{
    var ok = got === want;
    if (!ok)
    {
        failures++;
        console.log("FAIL " + what + "\n  got  " + JSON.stringify(got) + "\n  want " + JSON.stringify(want));
    }
    return ok;
}

// Strip the surrounding quotes the encoder adds, to feed the decoder.
function body_of(encoded)
{
    return encoded.substring(1, encoded.length - 1);
}

VECTORS.forEach(function (v) {
    check("encode " + JSON.stringify(v[0]), escapes.encode_string(v[0]), v[1]);
    check("decode " + JSON.stringify(v[1]), escapes.decode_string_body(body_of(v[1])), v[0]);
    check("roundtrip " + JSON.stringify(v[0]),
        escapes.decode_string_body(body_of(escapes.encode_string(v[0]))), v[0]);
});

// Legacy \uXXXX carries bytes of the original UTF-8, so a run reassembles into
// one char: U+00E5 is C3 A5 in UTF-8.
check("legacy \\u utf8 pair", escapes.decode_string_body("\\u00c3\\u00a5"), "å");
check("legacy \\u in context", escapes.decode_string_body("pre \\u00c3\\u00a5 post"), "pre å post");
// A lone å is the Latin-1 byte, not UTF-8, so it is malformed legacy data.
check("lone latin-1 byte rejected", escapes.decode_string_body("\\u00e5"), null);
// Newline is newline.
check("legacy \\r folds to newline", escapes.decode_string_body("a\\rb"), "a\nb");
check("raw CRLF normalises", escapes.encode_string("a\r\nb\rc"), '"a\\nb\\nc"');
// ...and neither legacy form is ever produced again.
check("never emits \\u", escapes.encode_string("å\r\n\r").indexOf("\\u"), -1);
check("never emits \\r", escapes.encode_string("å\r\n\r").indexOf("\\r"), -1);

// Invalid escapes are rejected, not guessed at.
check("bad escape rejected", escapes.decode_string_body("bad \\q escape"), null);
check("short \\u rejected", escapes.decode_string_body("short \\u00"), null);
check("non-hex \\u rejected", escapes.decode_string_body("\\uzzzz"), null);

if (failures > 0)
{
    console.log("\n" + failures + " failure(s)");
    process.exit(1);
}
console.log("all escape vectors pass");
