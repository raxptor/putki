const fs = require('fs');
const path = require('path');
const fsextra = require('fs-extra');
const md5 = require('js-md5');
const exceljs = require('exceljs');

function str2bool(s)
{
    if (s.toLowerCase() == "true") return true;
    if (s == "1") return true;
    return false;
}

exports.do_import = function(filename, Data, Types, on_done)
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


    var stats = { changed: 0, new: 0, created:0 };
    function set_value(root, path, value, def_type)
    {
        if (typeof(root) === 'string' || root instanceof String)
        {
            root = DataExt[root];
        }
        if (root == undefined)
        {            
            return false;
        }

        var obj_type = def_type;
        if (root.hasOwnProperty('_type'))
            obj_type = root._type;

        var dot = path.indexOf('.');
        if (dot == -1)
        {
            var field = path.toLowerCase();
            if (root[field] != value)
            {
                var defVal = undefined;
                var isBool = false;
                var isDefault = false;
                if (obj_type !== undefined && Types[obj_type] !== undefined)
                {
                    var flds = Types[obj_type].ExpandedFields;
                    for (var x in flds)
                    {
                        if (flds[x].Name == field)
                        {
                            defVal = flds[x].Default;
                            isBool = flds[x].Type == "Bool";
                            if (defVal === undefined)
                                defVal = (flds[x].Type == "Bool") ? "false" : 0;
                            if (defVal == value)
                                isDefault = true;
                            if (flds[x].Type == "Bool" && (str2bool(defVal) == str2bool(value)))
                            {
                                isDefault = true;
                            }
                            break;
                        }
                    }
                }                
                if (root[field] === undefined)
                {                    
                    if (!isDefault)
                    {
                        console.log("NEW ." + field + " => [" + value + "] default=[" + defVal + "])");
                        root[field] = value;
                        stats.new++;                    
                    }
                }
                else
                {
                    if (isBool && (str2bool(root[field]) == str2bool(value)))
                    {
                        return true;
                    }

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

            var sub_type, value_type;
            if (obj_type !== undefined && Types[obj_type] !== undefined)
            {
                var flds = Types[obj_type].ExpandedFields;
                for (var x in flds)
                {
                    if (flds[x].Name === base)
                    {
                        sub_type = flds[x].Type;
                        value_type = Types[sub_type].IsValueType;
                        break;
                    }
                }
            }         

            if (root[base] === undefined && sub_type !== undefined)
            {
                console.log("Creating object because it was missing ", sub_type, " value_type=", value_type);
                root[base] = {};
                if (!value_type) root[base]._type = sub_type;
            }
            
            return set_value(root[base], extra, value, sub_type);
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
                    if (header.getCell(j).text.startsWith("!"))
                        continue;
                    var path = cr.getCell(1) + header.getCell(j);
                    var value = cr.getCell(j).value;
                    if (value)
                    {
                        if (!set_value(DataExt, path, value))
                        {
                            var bp = basePath.toString();
                            if (bp.length > 0 && bp.indexOf('.') == -1 && DataExt[bp] === undefined && ws.name[0] == '@')
                            {
                                console.log("Allowed to create new instance of ", ws.Name);
                                var new_obj = {
                                    _type: ws.name.substr(1).toLowerCase(),
                                    _path: bp,
                                    _file: "excel-imports.txt"
                                };
                                DataExt[bp] = new_obj;
                                Data[bp] = new_obj;
                                stats.created++;
                                if (!set_value(DataExt, path, value))                                                                
                                {
                                    console.log("Failed to set value at path ", path);
                                    fail++;       
                                }
                            }
                            else
                            {
                                console.log("Failed to set value at path ", path);
                                fail++;
                            }
                        }
                        else
                        {
                            succ++;
                        }
                    }
                }
           }
        }
        on_done("Rows processed:" + rows + ". NewFields:" + stats.new + " ObjCreated:" + stats.created + " FieldUpdates:" + stats.changed + " Failed updates:" + fail + ". See log for details");
    });
    

}