// Modules to control application life and create native browser window
const {app, Menu, ipcMain, BrowserWindow, dialog} = require('electron');
const fs = require('fs');
const path = require('path');

var unsavedChanges = false;

ipcMain.on('change', function() {
  console.log("Changed");
  unsavedChanges = true;
});
ipcMain.on("saved", function() {
  console.log("Saved");
  unsavedChanges = false;
});

const template = [
  {
    label: 'File',
    submenu: [
      {
        label: "New object",
        click: function() {
          mainWindow.webContents.send('new-object');          
        }
      },
      {
        label: "Save",
        accelerator: 'CmdOrCtrl+S',        
        click: function() {
          mainWindow.webContents.send('save');
        }
      },
      {
        role: 'quit'
      },
    ]
  },   
  {
     label: 'Edit',
     submenu: [
        {
           role: 'undo'
        },
        {
           role: 'redo'
        },
        {
           type: 'separator'
        },
        {
           role: 'cut'
        },
        {
           role: 'copy'
        },
        {
           role: 'paste'
        }
     ]
  },  
  {
     label: 'View',
     submenu: [
        {
           role: 'reload'
        },
        {
           role: 'toggledevtools'
        },
        {
          label: 'Show unset values',
          type: 'checkbox',
          checked: true,
          click: function(mi) {
            mainWindow.webContents.send('show-unset', mi.checked);
          }
        }
     ]
  }
]

const menu = Menu.buildFromTemplate(template)
Menu.setApplicationMenu(menu);

// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow

function createWindow () {
  // Create the browser window.
  mainWindow = new BrowserWindow({width: 800, height: 600})
  mainWindow.configPath = configPath;

  // and load the index.html of the app.
  mainWindow.loadFile('index.html')

  // Open the DevTools.
  // mainWindow.webContents.openDevTools()


  mainWindow.on('close', function(e) {
    if (unsavedChanges)
    {
      e.preventDefault();
      dialog.showMessageBox(mainWindow, { 
        type: 'question',
        title: 'Unsaved changes',
        buttons: ['Save', 'Discard changes'],
        message: 'Save before quitting?',
        detail: 'Click save to save before exiting.',
      }, function(response) {
        unsavedChanges = false;
        if (response == 0) {
          ipcMain.on("saved", function() {
            app.quit();
          });
          mainWindow.webContents.send("save");
        } else {
          app.quit();
        }
      });
    }
  });
  // Emitted when the window is closed.
  mainWindow.on('closed', function () {
    // Dereference the window object, usually you would store windows
    // in an array if your app supports multi windows, this is the time
    // when you should delete the corresponding element.
    mainWindow = null
  })
}

var configPath = process.env.PUTKED;

function try_load(path, on_done)
{
  if (path != null && path != undefined) {
    var data = fs.readFileSync(configPath, "utf-8");
    if (data != null) {
      on_done(data);
      return;
    }
  }
  dialog.showOpenDialog(mainWindow, { filters: [{name: 'Configuration file', extensions: ['putked']}] }, function(x) {    
    if (x == undefined || x.length == 0) {
      dialog.showMessageBox(mainWindow, { 
        type: 'error',
        title: 'Error',
        message: 'You need to pick a configuration file for the project you want to edit.'
      }, function() {
        app.quit();
      });
      return;
    }
    configPath = x[0];
    try_load(configPath, on_done);
  });
}

ipcMain.on('request-configuration', function(evt, ed) {
  try_load(configPath, function(data) {
    evt.sender.send('configuration', {
      config_path: configPath,
      root: path.dirname(configPath),
      data: JSON.parse(data)
    });
  });
});
  
app.on('ready', function() {
  createWindow();
});

// Quit when all windows are closed.
app.on('window-all-closed', function () {
  // On macOS it is common for applications and their menu bar
  // to stay active until the user quits explicitly with Cmd + Q
  if (process.platform !== 'darwin') {
    app.quit()
  }
})

app.on('activate', function () {
  // On macOS it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (mainWindow === null) {
    createWindow()
  }
})

ipcMain.on('choose-menu', (event, args) => {
  var submenu = [];
  for (var i=0;i<args.length;i++)
  {
    (function(i) {
      submenu.push({
        label: args[i].Title,
        click: function() {
          event.sender.send("choose-menu", args[i].Data);
        }
      });
    })(i);
  }
  var menu = Menu.buildFromTemplate(submenu);
  menu.popup({});    
});

ipcMain.on('edit-pointer', (event, arg) => {

  console.log(arg);
  var submenu = [];
  for (var x in arg.types) {
      (function(x) {
        submenu.push({
            label: arg.types[x].display,
            click: function() {
              event.sender.send('edit-pointer-reply', arg.types[x]);
            }
        });
      })(x);
  }
  if (submenu.length > 10) {
    submenu = [{
      label: 'New',
      click: function() {
        event.sender.send('edit-pointer-reply', '@pick-type');
      }
    }];
  }
  var menu = Menu.buildFromTemplate([{
    label: 'Create',
    submenu: submenu
  }, {
    label: 'Link',
    click: function() {
      event.sender.send('edit-pointer-reply', '@link');
    }
  }, {
    label: 'Clear',
    click: function() {
      event.sender.send('edit-pointer-reply', '@clear');
    }
  }]);
  menu.popup({ });  
});
