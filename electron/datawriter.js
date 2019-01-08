var fs = require('fs');
var path = require('path');
const fsextra = require('fs-extra');

function format_string(str, indent)
{
    // indent = -1 means trial.
    if (indent == -1)
        return str;

    var chars = [];
    var hex = "0123456789ABCDEF";
    for (var i=0;i<str.length;i++)
    {
        var c = str[i];
        var cc = str.charCodeAt(i);
        if (c == '\r')
            continue;
        if (c == '\n')
            chars.push("\\n");
        else if (c == '\t')
            chars.push("\\t");
        else if (c == '\"')
            chars.push("\\\"");
        else if (cc <= 127 || c == ' ')            
            chars.push(c);
        else {            
            chars.push("\\u");
            chars.push(hex[(cc >> 12) & 0xf]);
            chars.push(hex[(cc >> 8) & 0xf]);
            chars.push(hex[(cc >> 4) & 0xf]);
            chars.push(hex[(cc >> 0) & 0xf]);
        }
    }
    return "\"" + chars.join("") + "\"";
}

var unfiltered = ["I32", "U32", "U8", "Float", "Bool"];

function format(types, data, indent, typename)
{
    var delim = ",";
    var nlsep = "";
    var finsep = "";
    var m = 0;

    /*
    if (indent != -1)
        m = (format(types, data, -1, typename) || "").length;
    if (indent != -1 && indent * 6 + m > 120) {
        nlsep = "\n" + "\t".repeat(indent+1);
        finsep = "\n" + "\t".repeat(indent);
    } else {
        finsep = " ";
        nlsep = " ";
        delim = ",";
    }
    */

    // quick
    nlsep = "\n" + "\t".repeat(indent+1);
    finsep = "\n" + "\t".repeat(indent);

    if (data.constructor == String && unfiltered.indexOf(typename) != -1)
    {
        return data;
    }

    if (data.constructor == String) 
    {
        return format_string(data, indent);
    }

    if (data instanceof Array)
    {
        var pcs = [];
        for (var k=0;k<data.length;k++) {
            pcs.push(format(types, data[k], indent+1, typename));
        }
        return "[" + nlsep + pcs.join(delim + nlsep) + finsep + "]";
    }
    if (data instanceof Object)
    {
        var pcs = [];
        var type = types[data._type || typename];
        var flds = type.ExpandedFields;
        for (var i=0;i<flds.length;i++) {
            var f = flds[i].Name;
            if (data[f] === undefined)
                continue;
            if (flds[i].Array && data[f].length == 0)
                continue;
            var frmted = format(types, data[f], indent+1, flds[i].Type);
            if (frmted !== null)
                pcs.push(flds[i].PrettyName + ": " + frmted);
        }

        if (pcs.length == 0 && data._type === undefined && data._path === undefined)
            return null;

        var hdr = "";
        if (data._type !== undefined) {
            hdr = hdr + "@" + types[data._type].PrettyName + " ";
         }
        if (data._path !== undefined) {
             hdr = hdr + data._path + " ";
        }
        return hdr + "{" + nlsep + pcs.join(delim + nlsep) + finsep + "}";
    }
    else
    {
        return data;
    }
}

exports.write = function(root, types, data, single_file)
{
    var files = {};
    for (var x in data)
    {
        var d = data[x]; 
        var actual = single_file || d._file;
        var file = files[actual];
        if (file === undefined) {
            file = [];
            files[actual] = file;
        }
        file.push(format(types, d, 0));
    }
    for (var x in files)
    {
        var pth = path.join(root, x);
        console.log("writing file ", pth);
        fsextra.ensureDirSync(path.dirname(pth));
        fs.writeFileSync(pth, files[x].join("\n\n"));
    }
};
