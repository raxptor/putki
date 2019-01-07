function ask_with_filter(make_contents, on_done)
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
    var pick = null;

    form.onsubmit = function(event) {
        event.preventDefault();
        document.body.removeChild(popup);
        if (pick)
        {
            on_done(pick.data);
        }
    };

    var build = function() {
        while (listBox.firstChild) {
            listBox.removeChild(listBox.firstChild);
        }
        var fstr = filter.value;
        var filtered = make_contents(fstr);
        pick = null;
        for (var idx in filtered)
        {
            var tp = filtered[idx];
            var typeBox = document.createElement('x-type-box');
            if (filtered.length == 1 || tp.exact)
            {
                typeBox.classList.add("only-one");
                pick = tp;
            }
            var nm = document.createTextNode(tp.display);
            typeBox.appendChild(nm);
            listBox.appendChild(typeBox);
            (function(type) {
                typeBox.onclick = function() {
                    document.body.removeChild(popup);
                    on_done(type.data);
                }
            })(tp);
        }
        return filtered;
    };

    var types = build();
    if (types.length == 1) {
        on_done(types[0].data);
        return;
    }

    filter.addEventListener("input", function() { 
        setTimeout(build, 10)
    });

    content.appendChild(listBox);
    document.body.appendChild(popup);
    filter.focus();
}

function compatible_types(alltypes, type_name_root)
{
    var list = [];
    for (var tp in alltypes)
    {
        if (!alltypes[tp].PermitAsAsset)
            continue;
        var pr = tp;
        while (pr)
        {
            pr = alltypes[pr].Parent;
            if (pr == type_name_root || tp == type_name_root)
            {
                list[tp] = alltypes[tp];
                pr = null;
            }
        }
    }
    list.sort();
    console.log(list);
    return list;
}

exports.ask_type = function(alltypes, type_name_root, on_done)
{
    var types = compatible_types(alltypes, type_name_root);
    ask_with_filter(function(fstr) {        
        var filtered = [];
        var lower = fstr.toLowerCase();
        for (var idx in types)
        {
            var tp = types[idx];
            if (fstr.length > 0 && idx.indexOf(lower) == -1)
                continue;
            filtered.push({
                data: idx,
                exact: idx == lower,
                display: '@' + tp.PrettyName
            });
        }
        return filtered;
    }, on_done);
}

exports.ask_instance = function(alltypes, alldata, type_name_root, on_done)
{
    var types = compatible_types(alltypes, type_name_root);
    var potential = {};

    var dig = function(d) {
        if (d.hasOwnProperty('_path') && d.hasOwnProperty('_type')) {
            if (types.hasOwnProperty(d._type)) {
                potential["#" + d._path] = d;
            }
        }
        if (d instanceof Array) {
            for (var i=0;i<d.length;i++)
                dig(d[i]);
        } else if (d instanceof Object) {
            for (var i in d)
                dig(d[i]);
        }
    }
    for (var idx in alldata)
    {
        if (alldata[idx]._path !== undefined) {
            if (types.hasOwnProperty(alldata[idx]._type)) {
                potential[idx] = alldata[idx];
            }
        }
        var k = alldata[idx];
        for (var i in k)
            dig(k[i]);
    }
    ask_with_filter(function(fstr) {
        var filtered = [];
        var lower = fstr.toLowerCase();
        for (var idx in potential)
        {            
            var pth = potential[idx]._path.toLowerCase();
            if (fstr.length > 0 && pth.indexOf(lower) == -1)
                continue;
            filtered.push({
                data: pth,
                exact: fstr == pth,
                display: idx
            });
        }
        return filtered;
    }, on_done);
}

