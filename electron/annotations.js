exports.format = function(inp)
{
    var tmp = "";
    for (var x in inp)
    {
        if (tmp.length > 0) tmp += "\n";
        tmp += inp[x].Text;
    }
    return tmp;
}

exports.makeTinyObject = function(inp)
{
    var el = document.createElement("span");
    el.appendChild(document.createTextNode("*"));
    el.setAttribute("title", exports.format(inp));
    el.classList.add("tiny-annotation");
    return el;
}

exports.addAnnotation = function(el, inp)
{
    if (inp !== undefined && inp.length > 0) 
    {
        el.setAttribute("title", exports.format(inp));
        el.classList.add("has-annotation");
    }
}

exports.makeAnnotationBox = function(inp)
{
    var el = document.createElement("x-annotation");
    el.appendChild(document.createTextNode(exports.format(inp)));
    return el;
}