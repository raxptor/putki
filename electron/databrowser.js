const { ipcRenderer } = require('electron');
var popups = require('./popups');
var Dialogs = require("dialogs");
var dialogs = new Dialogs({});

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

exports.create = function(onto, types, data, plugins, config, data_browser_preview, start_editing) {
    var base = document.createElement('x-browser');

    var form = document.createElement('form');
    var filter = document.createElement('input');
    filter.type = "text";
    form.appendChild(filter);
    base.appendChild(form);
    var grid = null;
    var fn_map = {};

    var pick = null;

    var rebuild = function(deep_dig) {
        if (grid)
            base.removeChild(grid);
        grid = document.createElement('x-browser-objlist');
        fn_map = {};
        for (var x in data) {
            var fn = data[x]._file || "new.txt";
            if (fn_map[fn] === undefined) {
                var hdr = document.createElement('x-browser-file');
                hdr.appendChild(document.createTextNode(fn));
                var controls = document.createElement('x-browser-controls');
                fn_map[fn] = {
                    header: hdr,
                    controls: controls,
                    items: []
                };
                (function(fn) {
                    controls.appendChild(mk_button("New instance", function() {
                        popups.ask_type(types, null, function(which) { 
                            dialogs.prompt("Enter path", "example/path", function (p) {
                                if (p != null)
                                {
                                    data[p.toLowerCase()] = {
                                        _path: p.toLowerCase().replace( /\\/g, '/').replace(' ', '-'),
                                        _type: which,
                                        _file: fn
                                    };
                                    rebuild();
                                }
                            });
                        });
                    }));
                })(fn);
            }
            var e = fn_map[fn];
            var path = document.createElement('x-browser-path');
            path.appendChild(document.createTextNode(data[x]._path));
            path.style.gridRow = e.count;
            var type = document.createElement('x-browser-type');
            type.appendChild(document.createTextNode("@" + types[data[x]._type].PrettyName));
            type.style.gridRow = e.count;
            var preview = document.createElement('x-browser-preview');
            //if (data_browser_preview !== null) {
            {
                preview.appendChild(document.createTextNode(data_browser_preview(data[x])));
                preview.style.gridRow = e.count;
            }
            (function(_path) {
                path.addEventListener('click', function() {
                    start_editing(_path);
                });
                path.addEventListener('contextmenu', function() {
                    var opts = [
                        { Title:"Edit", Data: { command: "edit", path: _path } },
                        { Title:"Move", Data: { command: "move", path: _path } },                        
                        { Title:"Delete", Data: { command: "delete", path: _path } }
                    ];                    
                    for (var i=0;i<plugins.length;i++) {
                        var editors = plugins[i].object_editors;
                        for (var j=0;j<editors.length;j++)
                        {
                            if (editors[i].types.indexOf(data[_path]._type) != -1)
                            {
                                opts.push( { Title:"Edit with " + editors[j].description, Data: { command: "plugin-edit", path:_path, plugin:i, editor:j } } );
                            }
                        }
                    }
                    ipcRenderer.send('choose-menu', opts);
                });
            })(data[x]._path);
            e.items.push({ path:data[x]._path, type:data[x]._type, elements: [path, type, preview] });

            var anchor_counter = 0;
            if (deep_dig) {
                var dig = function(root_path, d, type_name, skip) {
                    if (d === null || d === undefined)
                        return;
                    var t = type_name;
                    if (d.hasOwnProperty('_type'))
                        t = d._type;
                    if (d instanceof Array) {
                        for (var i=0;i<d.length;i++)
                            dig(root_path, d[i], t);
                    } else if (d instanceof Object) {
                        if (!skip && t == deep_dig.type) {
                            var path = document.createElement('x-browser-dig-path');
                            var spath;
                            var anchor = null;
                            if (d.hasOwnProperty("_path"))
                            {
                                spath = d._path;
                                d["_anchor"] = spath;
                                anchor = spath;
                            }
                            else
                            {
                                anchor = root_path + "!" + anchor_counter;
                                spath = root_path + "#embed-" + anchor_counter;
                                d["_anchor"] = anchor;
                                ++anchor_counter;
                            }   
                            path.appendChild(document.createTextNode(spath));
                            path.style.gridRow = e.count;
                            path.addEventListener('click', function() {
                                start_editing(root_path, anchor);
                            });                            
                            var type = document.createElement('x-browser-type');
                            type.appendChild(document.createTextNode("@" + types[t].PrettyName));
                            type.style.gridRow = e.count;
                            var preview = document.createElement('x-browser-preview');
                            {
                                preview.appendChild(document.createTextNode(data_browser_preview(d)));
                                preview.style.gridRow = e.count;
                            }
                            e.items.push({ path:root_path, anchor: spath, type:t, elements: [path, type, preview] });
                        }
                        var flds = types[t].ExpandedFields;
                        for (var i=0;i<flds.length;i++)
                        {
                            if (d.hasOwnProperty(flds[i].Name))
                                dig(root_path, d[flds[i].Name], flds[i].Type);
                        }        
                    }
                }
                dig(data[x]._path, data[x], data[x]._type, true);
            }
        }
        var count = 0;        
        for (var x in fn_map) {
            var e = fn_map[x];
            e.header.style.gridRow = ++count;
            grid.appendChild(e.header);            
            for (var i in e.items) {
                var row = ++count;
                var els = e.items[i].elements;                
                for (var j in els) {
                    els[j].style.gridRow = row;
                    grid.appendChild(els[j]);
                }
            }
            e.controls.style.gridRow = ++count;
            grid.appendChild(e.controls);            
        }
        filtrate();
        base.appendChild(grid);        
    };
    rebuild();
    if (onto != null)
        onto.appendChild(base);
    filter.focus();

    function filtrate() {
        var search = filter.value.toLowerCase();
        var totFound = 0;
        var last = null;
        var type_search = "^";
        if (search.startsWith("@")) {
            type_search = search.substr(1);
            search = "^";
        }
        for (var x in fn_map) {
            var e = fn_map[x];            
            e.header.classList.remove('hidden');        
            e.controls.classList.remove('hidden');        
            var found = 0;
            for (var i in e.items) {
                var els = e.items[i].elements;
                for (var j in els) {
                    els[j].classList.remove('hidden');
                }
                if (search.length > 0 && e.items[i].path.indexOf(search) == -1 && x.indexOf(search) == -1 && e.items[i].type.indexOf(type_search) == -1)
                {
                    for (var j in els) {
                        els[j].classList.add('hidden');
                    }
                }
                else
                {
                    last = e.items[i].path;
                    totFound++;
                    found++;
                }
            }
            if (found == 0)
            {
                e.header.classList.add('hidden');
                e.controls.classList.add('hidden');
            }
        }        

        if (totFound == 1)
            pick = last;
        else
            pick = null;
    }

    filtrate();

    form.onsubmit = function(event) {
        event.preventDefault();
        if (filter.value.startsWith("@")) {
            rebuild({ type: filter.value.substring(1).toLowerCase() });
            filtrate();
        } else {         
            if (pick != null) {
                start_editing(pick);
            }
        }
    };

    filter.addEventListener("input", function() { 
        setTimeout(filtrate, 10)
    });
    base._x_reload = function() {
        rebuild();
    };
    return base;
}