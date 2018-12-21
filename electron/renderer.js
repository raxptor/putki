var Primitive = {
    "String": {
    },
    "Hash": {
    },    
    "Int": {
    }
}

var UserTypes = {
    "Character": {
        Fields: [
            { Name: "Name", Type:"String", array:false },
            { Name: "Description", Type:"String", array:false },
        ]
    },
    "Roster": {
        Fields: [
            { Name: "Name", Type:"String", array:false },
            { Name: "Characters", Type:"Character", ptr:true, array:true }
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