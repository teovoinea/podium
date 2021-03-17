/* eslint no-unused-expressions: 0 */
import { app, BrowserWindow, ipcMain, globalShortcut } from 'electron';
import * as path from 'path';
import * as url from 'url';

let win: BrowserWindow | null;

const installExtensions = async () => {
    const installer = require('electron-devtools-installer');
    const forceDownload = !!process.env.UPGRADE_EXTENSIONS;
    const extensions = ['REACT_DEVELOPER_TOOLS', 'REDUX_DEVTOOLS'];

    return Promise.all(
        extensions.map(name => installer.default(installer[name], forceDownload))
    ).catch(console.log); // eslint-disable-line no-console
};

const createWindow = async () => {
    if (process.env.NODE_ENV !== 'production') {
        await installExtensions();
    }

    win = new BrowserWindow({
        width: 680,
        height: 57,
        frame: false,
        /* transparent: true, */ 
        // resizable: false
    });

    if (process.env.NODE_ENV !== 'production') {
        process.env.ELECTRON_DISABLE_SECURITY_WARNINGS = '1'; // eslint-disable-line require-atomic-updates
        win.loadURL(`http://localhost:2003`);
    } else {
        win.loadURL(
            url.format({
                pathname: path.join(__dirname, 'index.html'),
                protocol: 'file:',
                slashes: true
            })
        );
    }

    if (process.env.NODE_ENV !== 'production') {
        // Open DevTools, see https://github.com/electron/electron/issues/12438 for why we wait for dom-ready
        win.webContents.once('dom-ready', () => {
            win!.webContents.openDevTools({
                mode: 'detach'
            });
        });
    }

    win.on('closed', () => {
        win = null;
    });
};

app.on('ready', () => {
    createWindow();
    globalShortcut.register('CommandOrControl+Shift+Space', () => {
        if (win !== null) {
            win.show();
        }
    });
});

app.on('window-all-closed', () => {
    if (process.platform !== 'darwin') {
        app.quit();
    }
});

app.on('activate', () => {
    if (win === null) {
        createWindow();
    }
});

// do not quit when all windows are closed
// and continue running on background to listen
// for shortcuts
// app.on('window-all-closed', (e) => {
//     e.preventDefault()
//     e.returnValue = false
//   })

ipcMain.on('displayResults', () => {
    win?.setSize(680, 550, true);
});

ipcMain.on('shrinkWindow', () => {
    win?.setSize(680, 50, true);
});
