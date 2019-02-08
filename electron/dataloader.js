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
