var fs = require('fs');
var path = require('path');


/*
function make_array(types, data)
{
    var c = [];
    for (var i=0;i<data.length;i++)
        c.push(make(types, data));
    return {
        pre: "[",
        contents: c,
        post: "]"
    }
}

function make_object(types, data)
{
    var pcs = [];
    for (var d in data) {
        if (d.charAt(0) != '_') {
            var val = data[d];
            pcs.push(d + ": " + make(data[d]) )
            if (val instanceof Array)
                pcs.push(make_array());
            else
                pcs.push(d + ": " + data[d]);
        }
    }
    return {
        pre: "@" + types[data._type].PrettyName + " " + data._path + " {",
        contents: pcs,
        post: "}"
    };
}*/

function format(types, data, indent, typename)
{
    var delim = ",";
    var nlsep = "";
    var finsep = "";
    var m = 0;

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
        var flds = type.Fields;
        for (var i=0;i<flds.length;i++) {
            var f = flds[i].Name;
            if (data[f] === undefined)
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

exports.write = function(root, types, data)
{
    var files = {};
    for (var x in data)
    {
        var d = data[x]; 
        var file = files[d._file];
        if (file === undefined) {
            file = [];
            files[d._file] = file;
        }
        file.push(format(types, d, 0));
    }
    for (var x in files)
    {
        var pth = path.join(root, x);
        console.log("writing file ", pth);
        fs.writeFileSync(pth, files[x].join("\n\n"));
    }
};
