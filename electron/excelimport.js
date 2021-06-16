const fs = require('fs');
const path = require('path');
const fsextra = require('fs-extra');
const md5 = require('js-md5');
const exceljs = require('exceljs');

exports.do_import = function(filename, Data, on_done)
{
    console.log("doing import of ", filename);    

    var DataExt = {};
    var dig = function(d) {
        if (d === null || d === undefined)
            return;
        if (d.hasOwnProperty('_path')) {
            DataExt[d._path] = d;
        }
        if (d instanceof Array) {
            for (var i=0;i<d.length;i++)
                dig(d[i]);
        } else if (d instanceof Object) {
            for (var i in d)
                dig(d[i]);
        }
    }    
    dig(Data);
    for (var x in Data)
    {
        DataExt[x] = Data[x];
    }


    var stats = { changed: 0 };
    function set_value(root, path, value)
    {
        if (typeof(root) === 'string' || root instanceof String)
        {
            root = DataExt[root];
        }
        if (root == undefined)
        {
            return false;
        }
        var dot  = path.indexOf('.');
        if (dot == -1)
        {
            var field = path.toLowerCase();
            if (root[field] != value)
            {
                if (root[field] === undefined)
                {
                    console.log("NEW ." + field + " => [" + value + "]");
                }
                else
                {
                    console.log("UPDATE ." + field + " [" + root[field] + "] => [" + value + "]");
                    root[field] = value;
                    stats.changed++;
                }
            }
            return true;
        }
        else
        {
            var base = path.substr(0, dot).toLowerCase();
            var extra = path.substr(dot + 1);
            return set_value(root[base], extra, value);
        }
    }

    
    const wb = new exceljs.Workbook();    
    wb.xlsx.readFile(filename).then((workbook) => {
        var succ=0, fail=0, rows=0;
        if (!workbook)
        {
            on_done("Failed to open file.")
            return;
        }
        for (var sh in workbook.worksheets)
        {
            var ws = workbook.worksheets[sh];
            console.log("Processing sheet ", ws.name);
            var header = ws.getRow(1);            
            for (var i=2;i<=ws.rowCount;i++)
            {
                var cr = ws.getRow(i);                
                var basePath = cr.getCell(1);
                if (basePath.toString().startsWith("!"))
                    continue;
                rows++;
                for (var j=2;j<=header.cellCount;j++)
                {
                    var path = cr.getCell(1) + header.getCell(j);
                    var value = cr.getCell(j).value;
                    if (value)
                    {
                        if (!set_value(DataExt, path, value))
                        {
                            console.log("Failed to set value at path ", path);
                            fail++;
                        }
                        else
                        {
                            succ++;
                        }
                    }
                }
           }
        }
        on_done("Rows processed:" + rows + ". Updates:" + stats.changed + " Failed updates:" + fail + ". See log for details");
    });
    

}