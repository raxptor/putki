var Dialogs = require("dialogs");
var dialogs = new Dialogs({});

var Primitive = {
    "String": {
        Editor: "String"
    },
    "Text": {
        Editor: "Text"
    },
    "Bool": {
        Editor: "Checkbox"
    },
    "Hash": {
        Editor: "Hash"
    }, 
    "Int": {
        Editor: "Int"
    },
    "I32": {
        Editor: "Int"
    },
    "U32": {
        Editor: "Int"
    },
    "Float": {
        Editor: "Int"
    },
    "U8": {
        Editor: "Int"
    }
}
/*
var UserTypes = {
    "Character": {
        Fields: [
            { Name: "Name", Type:"String", Array:false },
            { Name: "Description", Type:"Text", Array:false },
            { Name: "Skills", Type:"Skill", Array:true, Pointer:true},
            { Name: "HasInventory", Type:"Bool" },
            { Name: "Cucumber", Type:"Bool" },
            { Name: "Damage", Type:"DamageType", Array:false },
            { Name: "Mask", Type:"Mask", Array:false },
            { Name: "Immunities", Type:"DamageType", Array:true },
            { Name: "MultiMasks", Type:"Mask", Array:true },
        ]
    },
    "Skill": {
        Fields: [
            { Name: "Name", Type:"String", Array:false },
            { Name: "Description", Type:"String", Array:false },
        ]
    },
    "Mask": {
        Fields: [
            { Name: "Types", Type:"DamageType", Array:true },
            { Name: "PrimaryOnly", Type:"Bool" },
            { Name: "BranchOnly", Type:"Bool" },
            { Name: "RequireTargetTag", Type:"String" }
        ]
    },
    "Roster": {
        Fields: [
            { Name: "Name", Type:"String", Array:false },
            { Name: "Characters", Type:"Character", Pointer :true, Array:true }
        ]
    },
    "DamageType": {
        Editor: "Select",
        Values: [
            { Name: "DAMAGE_PHYSICAL", Value:1 },
            { Name: "DAMAGE_HEALING", Value:2 },
            { Name: "DAMAGE_UBER", Value:3 }
        ]
    }
}
*/
var Mod0 = require("/Users/dannilsson/git/oldgods/_gen/js/types.js");
console.log(Mod0);
var UserTypes = Mod0.Types;

var Data = {
    "roster0": {
        _type: "Roster",
        Name: "Roster Deluxe",
        Characters: ["characters/char1", "characters/char2"]
    },
    "characters/char1": {
        _type: "Character",
        Name: "My favorite guy",
        Description: "He is big and strong"  
    },
    "characters/char2": {
        _type: "Character",
        Name: "My second guy",
        Description: "He is small and weak" 
    },
    "skills/perc/sk1": {
        _inline: true,
        _type: "Skill",
        Name: "Lunge",
        Description: "Basic skill"
    },
    "skills/guard": {
        _type:"Skill",
        Name: "Guard",
        Description: "Basic skill"
    },
    "skills/perc/sk2": {
        _type: "Skill",
        Name: "Pin",
        Description: "Extended skill"
    }
}

var Obj2Path = {};

for (var x in Data)
    Obj2Path[Data[x]] = x;

function clone(src) {
    console.log("clone", src, JSON.stringify(src), " and reparse ", JSON.parse(JSON.stringify(src)));
    return JSON.parse(JSON.stringify(src));
}

function resolve_type(type)
{
    if (UserTypes[type] !== undefined)
        return UserTypes[type];
    return Primitive[type];
}

function default_value(field, un_array)
{
    if (field.Default !== undefined)
        return field.Default;
    if (field.Array && !un_array)
        return [];
    if (field.Pointer)
        return null;
    var type = resolve_type(field.Type);
    if (type.Fields !== undefined)
        return {};
    if (type.Editor == "Int")
        return 0;
    if (type.Editor == "String" || type.Editor == "Text" || type.Editor == "Hash")
        return "";
    if (type.Values !== undefined && type.Values.length > 0)
        return type.Values[type.Values.length-1].Value;
}

function mk_button(command, fn)
{
    var _input = document.createElement('input');
    _input.type = "submit";
    _input.id = command;
    _input.name = command;
    _input.value = command;
    _input.addEventListener("click", fn);
    return _input;
}

function create_array_editor(ed)
{
    var iv = ed.data[ed.field.Name];
    var _array = document.createElement('x-array');

    if (iv === undefined)
        iv = [];
    
    for (var i=0;i<=iv.length;i++)
    {   
        if (i < iv.length)
        {
            var _idx = document.createElement('x-array-index');
            var _val = document.createElement('x-array-value');
            _idx.appendChild(document.createTextNode(i.toString()));
            var _we = create_wrapped_editor({
                field: ed.field,
                data: iv,
                datafield: i
            }, true);
            _val.appendChild(_we.inline);
            _val.classList.add("value");
            _idx.style.gridRow = i+1;
            _val.style.gridRow = i+1;
            _array.appendChild(_idx);
            _array.appendChild(_val);
            var ctl0 = document.createElement('x-array-controls');
            ctl0.appendChild(mk_button("delete", function(idx) { 
                return function() {
                    iv.splice(idx, 1);
                    _array._x_reload();
                }; } (i)
            ));
            ctl0.appendChild(mk_button("clone", function(idx) { 
                return function() {
                    iv.splice(idx, 0, clone(iv[idx]));
                    _array._x_reload();
                }; } (i)
            ));
            ctl0.style.gridRow = i+1;
            _array.appendChild(ctl0);
        }
        else
        {
            var ctl0 = document.createElement('x-array-bottom-controls');
            ctl0.style.gridRow = i+1;
            ctl0.appendChild(mk_button("new", function() { 
                return function() {
                    iv.splice(iv.length, 0, default_value(ed.field, true));
                    _array._x_reload();
                }; } (i)));
            _array.appendChild(ctl0); 
        }
    }
    return _array;
}

function create_array_preview(arr)
{
    var has_objects = false;
    var pure = [];
    for (var i=0;i<arr.length;i++)
    {
        if (arr[i] instanceof Object)
        {
            if (arr[i]._path !== undefined)
                pure.push(arr[i]._path);
             else
                has_objects = true;
        }
        else
        {
            pure.push(arr[i]);
        }
    }
    if (has_objects)
    {
        return document.createTextNode(arr.length + " item(s)");
    }
    if (arr.length == 0)
    {
        return document.createTextNode("0 item(s)");
    }
    return document.createTextNode(pure.join(", "));
}

function create_pointer_editor(ed)
{
    var ptr = document.createElement("x-pointer"); 
    var ptrval = document.createElement("div");
    var iv = ed.data[ed.datafield];
    if (iv instanceof Object) {
        var inl = build_full_entry({
            type: iv._type,
            path: iv._path,
            data: iv
        }, function (new_path) {
            iv._path = new_path;
        });
        ptrval.appendChild(inl);
    } else {
        var txt = document.createTextNode(iv);
        ptrval.classList.add("pointer-text");
        ptrval.classList.add("click-to-change");
        ptrval.appendChild(txt);
        var data = Data[iv];
        if (data === undefined)
            ptrval.classList.add("invalid-pointer");
    }

    var btns = document.createElement("x-pointer-buttons");
    var ptrnew = mk_button("new", function() {
        ed.data[ed.datafield] = { _type: ed.field.Type };
        ptr._x_reload();
    });
    var ptrclear = mk_button("clear", function() {
        ed.data[ed.datafield] = null;
        ptr._x_reload();
    });
    btns.appendChild(ptrnew);
    btns.appendChild(ptrclear);
    ptr.appendChild(ptrval);
    ptr.appendChild(btns);
    return ptr; 
}

function create_type_editor(ed, un_array)
{
    var type = resolve_type(ed.field.Type);
    var iv = ed.data[ed.datafield];
    if (iv === undefined)
    {
        if (ed.field.Array)
        {
            ed.data[ed.datafield] = [];
            iv = [];
        }
        else if (!ed.field.Pointer && type.Fields !== undefined)
        {
            ed.data[ed.datafield] = {};
            iv = {};
        }
        else if (ed.field.Default !== undefined)
        {
            iv = ed.field.Default;
        }
        else if (type.Editor == "Bool")
        {
            iv = false;
        }
        else if (type.Editor == "Int")
        {
            iv = 0;
        }
        else if (ed.field.Pointer)
        {
            ed.data[ed.datafield] = null;
            iv = null;
        }
        else if (type.Editor == "String" || type.Editor == "Text" || type.Editor == "Hash")
        {
            iv = "";
        }
        else 
        {
            return {
                block: document.createElement("div"),
                inline: document.createTextNode("unknown")
            }
        }
    }

    if (!un_array && ed.field.Array)
    {
        return {
            block: create_array_editor(ed),
            inline: create_array_preview(iv)
        };
    }
    if (ed.field.Pointer)
    {
        return {
            inline: document.createTextNode(iv),
            block: create_pointer_editor(ed)
        }
    }

    if (type.Editor == "String" || type.Editor == "Hash")
    {
        var ip = document.createElement("input");
        ip.value = iv;
        ip.addEventListener("change", function() {
            ed.data[ed.datafield] = ip.value;
        });  
        return {
            inline: ip
        }
    }
    if (type.Editor == "Int")
    {
        var ip = document.createElement("input");
        ip.value = iv;
        ip.type = "number";
        ip.addEventListener("change", function() {
            ed.data[ed.datafield] = ip.value;
        });  
        return {
            inline: ip
        };
    } 
    if (type.Editor == "Text")
    {
        var ta = document.createElement("textarea");
        ta.rows = 10;
        ta.value = iv;
        ta.addEventListener("change", function() {
            ed.data[ed.datafield] = ta.value;
        }); 
        return {
            inline: ta
        };
    }
    if (type.Editor == "Checkbox")
    {
        var ta = document.createElement("input");
        ta.type = "checkbox";
        ta.checked = [true, "true", "True", "1"].indexOf(ed.data[ed.datafield]) != -1;
        ta.addEventListener("change", function() {
            ed.data[ed.datafield] = ta.checked;
        });    
        return {
            inline: ta
        }    
    }
    if (type.Values !== undefined)
    {
        var sel = document.createElement("select");
        for (var i=0;i<type.Values.length;i++)
        {   
            var opt = document.createElement("option"); 
            opt.text = type.Values[i].Name;
            opt.value = type.Values[i].Value;
            sel.options.add(opt);
            if (iv == type.Values[i].Name   )
                sel.selectedIndex = i;
        }
        sel.addEventListener("change", function() {
            for (var x in type.Values)
            {
                if (sel.value == type.Values[x].Value)
                {
                    ed.data[ed.datafield] = type.Values[x].Name; 
                }
            }
        });
        return {
            inline: sel
        }
    }
    if (type.Fields !== undefined)
    {
        return {
            inline: document.createTextNode("OBJECT"),
            block: build_inline_entry({
                type: ed.field.Type,
                data: iv
            })
        };
    }
    var el = document.createElement("input");
    el.value = "UNKNOWN TYPE " + ed.field.Type;
    return {
        inline: el
    };
//    return document.createTextNode("UNKNOWN TYPE " + ed);
}

function create_wrapped_editor(ed, un_array)
{
    return create_type_editor(ed, un_array);
    /*
    obj._x_reload = function() {
        var r = obj.style.gridRow;
        var p = obj.parentNode;
        delete obj._x_reload;
        p.removeChild(obj);
        var n = create_wrapped_editor(ed, un_array);
        n.style.gridRow = r;
        p.appendChild(n);
        return n;
    };
    return obj;*/
}

function create_wrapped_propname(parent, editor, info)
{
    var _prop_name = document.createElement('x-prop-name');
    _prop_name.appendChild(document.createTextNode(info.field.Name));
    if (info.data[info.datafield] === undefined)
    {
        parent.classList.add("no-value");
        _prop_name.classList.add("no-value");
        _prop_name.classList.add("click-to-add");
    }
    _prop_name.addEventListener("click", function() {
        if (info.data[info.datafield] === undefined)
        { 
            parent.classList.remove("no-value");
            _prop_name.classList.remove("no-value");
            _prop_name.classList.remove("no-value");
            info.data[info.datafield] = default_value(info.field);
            //editor = editor._x_reload();
            //delete editor.style.display;
        }
    });
    return _prop_name;
}

function build_properties(objdesc)
{
    var _properties = document.createElement('x-properties');
    var type = resolve_type(objdesc.type);
    var row = 1;
    if (type.Fields)
    {
        for (var i=0;i<type.Fields.length;i++)
        {
            var f = type.Fields[i];
            var dom = create_wrapped_editor({
                data: objdesc.data,
                field: f,
                datafield: f.Name
            });
            var _prop_name = create_wrapped_propname(_properties, null, {
                data: objdesc.data,
                field: f,
                datafield: f.Name
            });
            _prop_name.style.gridRow = row;
            _properties.appendChild(_prop_name);

            if (dom.inline)
            {
                var _prop_value = document.createElement('x-prop-value'); 
                _prop_value.appendChild(dom.inline);
                _prop_value.style.gridRow = row;
                _properties.appendChild(_prop_value);
            }
            if (dom.block)
            {
                row = row + 1; 
                dom.block.style.gridRow = row;
                _properties.appendChild(dom.block);
                if (dom.inline)
                {
                    dom.block.style.display = "none";
                }
            }
            row = row + 1;
        }
    }
    return _properties;
}

function build_inline_entry(objdesc)
{
    var _entry = document.createElement('x-inline-entry'); 
    /*
    if (objdesc.path !== undefined)
    {
        var _path = document.createElement('x-path');
        var _path_text = document.createTextNode("@" + objdesc.type + " " + objdesc.path);
        _path.appendChild(_path_text);
        _entry.appendChild(_path);
    }
    */
    _entry.appendChild(build_properties(objdesc));
    return _entry;  
}  

function build_full_entry(objdesc, on_new_path)
{
    var _entry = document.createElement('x-entry'); 
    var _path = document.createElement('x-path');
    var _type_text = document.createTextNode("@" + objdesc.type + " ");
    var _path_text = document.createTextNode(objdesc.path);
    _path.classList.add("click-to-change"); 
    _path.appendChild(_type_text);
    _path.appendChild(_path_text);
    _entry.appendChild(_path);
    _entry.appendChild(build_properties(objdesc));
    _path.addEventListener("click", function() {
        dialogs.prompt("Enter new path", objdesc.path, function (p) {
            if (p != null)
            {
                objdesc.path = p;
                _path_text.textContent = p;
                on_new_path(p);
            }
        });
    });
    return _entry;  
}  

document.body.appendChild(build_full_entry({path: "gurka", type:"Character", data: {
    "Name": "Pervical Slusk",
    "Description": "A mastermind of deception",
    "Mask": { },
    "Immunities": [
        "DAMAGE_PHYSICAL",
        "DAMAGE_UBER"
    ],
    "Tags": [ "c", "b", "a" ],
    "BaseStats": {},
    "MultiMasks": [],
    "Skills": [
        {
            Name: "Pin",
            Description: "Basic skill",
            _path: "skills/perc/pin",
            _type: "Skill"
        },
        "skills/superlong/text/that/fills",
        "more/than/fits",
        "this/one/is/long/too",
        "skills/guard",
        "skills/invalid"
    ]
}}));

/*
var fs = require('fs');
var path = "c:/git/oldgods/unity-proj/Assets/StreamingAssets/GameData"
fs.readdir(path, function(err, items) {
    console.log(items); 
    for (var i=0; i<items.length; i++) {
        console.log(items[i]);
    }
});
*/