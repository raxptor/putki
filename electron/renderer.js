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
    }
}

var UserTypes = {
    "Character": {
        Fields: [
            { Name: "Name", Type:"String", Array:false },
            { Name: "Description", Type:"Text", Array:false },
            { Name: "HasInventory", Type:"Bool" },
            { Name: "Cucumber", Type:"Bool" },
            { Name: "Damage", Type:"DamageType", Array:false },
            { Name: "Mask", Type:"Mask", Array:false },
            { Name: "Immunities", Type:"DamageType", Array:true },
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
            { Name: "Characters", Type:"Character", ptr:true, Array:true }
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
    }    
}

function clone(src) {
    return JSON.parse(JSON.stringify(src));
}

function resolve_type(type)
{
    if (UserTypes[type] !== undefined)
        return UserTypes[type];
    return Primitive[type];
}

function default_value(field)
{
    var type = resolve_type(field.Type);
    if (field.Default !== undefined)
        return field.Default;
    if (type.Array)
        return [];
    if (type.Fields !== undefined)
        return {};
    if (type.Editor == "Int")
        return 0;
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
    var _array = document.createElement('table');

    if (iv === undefined)
        iv = [];
    
    for (var i=0;i<=iv.length;i++)
    {
        var _row = document.createElement('tr');
        if (i < iv.length)
        {
            var _idx = document.createElement('td');
            var _val = document.createElement('td');
            _idx.appendChild(document.createTextNode(i.toString()));
            _val.appendChild(create_type_editor({
                field: ed.field,
                data: iv,
                datafield: i
            }, true));
            _row.appendChild(_idx);
            _row.appendChild(_val);
            var ctl0 = document.createElement('td');
            var ctl1 = document.createElement('td');
            ctl0.appendChild(mk_button("delete", function(idx) { 
                return function() {
                    iv.splice(idx, 1);
                    _array._x_reload();
                }; } (i)
            ));
            ctl1.appendChild(mk_button("clone", function(idx) { 
                return function() {
                    iv.splice(idx, 0, clone(iv[idx]));
                    _array._x_reload();
                }; } (i)
            ));
            _row.appendChild(ctl1);
            _row.appendChild(ctl0);
        }
        else
        {
            var ctl0 = document.createElement('td');
            ctl0.colSpan = 2;
            ctl0.appendChild(mk_button("new", function() { 
                return function() {
                    iv.splice(iv.length, 0, default_value(ed.field));
                    _array._x_reload();
                }; } (i)));
            _row.appendChild(ctl0);
        }
        _array.appendChild(_row);
    }
    return _array;
}

function create_type_editor(ed, un_array)
{
    var type = resolve_type(ed.field.Type);
    var iv = ed.data[ed.datafield];
    if (!un_array && ed.field.Array)
    {
        return create_array_editor(ed);
    }

    if (type.Editor == "String")
    {
        var ip = document.createElement("input");
        ip.value = iv;
        return ip;
    }
    if (type.Editor == "Text")
    {
        var ta = document.createElement("textarea");
        ta.rows = 10;
        ta.value = iv;
        return ta;
    }
    if (type.Editor == "Checkbox")
    {
        var ta = document.createElement("input");
        ta.type = "checkbox";
        return ta;
    }
    if (type.Editor == "Select")
    {
        var sel = document.createElement("select");
        for (var i=0;i<type.Values.length;i++)
        {
            var opt = document.createElement("option"); 
            opt.text = type.Values[i].Name;
            opt.value = type.Values[i].Value;
            sel.options.add(opt);
            if (iv == type.Values[i].Name)
                sel.selectedIndex = i;
        }
        sel.addEventListener("change", function() {
            for (var x in type.Values)
            {
                if (sel.value == type.Values[x].Value)
                {
                    console.log("its value is ", type.Values[x].Name);
                    console.log(ed);
                    ed.data[ed.datafield] = type.Values[x].Name; 
                }
            }
        });
        return sel;
    }
    if (type.Fields !== undefined)
    {
        return build_inline_entry({
            type: ed.field.Type,
            data: iv
        });
    }
    return document.createTextNode("UNKNOWN TYPE " + ed);
}

function create_wrapped_editor(parent, ed)
{
    var obj = create_type_editor(ed);
    parent.appendChild(obj);
    obj._x_reload = function() {
        delete obj._x_reload;
        parent.removeChild(obj);
        create_wrapped_editor(parent, ed);
    };
}

function build_properties(objdesc)
{
    var _properties = document.createElement('x-properties');
    var type = resolve_type(objdesc.type);
    if (type.Fields)
    {
        for (var i=0;i<type.Fields.length;i++)
        {
            var _property = document.createElement('x-property');
            var f = type.Fields[i];
            var _prop_name = document.createElement('x-prop-name');
            _prop_name.appendChild(document.createTextNode(f.Name));
            var _prop_value = document.createElement('x-prop-value');
            create_wrapped_editor(_prop_value, {
                data: objdesc.data,
                field: f,
                datafield: f.Name
            });
            _property.appendChild(_prop_name);
            if (objdesc.data[f.Name] === undefined)
                _property.classList.add("no-value");
            else
                _property.appendChild(_prop_value);
            _properties.appendChild(_property);
        }
    }
    return _properties;
}

function build_inline_entry(objdesc)
{
    var _entry = document.createElement('x-inline-entry'); 
    if (objdesc.path !== undefined)
    {
        var _path = document.createElement('x-path');
        var _path_text = document.createTextNode("@" + objdesc.type + " " + objdesc.path);
        _path.appendChild(_path_text);
        _entry.appendChild(_path);
    }
    _entry.appendChild(build_properties(objdesc));
    return _entry;  
}  

function build_full_entry(objdesc)
{
    var _entry = document.createElement('x-entry'); 
    var _path = document.createElement('x-path');
    var _path_text = document.createTextNode("@" + objdesc.type + " " + objdesc.path);
    _path.appendChild(_path_text);
    _entry.appendChild(_path);
    _entry.appendChild(build_properties(objdesc));
    return _entry;  
}  

document.body.appendChild(build_full_entry({path: "gurka", type:"Character", data: {
    "Name": "Pervical Slusk",
    "Description": "A mastermind of deception",
    "Mask": { },
    "Immunities": [
        "DAMAGE_PHYSICAL",
        "DAMAGE_UBER"
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