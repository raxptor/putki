var fs = require('fs');
var path = require('path');

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

function unhex(ch)
{
    var x = ch.charCodeAt();
    var sym = '09af';
    if (x >= sym.charCodeAt(0) && x <= sym.charCodeAt(1))
        return x - sym.charCodeAt(0);
    if (x >= sym.charCodeAt(2) && x <= sym.charCodeAt(3))
        return 10 + x - sym.charCodeAt(2);
    return 0;
}

function decode_string(buf, begin, end)
{
    var tmp = [];
    var len = 0;
    for (var i=begin;i<end;i++)
    {
        if (buf[i] != '\\')
        {
            tmp.push(buf[i]);
        }
        else if ((i+1) < end)
        {
            if (buf[i+1] == 'u')
            {
                if ((i+5) < end)
                {
                    var code = 
                        16*16*16*unhex(buf[i+2]) + 
                        16*16*unhex(buf[i+3]) +
                        16*unhex(buf[i+4]) +
                        unhex(buf[i+5]);
                    tmp.push(String.fromCharCode(code));
                    i += 5;
                }
            }
            else if (buf[i+1] =='n')
            {
                tmp.push('\n');
                ++i;
            }
            else if (buf[i + 1] == '\\')
            {
                tmp.push('\\');
                ++i;
            }
        }
    }
    return tmp.join("");
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
                    status.pos = i + 1;
                    if (v.startsWith("$FIX-WS:"))
                    {
                        var sb = [];
                        var ws = true;
                        for (var j=8;j<v.Length;j++)
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
                        v = sb.join("");
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

function tree_sync(root, cb)
{
    var items = fs.readdirSync(root);
    for (var i=0;i<items.length;i++)
    {
        var p = path.join(root, items[i]);
        var st = fs.statSync(p);
        if (st.isDirectory())
            tree_sync(p, cb);
        if (!p.endsWith(".txt"))
            continue;
        cb(p);
    }
}

exports.load_tree = function(_path, result)
{
    tree_sync(_path, function(file) {
        var data = strip_comments(fs.readFileSync(file, "utf8"));
        var pd = {
            data: data,
            pos: 0,
            error: false,
            result: result,
            file: path.relative(_path, file)
        };
        parse(pd, true);
        if (pd.error)
        {
            console.error("Failed to parse [" + pd + "]");
            return false;
        }
    });
    return true;
}

exports.load_file = function(fn)
{
   var data = fs.readFileSync(fn, "utf8");
   data = strip_comments(data);
   console.log(strip_comments(data));

   var pd = {
       data: data,
       pos: 0,
       error: false,
       file: fn,
       result: {}
   };

   console.log("PARSING");
   console.log("retval=", parse(pd, true));
   console.log(pd);
   console.log(JSON.stringify(pd.result["items/grenade"]));
};
