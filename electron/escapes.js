// String escaping for putki text data. See doc/text-format.md, which is the
// normative definition; this file and the Rust implementation in
// rust/putki-inki/src/inki/lexer.rs must agree.
//
// Deliberately free of dependencies so it can be unit tested with plain node,
// and shared by the reader and the writer so the two cannot drift apart.

function hexval(ch)
{
    if (ch === undefined)
        return -1;
    var x = ch.charCodeAt(0);
    if (x >= 0x30 && x <= 0x39)
        return x - 0x30;
    x |= 0x20;
    if (x >= 0x61 && x <= 0x66)
        return 10 + x - 0x61;
    return -1;
}

// Encode a string as a quoted literal.
//
// Emits only \\ \" \n and \t; everything else goes out as raw UTF-8. Putki
// draws no distinction between carriage return and newline, so CRLF and a lone
// CR both normalise to \n. \uXXXX is accepted on read but never produced, so it
// drains out of the data as files are re-saved.
function encode_string(str)
{
    // By code point, so astral characters are not split into surrogate halves.
    var chars = Array.from(str);
    var out = ['"'];
    for (var i = 0; i < chars.length; i++)
    {
        var c = chars[i];
        if (c === '\\')
            out.push('\\\\');
        else if (c === '"')
            out.push('\\"');
        else if (c === '\n')
            out.push('\\n');
        else if (c === '\t')
            out.push('\\t');
        else if (c === '\r')
        {
            if (chars[i + 1] === '\n')
                i++;
            out.push('\\n');
        }
        else
            out.push(c);
    }
    out.push('"');
    return out.join('');
}

// Decode the body of a quoted literal, i.e. what sits between the quotes.
// Returns null on an invalid escape rather than guessing.
function decode_string_body(body)
{
    var chars = Array.from(body);
    var encoder = new TextEncoder();
    // Accumulated as bytes because legacy \uXXXX encoded the individual bytes
    // of the original UTF-8, not a code point, so a run of them has to be
    // reassembled before it is valid text.
    var bytes = [];
    for (var i = 0; i < chars.length; i++)
    {
        var c = chars[i];
        if (c === '\r')
        {
            if (chars[i + 1] === '\n')
                i++;
            bytes.push(0x0a);
            continue;
        }
        if (c !== '\\')
        {
            var encoded = encoder.encode(c);
            for (var k = 0; k < encoded.length; k++)
                bytes.push(encoded[k]);
            continue;
        }
        var n = chars[++i];
        if (n === 'n')
            bytes.push(0x0a);
        else if (n === 't')
            bytes.push(0x09);
        else if (n === 'r')
            bytes.push(0x0a);
        else if (n === '\\')
            bytes.push(0x5c);
        else if (n === '"')
            bytes.push(0x22);
        else if (n === 'u')
        {
            var code = 0;
            for (var k = 0; k < 4; k++)
            {
                var h = hexval(chars[i + 1 + k]);
                if (h < 0)
                    return null;
                code = (code << 4) | h;
            }
            if (code > 0xff)
                return null;
            bytes.push(code);
            i += 4;
        }
        else
            return null;
    }
    try
    {
        return new TextDecoder('utf-8', { fatal: true }).decode(new Uint8Array(bytes));
    }
    catch (e)
    {
        return null;
    }
}

exports.encode_string = encode_string;
exports.decode_string_body = decode_string_body;
