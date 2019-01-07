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

exports.create = function(types, data, config) {
    var base = document.createElement('x-browser');    
    var rebuild = function() {  
        while (base.firstChild) {
            base.removeChild(base.firstChild);
        }
        var grid = document.createElement('x-browser-objlist');
        var fn_map = {};
        console.log(data);
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
                            dialogs.prompt("Enter new path", "example/path", function (p) {
                                if (p != null)
                                {
                                    data[p.toLowerCase()] = {
                                        _path: p.toLowerCase().replace('\\', '/').replace(' ', '-'),
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
            type.appendChild(document.createTextNode(data[x]._type));
            type.style.gridRow = e.count;
            e.items.push({ elements: [path, type] });
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
        base.appendChild(grid);        
    };
    rebuild();    
    return base;
}