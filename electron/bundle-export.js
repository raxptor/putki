var fs = require('fs');
var dataloader = require('./dataloader');

if (process.argv.length < 5)
{
    console.log("Supply more arguments. Root, Revision, Output");
    console.log(process.argv);
}

var root = process.argv[2];
var revision = process.argv[3];
var output = process.argv[4];

var Data = {};
console.log("Loading tree at", root);
dataloader.load_tree(root, Data);
var result = JSON.stringify({
    revision: revision,
    data: Data
}, null, 2);

console.log("Writing ", result.length, "to", output);
fs.writeFileSync(output, result, "utf-8");
