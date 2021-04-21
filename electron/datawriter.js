var fs = require('fs');
var path = require('path');
const fsextra = require('fs-extra');
var md5 = require('js-md5');

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
        else if (c == '\\')
            chars.push("\\\\");
        else if (c == '\t')
            chars.push("\\t");
        else if (c == '\"')
            chars.push("\\\"");
        else if (cc <= 127 || c == ' ')            
            chars.push(c);
        else         
            chars.push(c);
    }
    return "\"" + chars.join("") + "\"";
}

var unfiltered = ["I32", "U32", "U8", "Float", "Bool"];

function generateUUID() { // Public Domain/MIT
    var d = new Date().getTime();//Timestamp
    var d2 = (performance && performance.now && (performance.now()*1000)) || 0;//Time in microseconds since page-load or 0 if unsupported
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
        var r = Math.random() * 16;//random number between 0 and 16
        if(d > 0){//Use timestamp until depleted
            r = (d + r)%16 | 0;
            d = Math.floor(d/16);
        } else {//Use microseconds since page-load if supported
            r = (d2 + r)%16 | 0;
            d2 = Math.floor(d2/16);
        }
        return (c === 'x' ? r : (r & 0x3 | 0x8)).toString(16);
    });
}

var new_line = "\n";

exports.set_new_line = function(nl) {
    new_line = nl;
}

function format(types, data, indent, typename, paths, build_fns)
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

    // Quick; no need for comma delimiters when having newlines.    
    delim = "";
    nlsep = new_line + "\t".repeat(indent+1);
    finsep = new_line + "\t".repeat(indent);

    if (data === null)
        return "null";

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
            var pc = format(types, data[k], indent+1, typename, paths, build_fns);
            if (pc != null)
                pcs.push(pc);
            else
                pcs.push("{}");
        }
        return "[" + nlsep + pcs.join(delim + nlsep) + finsep + "]";
    }
    if (data instanceof Object)
    {
        var pcs = [];
        var tn = data._type || typename;
        var type = types[tn];

        // custom mangling step.
        if (build_fns !== undefined && build_fns[tn] !== undefined)
            data = build_fns[tn](data);

        var flds = type.ExpandedFields.slice(0).sort( (a, b) => {
            var x = a["Name"];
            var y = b["Name"];
            return x < y ? -1 : (x > y ? 1 : 0);
        });


        for (var i=0;i<flds.length;i++) {
            var f = flds[i].Name;
            if (data[f] === undefined || data[f] == null)
                continue;
            if (flds[i].Array && data[f].length == 0)
                continue;
            var frmted = format(types, data[f], indent+1, flds[i].Type, paths, build_fns);
            if (frmted !== null)
                pcs.push(flds[i].PrettyName + ": " + frmted);
        }

        if (pcs.length == 0 && data._type === undefined && data._path === undefined)
            return null;

        if (type.RequirePath) {
            // need to assign path also to get the header written
            data._type = data._type || typename;
            if (data._path == undefined) {
                var check = "guid/" + generateUUID();
                console.log("Assigned path ", check, " to type ", data._type);
                data._path = check;
                paths[data._path] = true;
            } else {
                paths[data._path] = true;
            }
        }

        if (data._path !== undefined) {
            // enforce output of type if path is present.                        
            data._type = data._type || typename;
        }

        var hdr = "";
        if (data._type !== undefined) {
            hdr = hdr + "@" + types[data._type].PrettyName + " ";
         }
        if (data._path !== undefined) {
             hdr = hdr + data._path + " ";
        }        
        if (indent == 0)
            return hdr.trim() + new_line + "{" + nlsep + pcs.join(delim + nlsep) + finsep + "}";    
        else {
            if (pcs.length == 0)
                return hdr + "{ }";
            else
                return hdr + "{" + nlsep + pcs.join(delim + nlsep) + finsep + "}";
        }
    }
    else
    {
        return data;
    }
}

exports.write = function(root, types, data, single_file, build_fns)
{
    var files = {};
    var paths = {};
    for (var x in data)
    {
        if (data[x]._path !== undefined)
            paths[data[x]._path] = true;    
    }
    for (var x in data)
    {
        var d = data[x]; 
        var actual = single_file || d._file;
        var file = files[actual];
        if (file === undefined) {
            file = [];
            files[actual] = file;
        }
        file.push(format(types, d, 0, undefined, paths, build_fns));
    }
    for (var x in files)
    {
        var pth = path.join(root, x);
        console.log("writing file ", pth);
        fsextra.ensureDirSync(path.dirname(pth));
        fs.writeFileSync(pth, files[x].join(new_line + new_line) + new_line);
    }
};
