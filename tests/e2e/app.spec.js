import { expect, test } from '@playwright/test';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(__dirname, '../..');
const srcRoot = path.join(appRoot, 'src');

let server;
let baseUrl;

function contentType(filePath) {
  if (filePath.endsWith('.html')) return 'text/html; charset=utf-8';
  if (filePath.endsWith('.js')) return 'text/javascript; charset=utf-8';
  if (filePath.endsWith('.css')) return 'text/css; charset=utf-8';
  if (filePath.endsWith('.svg')) return 'image/svg+xml';
  return 'application/octet-stream';
}

test.beforeAll(async () => {
  server = http.createServer(async (req, res) => {
    const requestUrl = new URL(req.url || '/', 'http://127.0.0.1');
    const requested = requestUrl.pathname === '/' ? 'index.html' : requestUrl.pathname.slice(1);
    const filePath = path.resolve(srcRoot, requested);
    if (!filePath.startsWith(srcRoot)) {
      res.writeHead(403);
      res.end('Forbidden');
      return;
    }

    try {
      const data = await fs.readFile(filePath);
      res.writeHead(200, { 'content-type': contentType(filePath) });
      res.end(data);
    } catch {
      res.writeHead(404);
      res.end('Not found');
    }
  });

  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const address = server.address();
  baseUrl = `http://127.0.0.1:${address.port}`;
});

test.afterAll(async () => {
  await new Promise(resolve => server.close(resolve));
});

async function installTauriMock(page, options = {}) {
  const initialSettings = {
    game_path: 'E:\\SteamLibrary\\steamapps\\common\\007 First Light',
    language: 'en',
    nexus_api_key: '',
    nexus_mod_id: '0',
    auto_check_updates: false,
    ...(options.initialSettings || {})
  };

  await page.addInitScript(({ initialSettings, selectedDirectory, selectedFile }) => {
    let settings = { ...initialSettings };
    let backupExists = true;

    const inspectPreview = {
      file_name: 'Switch HUD Buttons.zip',
      package_type: 'zip',
      installable: true,
      has_packagedefinition: true,
      has_metadata: true,
      warnings: [
        'Included packagedefinition.txt will be ignored and regenerated safely.',
        'chunk0patch1.rpkg will be installed as chunk0patch2.rpkg to avoid reserved or occupied patch slots.'
      ],
      rpkg_files: [
        {
          original_name: 'chunk0patch1.rpkg',
          target_name: 'chunk0patch2.rpkg',
          chunk: 0,
          requested_patch: 1,
          target_patch: 2,
          size: 1048576
        }
      ]
    };

    window.__testCalls = [];
    window.__TAURI__ = {
      core: {
        invoke: async (command, args = {}) => {
          window.__testCalls.push({ command, args });
          switch (command) {
            case 'load_settings':
              return { ...settings };
            case 'save_settings':
              settings = { ...args.settings };
              return { ...settings };
            case 'detect_game':
              return settings.game_path
                ? { found: true, path: settings.game_path, platform: 'manual' }
                : { found: false, path: '', platform: 'unknown' };
            case 'get_mod_status':
              return { installed: false, version: '', backup_exists: backupExists };
            case 'list_mods':
              return [
                {
                  id: 'chunk0patch2',
                  filename: 'chunk0patch2.rpkg',
                  original_filename: 'chunk0patch1.rpkg',
                  name: 'Switch HUD Buttons',
                  author: 'Test',
                  description: 'Button prompt atlas replacement',
                  version: '1.0.0',
                  active: true,
                  chunk: 0,
                  patch: 2
                }
              ];
            case 'inspect_mod':
              return inspectPreview;
            case 'delete_backup':
              backupExists = false;
              return null;
            case 'check_updates':
              return { version: '0.2.0', url: '', has_update: false };
            case 'open_game_folder':
            case 'install_mod':
            case 'uninstall_mod':
            case 'toggle_mod':
            case 'delete_mod':
              return null;
            default:
              throw new Error(`Unexpected command: ${command}`);
          }
        }
      },
      dialog: {
        open: async (args = {}) => {
          window.__testCalls.push({ command: 'dialog.open', args });
          return args.directory ? selectedDirectory : selectedFile;
        }
      },
      opener: {
        openUrl: async () => null
      }
    };
  }, {
    initialSettings,
    selectedDirectory: options.selectedDirectory || 'E:\\Games\\007 First Light',
    selectedFile: options.selectedFile || 'C:\\Mods\\Switch HUD Buttons.zip'
  });
}

test('loads and persists the configured game path through backend settings', async ({ page }) => {
  await installTauriMock(page, {
    selectedDirectory: 'E:\\Library\\007 First Light'
  });
  await page.goto(baseUrl);

  await expect(page.locator('#input-game-path')).toHaveValue('E:\\SteamLibrary\\steamapps\\common\\007 First Light');
  await expect(page.locator('#game-path-display')).toContainText('E:\\SteamLibrary\\steamapps\\common\\007 First Light');

  await page.locator('#nav-settings').click();
  await page.locator('#btn-choose-path').click();
  await expect(page.locator('#input-game-path')).toHaveValue('E:\\Library\\007 First Light');
  await page.locator('#btn-save-settings').click();
  await expect(page.locator('#game-path-display')).toContainText('E:\\Library\\007 First Light');

  const calls = await page.evaluate(() => window.__testCalls);
  const saveCalls = calls.filter(call => call.command === 'save_settings');
  expect(saveCalls.at(-1).args.settings.game_path).toBe('E:\\Library\\007 First Light');
});

test('shows mod contents and safe target patch name before install', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  await page.locator('#nav-install').click();
  await page.locator('#btn-browse-mod').click();

  await expect(page.locator('#selected-file-name')).toContainText('Switch HUD Buttons.zip');
  await expect(page.locator('#mod-preview')).toBeVisible();
  await expect(page.locator('#mod-preview-list')).toContainText('chunk0patch2.rpkg');
  await expect(page.locator('#mod-preview-list')).toContainText('chunk0patch1.rpkg');
  await expect(page.locator('#mod-preview-warnings')).toContainText('packagedefinition.txt');
  await expect(page.locator('#btn-do-install')).toBeEnabled();

  const calls = await page.evaluate(() => window.__testCalls);
  const inspectCall = calls.find(call => call.command === 'inspect_mod');
  expect(inspectCall.args.modPath).toBe('C:\\Mods\\Switch HUD Buttons.zip');
  expect(inspectCall.args.gamePath).toBe('E:\\SteamLibrary\\steamapps\\common\\007 First Light');
});
