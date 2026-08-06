// parese functions

var escapes = require('./escapes.js');

function strip_comments(input)
{
    stripped = [];
    var comment = false;
    var quote = false;
    var escape = false;
    for (var i=0;i<input.length;i++)
    {
        var c = input[i];
        var n = '\0';
        if (i < (input.length-1))
            n = input[i+1];

        if (!quote && c == '\"')
        {
            quote = true;
        }
        else if (quote && !escape && c == '\"')
        {
            quote = false;
        }
        if (quote && !escape && c == '\\')
        {
            escape = true;
        }
        if (c != '\\')
        {
            escape = false;
        }
        if (!quote && !comment && (c == '#' || (c == '/' && n == '/')))
        {
            comment = true;
        }
        if (comment && (c.charCodeAt() == 0xd || c.charCodeAt() == 0xa))
        {
            comment = false;
        }	
        if (!comment)
        {
            stripped.push(c);
        }
    }
    return stripped.join('');
}

function is_whitespace(c)
{
    return (c == ' ') || (c == '\t') || (c == '\n') || c.charCodeAt() == 0xd || c.charCodeAt() == 0xa;
}

// Escape handling lives in escapes.js so the reader and the writer cannot
// drift apart. Returns null on an invalid escape; callers must flag the error
// rather than carry on with a half-decoded value.
function decode_string(buf, begin, end)
{
    return escapes.decode_string_body(buf.substring(begin, end));
}

function parse(status, rootlevel)
{
    const NOTHING = 0;
    const QUOTED_VALUE = 1;
    const ARRAY = 2;
    const HEADER = 3;
    const VALUE = 4;
    const OBJECT = 5;

    var state = NOTHING;
    var o = {};
    var a = [];
    var name;

    for (var i=status.pos;i<status.data.length;i++)
    {
        var c = status.data[i];
        switch (state)
        {
            case NOTHING:
                {
                    if (!is_whitespace(c))
                    {
                        switch (c)
                        {
                            case '@': state = HEADER; break;
                            case '{': state = OBJECT; o = {}; break;
                            case '[': state = ARRAY; a = []; break;
                            case ' ': case '\n': case '\t': break;
                            case '"': state = QUOTED_VALUE; status.pos = i + 1; break;
                            default: state = VALUE; status.pos = i; break;
                        }
                    }
                    break;
                }
            case QUOTED_VALUE:
            {
                if (c == '\\')
                {
                    i++;
                    break;
                }
                if (c == '"')
                {
                    var v = decode_string(status.data, status.pos, i);
                    if (v === null)
                    {
                        status.error = true;
                        return null;
                    }
                    status.pos = i + 1;
                    if (v.startsWith("$FIX-WS:"))
                    {
                        var sb = [];
                        var ws = true;
                        for (var j=8;j<v.length;j++)
                        {
                            var vc = v[j];
                            if (vc == ' ' || vc == '\t' || vc == '\r' || vc == '\n')
                            {
                                if (!ws)
                                {
                                    ws = true;
                                    sb.push(' ');
                                }
                            }
                            else
                            {
                                ws = false;
                                sb.push(vc);
                            }
                        }
                        v = sb.join("").replace( /\\n/g, "\n");
                    }
                    return v;
                }
                break;
            }
            case HEADER:
                {
                    if (c == '{' || c== '[')
                    {
                        var header = decode_string(status.data, status.pos, i);
                        if (header === null)
                        {
                            status.error = true;
                            return null;
                        }
                        var pcs = header.trim().split(' ');                                
                        if (pcs.length < 1)
                        {
                            status.error = true;
                            return null;
                        }

                        status.pos = i;
                        var data = parse(status);
                        if (status.error || data == null)
                            return null;
                        i = status.pos - 1;

                        data["_type"] = pcs[0].replace("@", "").toLowerCase();
                        if (pcs.length > 1)
                        {
                            // it has path
                            var path = pcs[1].trim();
                            if (path.length > 0 && path[0] != "&")
                                data["_path"] = path;
                            data["_file"] = status.file;
                            if (rootlevel)
                                status.result[path] = data;
                            else
                                return data;
                        }
                        else
                        {
                            if (!rootlevel)
                                return data;
                        }
                        state = NOTHING;
                    }
                    break;
                }
            case VALUE:
                {
                    if (is_whitespace(c) || c == ',' || c == ']' || c == '}' || c == ':' || c == '=')
                    {
                        var v = decode_string(status.data, status.pos, i);
                        if (v === null)
                        {
                            status.error = true;
                            return null;
                        }
                        status.pos = i;
                        return v;
                    }
                    break;
                }
            case OBJECT:
                {
                    if (c == '}')
                    {
                        status.pos = i + 1;
                        return o;
                    }
                    if (is_whitespace(c) || c == ',')
                    {
                        continue;
                    }
                    if (name == null)
                    {
                        status.pos = i;
                        name = parse(status);
                        if (name == null)
                        {
                            status.error = true;
                            return null;
                        }
                        i = status.pos - 1;
                    }
                    else 
                    {
                        if (c == ':' || c == '=')
                        {
                            continue;
                        }
                        status.pos = i;
                        var val = parse(status);
                        if (val == null)
                        {
                            status.error = true;
                            return null;
                        }
                        o[name.toLowerCase()] = val;
                        i = status.pos - 1;
                        name = null;
                    }
                    break;
                }
            case ARRAY:
                {
                    if (c == ']')
                    {
                        status.pos = i + 1;
                        return a;
                    }
                    if (is_whitespace(c) || c == ',')
                    {
                        continue;
                    }
                    status.pos = i;
                    var val = parse(status);
                    if (val == null)
                    {
                        status.error = true;
                        return null;
                    }
                    a.push(val);
                    i = status.pos - 1;
                    break;
                }
            default:
                break;
        }
    }
    return null;
}

exports.parse = parse;
exports.strip_comments = strip_comments;
