const { ipcRenderer } = require('electron');
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

for (var xx in UserTypes)
{
    (function (x) {
        var all = [];
        var add = function(tn) {
            all = all.concat(UserTypes[tn].Fields);
            if (UserTypes[tn].Parent !== undefined)
                add(UserTypes[tn].Parent);
        };
        if (UserTypes[x].Fields !== undefined)
        {
            add(x);
            UserTypes[x].ExpandedFields = all;
        }
    })(xx);
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

function default_value(field, is_array_element)
{
    if (field.Default !== undefined)
        return field.Default;
    if (field.Array && !is_array_element)
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

function create_array_editor(ed, args)
{
    var expanded = (args || {}).expanded || [];
    var iv = ed.data[ed.field.Name];
    var _array = document.createElement('x-array');

    if (iv === undefined)
    {
        iv = [];
        ed.data[ed.field.name] = iv;
    } 
    
    var row = 1;
    for (var i=0;i<=iv.length;i++)
    {   
        if (i < iv.length)
        {
            var ctl0 = document.createElement('x-array-controls');
            ctl0.appendChild(mk_button("clone", function(idx) { 
                return function() {
                    iv.splice(idx, 0, clone(iv[idx]));
                    on_inline_changed(_array._x_reload());
                }; } (i)
            ));
            ctl0.appendChild(mk_button("delete", function(idx) { 
                return function() {
                    iv.splice(idx, 1);
                    on_inline_changed(_array._x_reload());
                }; } (i)
            ));
            ctl0.style.gridRow = row;
            var _idx = document.createElement('x-prop-index');
            _idx.appendChild(document.createTextNode(i.toString()));
            _idx.style.gridRow = row;
            _array.appendChild(_idx);
            row = create_property(_array, row, {
                data: ed.data[ed.datafield],
                field: ed.field,
                datafield: i
            }, true, expanded.indexOf(i) != -1);    
            _array.appendChild(ctl0); 
        }
        else
        {
            var ctl0 = document.createElement('x-array-bottom-controls');
            ctl0.style.gridRow = row;
            ctl0.appendChild(mk_button("add empty", function() { 
                return function() {
                    iv.splice(iv.length, 0, default_value(ed.field, true));
                    on_inline_changed(_array._x_reload({ expanded: [iv.length-1] }));
                }; } (i)));
            _array.appendChild(ctl0); 

            if (ed.field.Pointer) {
                var ptrnew = mk_button("add inst", function() {
                    ask_type(ed.field.Type, function(seltype) {
                        iv.splice(iv.length, 0, { _type: seltype });
                        on_inline_changed(_array._x_reload({ expanded: [iv.length-1] }));
                    });
                });
                ctl0.appendChild(ptrnew);
            }        
        }
    }
    return _array;
}

function create_pointer_preview(object)
{
    var descs = [];
    if (object instanceof Object)
    {
        if (object._type !== undefined)
            descs.push("@" + object._type);
        if (object._path !== undefined)
            descs.push(object._path);
        if (object._type !== undefined)
            descs.push(create_object_preview_txt(object, resolve_type(object._type)));
    }
    else
    {
        if (object === undefined || object === null)
            descs.push("null");
        else
            descs.push(object);   
    }
    var tn = document.createTextNode(descs.join(' '));
    return tn;
}

function create_object_preview_txt(object, type)
{
    var txts = [];
    for (var x in type.ExpandedFields)
    {
        var fn = type.ExpandedFields[x].Name;
        var val = object[fn];
        if (val === undefined || val === null || (val instanceof Array && val.length == 0) ||
            (val instanceof Object && Object.keys(val).length === 0))
            continue;
        if (val instanceof Object)
        {
            var t = resolve_type(type.ExpandedFields[x].Type);
            txts.push(fn + ":{" + create_object_preview_txt(val, t)+ "}");
        }
        else
        {
            if (val instanceof String)
                txts.push(fn + "=" + "\"" + val + "\"");
            else
                txts.push(fn + "=" + val.toString());
        }
    }
    if (txts.length == 0)
        return "<default>";
    else
        return txts.join(', ');
}

function reload_wrapped(new_fn)
{
    var preview = new_fn();
    preview._x_reload = function(args) {
        var pn = preview.parentNode;
        var neue = new_fn(args);
        neue._x_reload = preview._x_reload;
        neue._x_changed = preview._x_changed;
        neue.classList = preview.classList;
        if (preview.style)
            neue.style.gridRow = preview.style.gridRow;
        delete preview._x_reload;
        pn.removeChild(preview);
        if (preview._x_reloaded)
            preview._x_reloaded(neue);
        preview = neue; 
        pn.appendChild(preview);
        return preview;
    };
    return preview;
}

function on_inline_changed(node)
{
    while (node !== null)
    {
        var next = node.parentNode;
        if (node._x_changed)
        {
            console.log("Change handeled by ", node);
            node._x_changed();
        }
        node = next;
    }
    console.log("No change handler ", node);
}

function create_object_preview(object, type)
{
    var tn = document.createTextNode(create_object_preview_txt(object, type));
    tn._x_is_preview = true;
    return tn;
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
            if (arr[i] == null)
                pure.push("null");
            else
                pure.push(arr[i]);
        }
    }
    if (has_objects)
    {
        return document.createTextNode(arr.length + " item(s)");
    }
    if (arr.length == 0)
    {
        return document.createTextNode("<empty>");
    }
    return document.createTextNode("[" + pure.join(", ") + "]");
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
        /*
        var txt = document.createTextNode(iv);
        ptrval.appendChild(txt);
        var data = Data[iv];
        */
    }

    var btns = document.createElement("x-pointer-buttons");
    var ptrnew = mk_button("new", function() {
        ask_type(ed.field.Type, function(seltype) {
            ed.data[ed.datafield] = { _type: seltype };
            on_inline_changed(ptr._x_reload());    
        });
    });
    var ptrclear = mk_button("clear", function() {
        ed.data[ed.datafield] = null;
        on_inline_changed(ptr._x_reload());
    });
    btns.appendChild(ptrnew);
    btns.appendChild(ptrclear);
    ptr.appendChild(ptrval);
    ptr.appendChild(btns);
    return ptr; 
}

function set_default(obj, field, val)
{
    if (obj[field] === undefined || obj[field] === null)
        obj[field] = val;
}

function def_arr(desc) 
{
    set_default(desc.data, desc.datafield, []);
}

function def_obj(desc) 
{
    set_default(desc.data, desc.datafield, {});
}

function create_type_editor(ed, is_array_element)
{
    var type = resolve_type(ed.field.Type);
    if (!is_array_element && ed.field.Array)
    {
        return {
            block: reload_wrapped(function(args) { def_arr(ed); return create_array_editor(ed, args); }),
            inline: reload_wrapped(function(args) { def_arr(ed); return create_array_preview(ed.data[ed.datafield], args) })
        };
    }
    if (ed.field.Pointer)
    {
        var preview = reload_wrapped(function() { return create_pointer_preview(ed.data[ed.datafield]); });
        preview._x_context_menu = function() {
            ipcRenderer.send('edit-pointer', 'ping');
        };
        return {
            inline: preview,
            block: reload_wrapped(function() { return create_pointer_editor(ed); })
        }
    }

    if (type.Editor == "String" || type.Editor == "Hash")
    {
        return {
            inline: reload_wrapped(function() {
                var ip = document.createElement("input");
                if (ed.data[ed.datafield] !== undefined)
                    ip.value = ed.data[ed.datafield];
                else
                    ip.value = default_value(ed.field, is_array_element);
                ip.addEventListener("change", function() {
                    ed.data[ed.datafield] = ip.value;
                    on_inline_changed(ip);
                });  
                return ip;         
            })
        }
    }
    if (type.Editor == "Int")
    {
        return {
            inline: reload_wrapped(function() {
                var ip = document.createElement("input");
                var val = ed.data[ed.datafield];
                if (val === undefined)
                    ip.value = default_value(ed.field, is_array_element);
                else
                    ip.value = val;
                ip.type = "number";
                ip.addEventListener("change", function() {
                    ed.data[ed.datafield] = ip.value;
                    on_inline_changed(ip);
                });  
                return ip;
            })
        }
    } 
    if (type.Editor == "Text")
    {
        var ta = document.createElement("textarea");
        ta.rows = 10;
        ta.value = iv;
        ta.addEventListener("change", function() {
            ed.data[ed.datafield] = ta.value;
            on_inline_changed(ta);
        }); 
        return {
            inline: ta
        };
    }
    if (type.Editor == "Checkbox")
    {
        return {
            inline: reload_wrapped(function() {
                var ta = document.createElement("input");
                ta.type = "checkbox";
                var val = ed.data[ed.datafield];
                if (val === undefined)
                    val = default_value(ed.field, is_array_element);
                ta.checked = [true, "true", "True", "1"].indexOf(val) != -1;
                ta.addEventListener("change", function() {
                    ed.data[ed.datafield] = ta.checked;
                    on_inline_changed(ta); 
                });  
                return ta;
            })
        };
    }
    if (type.Values !== undefined)
    {
        return {
            inline: reload_wrapped(function() {
                console.log("reloading select");
                var sel = document.createElement("select");
                for (var i=0;i<type.Values.length;i++)
                {   
                    var opt = document.createElement("option"); 
                    opt.text = type.Values[i].Name;
                    opt.value = type.Values[i].Value;
                    sel.options.add(opt);
                    if (ed.data[ed.datafield] == type.Values[i].Name)
                        sel.selectedIndex = i;
                }
                sel.addEventListener("change", function() {
                    for (var x in type.Values)
                    {
                        if (sel.value == type.Values[x].Value)
                        {
                            ed.data[ed.datafield] = type.Values[x].Name; 
                            on_inline_changed(sel);
                        }
                    }
                });
                return sel;
            })
        }
    }
    if (type.Fields !== undefined)
    {
        return {
            inline: reload_wrapped(function() { def_obj(ed); return create_object_preview(ed.data[ed.datafield], type) } ),
            block: reload_wrapped(function() { def_obj(ed); return build_block_entry({
                type: ed.field.Type,
                data: ed.data[ed.datafield]
            })})
        };
    }
    var el = document.createElement("input");
    el.value = "UNKNOWN TYPE " + ed.field.Type;
    return {
        inline: el
    };
//    return document.createTextNode("UNKNOWN TYPE " + ed);
}

// returns row
function create_property(parent, row, objdesc, is_array_element, expanded)
{
    var update_label = function() { };
    var dom = create_type_editor(objdesc, is_array_element);
 
    if (!is_array_element)
    {
        var _prop_name = document.createElement('x-prop-name');
        _prop_name.appendChild(document.createTextNode(objdesc.field.Name));
        _prop_name.style.gridRow = row;
        parent.appendChild(_prop_name);
        update_label = function() {
            console.log("update label");
            _prop_name.classList.remove("no-value");
            var cv = objdesc.data[objdesc.datafield];
            if (cv === undefined || cv === null || (cv instanceof Array && cv.length == 0) ||
                (cv instanceof Object && Object.keys(cv).length === 0))
                _prop_name.classList.add("no-value");
        };
        _prop_name.addEventListener("contextmenu", function() {
            var value = objdesc.data[objdesc.datafield];
            if (value instanceof Array)
            {
                value.splice(0, value.length);
            }
            else if (value instanceof Object)
            {
                for (var x in value)
                    delete value[x];
            }
            else
            {
                delete objdesc.data[objdesc.datafield];
            }
            if (dom.inline && dom.inline._x_reload)
                dom.inline._x_reload();
            if (dom.block && dom.block._x_reload)
                dom.block._x_reload();
            update_label();
        });
        update_label();
    }

    var _prop_value;
    if (dom.inline)
    {
        _prop_value = document.createElement('x-prop-value'); 
        _prop_value.appendChild(dom.inline);
        _prop_value.style.gridRow = row;
        if (is_array_element)
            _prop_value.classList.add("array-element");
        _prop_value.addEventListener("contextmenu", function() {
            if (dom.inline._x_context_menu)
                dom.inline._x_context_menu();
        });
        parent.appendChild(_prop_value);
        dom.inline._x_reloaded = function(neue) {
            neue._x_reloaded = dom.inline._x_reloaded;
            delete dom.inline._x_reloaded;
            dom.inline = neue;
        } 
    }
    if (dom.block)
    {
        row = row + 1; 
        dom.block.style.gridRow = row;
        if (is_array_element)
        {
            dom.block.classList.add("array-element");
        }
        dom.block.classList.add("block-editor");
        parent.appendChild(dom.block);
        dom.block._x_changed = function() { console.log(";aa"); };
        if (dom.inline)
        {
            if (!expanded)
                dom.block.classList.add("collapsed");
            _prop_value.classList.add("click-to-expand");
            _prop_value.addEventListener("click", function() {
                dom.block.classList.toggle("collapsed");
            });
            if (dom.inline._x_is_preview)
                _prop_value.classList.add("preview");
            // reload the preview if content changes
            dom.block._x_reloaded = function(neue) {
                neue._x_reloaded = dom.block._x_reloaded;
                delete dom.block._x_reloaded;
                dom.block = neue;
            }
            dom.block._x_changed = function() {
                update_label();
                console.log("BLOCK CHANGED val=", objdesc.data[objdesc.datafield], " from ", objdesc);
                if (dom.inline._x_reload) {
                    dom.inline = dom.inline._x_reload();
                }
            }
        }
    }
    else
    {
        dom.inline._x_changed = function() {
            update_label();
        };
    }
    row = row + 1;
    return row;
}

function build_properties(objdesc)
{
    var _properties = document.createElement('x-properties');
    var row = 1;
    var insert_fn = function(typename) {
        var type = resolve_type(typename);
        if (type.Fields)
        {
            for (var i=0;i<type.Fields.length;i++)
            {
                var f = type.Fields[i];
                row = create_property(_properties, row, {
                    data: objdesc.data,
                    field: f,
                    datafield: f.Name
                }); 
            }
            if (type.Parent !== undefined)
                insert_fn(type.Parent);
        }
    }
    insert_fn(objdesc.type);
    return _properties;
}

function build_block_entry(objdesc)
{
    var _entry = document.createElement('x-inline-entry'); 
    var inline = build_properties(objdesc);
    _entry.appendChild(inline);
    _entry._x_changed = function() {
        console.log("object changed!");
    };
    return _entry;  
}  

function build_full_entry(objdesc, on_new_path)
{
    var _entry = document.createElement('x-entry'); 
    var _path = document.createElement('x-path');
    var _type_text = document.createTextNode("@" + objdesc.type + " ");
    var _path_text = document.createTextNode(objdesc.path !== undefined ? objdesc.path : "<anonymous>");
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
                on_inline_changed(_path);
            }
        });
    });
    return _entry;  
}  

function compatible_types(type_name_root)
{
    var list = [];
    for (var tp in UserTypes)
    {
        if (!UserTypes[tp].PermitAsAsset)
            continue;
        var pr = tp;
        while (pr)
        {
            pr = UserTypes[pr].Parent;
            if (pr == type_name_root || tp == type_name_root)
            {
                list.push(tp);
                pr = null;
            }
        }
    }
    list.sort();
    console.log(list);
    return list;
}

function ask_type(type_name_root, on_done)
{
    var popup = document.createElement('div');
    popup.classList.add("modal");
    var content = document.createElement('x-popup-type');
    content.classList.add("modal-content");
    popup.appendChild(content);

    var form = document.createElement('form');
    var filter = document.createElement('input');
    form.appendChild(filter);
    content.appendChild(form);

    var listBox = document.createElement('x-type-select-box');
    var types = compatible_types(type_name_root);
    var pick = null;

    form.onsubmit = function(event) {
        event.preventDefault();
        document.body.removeChild(popup);
        if (pick)
        {
            on_done(pick);
        }
    };


    var build = function() {
        while (listBox.firstChild) {
            listBox.removeChild(listBox.firstChild);
        }
        var fstr = filter.value.toLowerCase();
        console.log("filtering on ", fstr);

        var filtered = [];
        for (var idx in types)
        {
            var tp = types[idx];
            if (fstr.length > 0 && tp.toLowerCase().indexOf(fstr) == -1)
                continue;
            filtered.push(tp);
        }

        pick = null;
        for (var idx in filtered)
        {
            var tp = filtered[idx];
            var t = UserTypes[tp];
            var typeBox = document.createElement('x-type-box');
            if (filtered.length == 1 || tp.toLowerCase() == fstr)
            {
                typeBox.classList.add("only-one");
                pick = tp;
            }
            var nm = document.createTextNode('@' + tp);
            typeBox.appendChild(nm);
            listBox.appendChild(typeBox);
            (function(type) {
                typeBox.onclick = function() {
                    document.body.removeChild(popup);
                    on_done(type);
                }
            })(tp);
        }
        return filtered;
    };

    var types = build();
    if (types.length == 1) {
        on_done(types[0]);
        return;
    }

    filter.addEventListener("input", function() { 
        setTimeout(build, 10)
    });

    content.appendChild(listBox);
    document.body.appendChild(popup);
    filter.focus();
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
    "BaseStats": {
        "Initiative": 4,
        "SanityPool": 400   
    },
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


document.body.appendChild(build_full_entry({path: "CueCumber", type:"AtkModifyDamage", data: {
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