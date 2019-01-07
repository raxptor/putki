// Modules to control application life and create native browser window
const {app, Menu, ipcMain, BrowserWindow} = require('electron');

const template = [
  {
    label: 'File',
    submenu: [
      {
        label: "Save"
      },
      {
        label: "Import JSON",
        click: function() { console.log("Export JSON"); }
      },      
      {
        label: "Export JSON",
        click: function() { console.log("Export JSON"); }
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
           type: 'separator'
        },
        {
           role: 'resetzoom'
        },
        {
           role: 'zoomin'
        },
        {
           role: 'zoomout'
        },
        {
           type: 'separator'
        },
        {
           role: 'togglefullscreen'
        }
     ]
  },  
  {
     role: 'window',
     submenu: [
        {
           role: 'minimize'
        },
        {
           role: 'close'
        }
     ]
  }  
]

/*
const menu = Menu.buildFromTemplate(template)
Menu.setApplicationMenu(menu);
*/
// Keep a global reference of the window object, if you don't, the window will
// be closed automatically when the JavaScript object is garbage collected.
let mainWindow

function createWindow () {
  // Create the browser window.
  mainWindow = new BrowserWindow({width: 800, height: 600})

  // and load the index.html of the app.
  mainWindow.loadFile('index.html')

  // Open the DevTools.
  // mainWindow.webContents.openDevTools()

  // Emitted when the window is closed.
  mainWindow.on('closed', function () {
    // Dereference the window object, usually you would store windows
    // in an array if your app supports multi windows, this is the time
    // when you should delete the corresponding element.
    mainWindow = null
  })
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createWindow)

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
