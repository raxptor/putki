var parser = require('./dataparser');
var fs = require('fs');
var path = require('path');

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

function tree_sync_legacy(root, cb)
{
    var items = fs.readdirSync(root);
    for (var i=0;i<items.length;i++)
    {
        var p = path.join(root, items[i]);
        var st = fs.statSync(p);
        if (st.isDirectory())
            tree_sync_legacy(p, cb);
        if (!p.endsWith(".json"))
            continue;
        cb(p);
    }
}

function fixup_legacy(obj)
{
    if (Array.isArray(obj)) 
    {
        var out = [];
        for (var x in obj) {
            out[x] = fixup_legacy(obj[x]);
        }
        return out;
    }
    if (obj instanceof Object)
    {        
        var out = {};
        Object.keys(obj).forEach(function(key,index) {
            var value = obj[key];
            out[key.toLowerCase()] = fixup_legacy(value);
        });  
        if (out["parent"] !== undefined) 
        {
            // transfer parents down
            Object.keys(out["parent"]).forEach(function(key,index) {
                out[key.toLowerCase()] = out["parent"][key];
            });  
            delete out["parent"];
        }        
        return out;
    }
    return obj;
}


function inline_legacy_aux(obj, resolve, parent)
{
    if (obj === null || obj === undefined)
        return obj;
    if (obj._path !== undefined && obj._path.indexOf('#') == -1)
        parent = obj._path;
    if (obj._auxparent !== undefined)
        parent = obj._auxparent;

    if (Array.isArray(obj)) 
    {
        var out = [];
        for (var x in obj) {
            out[x] = inline_legacy_aux(obj[x], resolve, parent);
        }
        return out;
    }
    if (obj instanceof Object)
    {        
        var out = {};
        Object.keys(obj).forEach(function(key,index) {
            var value = obj[key];
            out[key] = inline_legacy_aux(value, resolve, parent);
        });  
        return out;
    }
    if (typeof obj === "string")
    {
        var idx = obj.indexOf("\#");
        if (idx > 0)
        {
            // full path
            obj = resolve(obj);
            obj = inline_legacy_aux(obj, resolve, parent);
        }
        else if (idx == 0)
        {
            // need to append path
            obj = resolve(parent + obj);
            obj = inline_legacy_aux(obj, resolve, parent);
        }        
    }
    return obj;
}


exports.load_tree_legacy = function(_path, result, result_file_set, user_types)
{
    console.log("loading legacy data.");
    tree_sync_legacy(_path, function(file) {        
        var json = parser.strip_comments(fs.readFileSync(file, "utf8"));
        var obj_path = path.relative(_path, file).replace(/\\/g, "/").replace(".json", "");
        var obj = fixup_legacy(eval("(" + json + ')'));

        var main = obj.data;
        main._path = obj_path;
        main._type = obj.type.toLowerCase();
        main._file = "main.txt";

        if (obj_path == "ui/cutscene/cutsceneroot")
        {
            console.log(obj.data);
        }

        if (user_types[main._type] === undefined) 
        {
            console.log("ignoring object with type " + main._type);
            return;
        }

        result[main._path] = main;

        if (obj.aux !== undefined)
        {
            for (var i=0;i<obj.aux.length;i++)
            {
                var aux = obj.aux[i].data;
                if (aux === undefined || obj.aux[i].type === undefined)
                    continue;

                aux._type = obj.aux[i].type.toLowerCase();

                if (user_types[aux._type] === undefined) 
                {
                    console.log("ignoring aux object with type " + main._type);
                    return;
                }                
                aux._path = obj_path + obj.aux[i].ref;
                aux._auxparent = obj_path;
                aux._file = "main.txt";
                result[aux._path] = aux;
            }
        }
    });

    console.log("Inlining aux objects");
    for (var x in result)
    {
        result[x] = inline_legacy_aux(result[x], function(path) {
            if (result[path] !== undefined) {
                var obj = result[path];
                delete obj._path;
                delete result[path];
                return obj;
            } else {
                console.log("Unable to resolve ", path);
                return null;
            }
        });
    }

    result_file_set["main.txt"] = true;
    return true;
}

exports.load_tree = function(_path, result, result_file_set)
{    
    tree_sync(_path, function(file) {
        var data = parser.strip_comments(fs.readFileSync(file, "utf8"));
        var pd = {
            data: data,
            pos: 0,
            error: false,
            result: result,
            file: path.relative(_path, file).replace(/\\/g, "/")
        };
        if (result_file_set)
            result_file_set[pd.file] = true;
        parser.parse(pd, true);
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
   data = parser.strip_comments(data);
   var pd = {
       data: data,
       pos: 0,
       error: false,
       file: fn,
       result: {}
   };
   console.log("retval=", parser.parse(pd, true));
};
