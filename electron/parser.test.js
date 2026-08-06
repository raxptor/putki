// Run with: node parser.test.js
//
// End-to-end check that the parser decodes escapes via escapes.js, complementing
// escapes.test.js which tests the module in isolation.

var parser = require('./dataparser.js');
var escapes = require('./escapes.js');

function parse_field(encoded)
{
    var src = "@T o { f: " + encoded + " }";
    var st = { data: parser.strip_comments(src), pos: 0, error: false, result: {}, file: "t" };
    parser.parse(st, true);
    if (st.error)
        return { error: true };
    return { value: st.result["o"] ? st.result["o"]["f"] : undefined, error: false };
}

var failures = 0;
function check(what, got, want)
{
    if (got !== want)
    {
        failures++;
        console.log("FAIL " + what + "\n  got  " + JSON.stringify(got) + "\n  want " + JSON.stringify(want));
    }
}

// [encoded literal as it appears in a data file, expected decoded value]
var CASES = [
    ['"plain"', "plain"],
    ['"with \\"quote\\""', 'with "quote"'],
    ['"back\\\\slash"', "back\\slash"],
    ['"line\\nbreak"', "line\nbreak"],
    ['"tab\\there"', "tab\there"],
    ['"unicode: åäö"', "unicode: åäö"],
    ['"\\u00c3\\u00a5"', "å"],
    ['"a\\rb"', "a\nb"],
];

CASES.forEach(function (c) {
    var got = parse_field(c[0]);
    check("parse " + c[0], got.error ? "<error>" : got.value, c[1]);
});

// Writer output must survive a trip through the reader.
["plain", 'with "quote"', "back\\slash", "line\nbreak", "tab\there", "åäö", ""].forEach(function (raw) {
    var got = parse_field(escapes.encode_string(raw));
    check("roundtrip " + JSON.stringify(raw), got.error ? "<error>" : got.value, raw);
});

// Invalid escapes must set status.error rather than yield a mangled value.
[['"bad \\q escape"', "unknown escape"], ['"short \\u00"', "truncated \\u"]].forEach(function (c) {
    var got = parse_field(c[0]);
    if (!got.error)
    {
        failures++;
        console.log("FAIL " + c[1] + " was accepted, value " + JSON.stringify(got.value));
    }
});

if (failures > 0)
{
    console.log("\n" + failures + " failure(s)");
    process.exit(1);
}
console.log("all parser escape cases pass");
