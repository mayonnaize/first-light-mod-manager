import { expect, test as baseTest } from '@playwright/test';
import { addCoverageReport } from 'monocart-reporter';

// カバレッジ計測用カスタムテストオブジェクト定義
const test = baseTest.extend({
  page: async ({ page }, use) => {
    // JavaScriptカバレッジ計測開始
    await page.coverage.startJSCoverage({
      resetOnNavigation: false
    });
    await use(page);
    // JavaScriptカバレッジ計測終了
    const coverage = await page.coverage.stopJSCoverage();
    await addCoverageReport(coverage, test.info());
  }
});
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

    ...(options.initialSettings || {})
  };

  await page.addInitScript(({ initialSettings, selectedDirectory, selectedFile, checkUpdatesResponse, mockFailures, inspectModResponse, listModsResponse }) => {
    let settings = { ...initialSettings };
    let backupExists = mockFailures.backupExists !== false;

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
          if (mockFailures[command]) {
            throw new Error(`Mock error for command: ${command}`);
          }
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
              return listModsResponse || [
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
              return inspectModResponse || inspectPreview;
            case 'delete_backup':
              backupExists = false;
              return null;

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
          if (mockFailures.dialog_open) {
            throw new Error('Mock error for command: dialog.open');
          }
          return args.directory ? selectedDirectory : selectedFile;
        },
        confirm: async (message) => {
          window.__testCalls.push({ command: 'dialog.confirm', args: { message } });
          return window.confirm(message);
        }
      },
      opener: {
        openUrl: async (url) => {
          window.__testCalls.push({ command: 'opener.openUrl', args: { url } });
          if (mockFailures.opener_openUrl) {
            throw new Error('Mock error for opener.openUrl');
          }
          return null;
        }
      },
      event: {
        listen: async (eventName, callback) => {
          window.__testCalls.push({ command: 'event.listen', args: { eventName } });
          window.__eventListeners = window.__eventListeners || {};
          window.__eventListeners[eventName] = window.__eventListeners[eventName] || [];
          window.__eventListeners[eventName].push(callback);
          return () => {
            const listeners = window.__eventListeners[eventName];
            if (listeners) {
              const idx = listeners.indexOf(callback);
              if (idx !== -1) listeners.splice(idx, 1);
            }
          };
        }
      }
    };
  }, {
    initialSettings,
    selectedDirectory: options.selectedDirectory || 'E:\\Games\\007 First Light',
    selectedFile: options.selectedFile || 'C:\\Mods\\Switch HUD Buttons.zip',
    checkUpdatesResponse: options.checkUpdatesResponse || null,
    mockFailures: options.mockFailures || {},
    inspectModResponse: options.inspectModResponse || null,
    listModsResponse: options.listModsResponse || null
  });
}

async function waitForTestCall(page, command) {
  await page.waitForFunction((cmd) => {
    return window.__testCalls && window.__testCalls.some(c => c.command === cmd);
  }, command);
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

test('switches language to PT and reflects translated labels in UI', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // Settings タブに移動して言語を PT に切り替え
  await page.locator('#nav-settings').click();
  await page.locator('#select-language').selectOption('pt');
  // 言語変更イベントを待機
  await waitForTestCall(page, 'save_settings');

  // PT 翻訳が DOM に反映されていることを確認
  await expect(page.locator('[data-i18n="nav_home"]')).toContainText('Início');
  await expect(page.locator('[data-i18n="nav_install"]')).toContainText('Instalar Mod');

  // save_settings が呼ばれ、言語が PT で保存されたことを確認
  const calls = await page.evaluate(() => window.__testCalls);
  const saveCalls = calls.filter(c => c.command === 'save_settings');
  expect(saveCalls.length).toBeGreaterThan(0);
  expect(saveCalls.at(-1).args.settings.language).toBe('pt');
});

test('displays installed mod cards on home tab with correct metadata', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // Home タブがデフォルトで表示され、mod-list-section が可視
  await expect(page.locator('#mod-list-section')).toBeVisible();

  // モックデータのMOD名が表示されていること
  const grid = page.locator('#mod-list-grid');
  await expect(grid).toContainText('Switch HUD Buttons');

  // toggleスイッチとdeleteボタンが存在する
  await expect(grid.locator('.toggle-mod-active')).toHaveCount(1);
  await expect(grid.locator('.btn-delete-mod')).toHaveCount(1);

  // アクティブなMODのトグルはチェック済み
  const toggle = grid.locator('.toggle-mod-active');
  await expect(toggle).toBeChecked();
});

test('toggling a mod calls toggle_mod and refreshes the list', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // mod リストが表示されるまで待機
  await expect(page.locator('#mod-list-grid')).toContainText('Switch HUD Buttons');

  // チェックボックスは CSS で非表示のため JS で change イベントを直接発火
  // (ユーザーはラベルのビジュアルスライダーをクリックするが、同じ change イベントが発火する)
  await page.evaluate(() => {
    const checkbox = document.querySelector('.toggle-mod-active');
    checkbox.checked = false;
    checkbox.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await waitForTestCall(page, 'toggle_mod');

  const calls = await page.evaluate(() => window.__testCalls);
  const toggleCall = calls.find(c => c.command === 'toggle_mod');
  expect(toggleCall).toBeDefined();
  expect(toggleCall.args.modId).toBe('chunk0patch2');
  expect(toggleCall.args.active).toBe(false);
});

test('deleting a backup calls delete_backup and updates backup status', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // Settings タブを開く
  await page.locator('#nav-settings').click();

  // バックアップステータスが表示されていること
  await expect(page.locator('#backup-status-text')).toContainText('Backup created');

  // バックアップを削除 (confirm をオートOKに設定)
  page.on('dialog', dialog => dialog.accept());
  await page.locator('#btn-delete-backup').click();
  await waitForTestCall(page, 'delete_backup');

  const calls = await page.evaluate(() => window.__testCalls);
  const deleteCall = calls.find(c => c.command === 'delete_backup');
  expect(deleteCall).toBeDefined();
  expect(deleteCall.args.gamePath).toBe('E:\\SteamLibrary\\steamapps\\common\\007 First Light');
});

test('clear file button hides preview and resets selected file state', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // Install タブを開き、ファイルを選択
  await page.locator('#nav-install').click();
  await page.locator('#btn-browse-mod').click();

  // ファイル選択済み状態を確認
  await expect(page.locator('#selected-file-display')).toBeVisible();
  await expect(page.locator('#drop-zone')).toBeHidden();

  // ✕ ボタンでクリア
  await page.locator('#btn-clear-file').click();

  // ドロップゾーンが再表示され、ファイル表示が消える
  await expect(page.locator('#drop-zone')).toBeVisible();
  await expect(page.locator('#selected-file-display')).toBeHidden();
  await expect(page.locator('#mod-preview')).toBeHidden();

  // インストールボタンは無効
  await expect(page.locator('#btn-do-install')).toBeDisabled();
});

test('install button is disabled without both game path and mod file', async ({ page }) => {
  // ゲームパスなしの設定
  await installTauriMock(page, { initialSettings: { game_path: '', language: 'en' } });
  await page.goto(baseUrl);

  await page.locator('#nav-install').click();

  // ゲームパスがないのでインストールボタンは無効
  await expect(page.locator('#btn-do-install')).toBeDisabled();
});

test('installMod: installs mod and transitions back to home tab', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // インストール画面へ遷移
  await page.locator('#nav-install').click();
  await page.locator('#btn-browse-mod').click();

  // インストール実行
  await page.locator('#btn-do-install').click();
  await waitForTestCall(page, 'install_mod');

  // ホームタブ遷移検証
  await expect(page.locator('#tab-home')).toHaveClass(/active/);

  const calls = await page.evaluate(() => window.__testCalls);
  const installCall = calls.find(c => c.command === 'install_mod');
  expect(installCall).toBeDefined();
});

test('uninstallMod: uninstalls mods and restores backup', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // 警告ダイアログ自動承認設定
  page.on('dialog', dialog => dialog.accept());
  // アンインストール実行
  await page.locator('#btn-uninstall-main').click();
  await waitForTestCall(page, 'uninstall_mod');

  const calls = await page.evaluate(() => window.__testCalls);
  const uninstallCall = calls.find(c => c.command === 'uninstall_mod');
  expect(uninstallCall).toBeDefined();
});



test('deleteMod: deleting mod triggers delete_mod IPC call', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // 警告ダイアログ自動承認設定
  page.on('dialog', dialog => dialog.accept());
  // モッド削除実行
  await page.locator('.btn-delete-mod').click();
  await waitForTestCall(page, 'delete_mod');

  const calls = await page.evaluate(() => window.__testCalls);
  const deleteCall = calls.find(c => c.command === 'delete_mod');
  expect(deleteCall).toBeDefined();
  expect(deleteCall.args.modId).toBe('chunk0patch2');
});

test('openGameFolder: clicking folder button invokes open_game_folder', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);

  // ゲームフォルダ展開実行
  await page.locator('#btn-open-folder').click();
  await waitForTestCall(page, 'open_game_folder');

  const calls = await page.evaluate(() => window.__testCalls);
  const openCall = calls.find(c => c.command === 'open_game_folder');
  expect(openCall).toBeDefined();
});

test('drag and drop a valid mod file triggers inspection', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();

  // ドロップゾーンへのドラッグアンドドロップ操作模擬
  await page.evaluate(() => {
    const zone = document.getElementById('drop-zone');
    const dt = new DataTransfer();
    const file = new File(['rpkg'], 'test_mod.rpkg', { type: 'application/octet-stream' });
    Object.defineProperty(file, 'path', { value: 'C:\\Mods\\test_mod.rpkg' });
    dt.items.add(file);
    const dragEvent = new DragEvent('drop', { dataTransfer: dt, bubbles: true });
    zone.dispatchEvent(dragEvent);
  });

  await expect(page.locator('#selected-file-display')).toBeVisible();
});

test('drag and drop an invalid file displays error toast', async ({ page }) => {
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();

  // 不正な拡張子ファイルのドロップ操作模擬
  await page.evaluate(() => {
    const zone = document.getElementById('drop-zone');
    const dt = new DataTransfer();
    const file = new File(['txt'], 'test_mod.txt', { type: 'text/plain' });
    dt.items.add(file);
    const dragEvent = new DragEvent('drop', { dataTransfer: dt, bubbles: true });
    zone.dispatchEvent(dragEvent);
  });

  await expect(page.locator('#toast-container')).toContainText('Please select a .rpkg or .zip file');
});

test('inspectSelectedMod: handles error when backend command fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { inspect_mod: true } });
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();
  await page.locator('#btn-browse-mod').click();

  await expect(page.locator('#mod-preview-warnings')).toContainText('Mock error for command: inspect_mod');
});

test('installMod: handles error when backend install fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { install_mod: true } });
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();
  await page.locator('#btn-browse-mod').click();
  await page.locator('#btn-do-install').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: install_mod');
});

test('uninstallMod: handles error when backend uninstall fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { uninstall_mod: true } });
  page.on('dialog', dialog => dialog.accept());
  await page.goto(baseUrl);
  await page.locator('#btn-uninstall-main').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: uninstall_mod');
});

test('toggleMod: handles error when backend toggle fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { toggle_mod: true } });
  await page.goto(baseUrl);
  await page.evaluate(() => {
    const checkbox = document.querySelector('.toggle-mod-active');
    checkbox.checked = false;
    checkbox.dispatchEvent(new Event('change', { bubbles: true }));
  });

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: toggle_mod');
});

test('deleteMod: handles error when backend delete fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { delete_mod: true } });
  page.on('dialog', dialog => dialog.accept());
  await page.goto(baseUrl);
  await page.locator('.btn-delete-mod').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: delete_mod');
});

test('saveSettings: handles error when backend save fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { save_settings: true } });
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#btn-save-settings').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: save_settings');
});

test('selectGamePath: handles error when browse dialog fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { dialog_open: true } });
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#btn-choose-path').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: dialog.open');
});

test('openGameFolder: handles error when open fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { open_game_folder: true } });
  await page.goto(baseUrl);
  await page.locator('#btn-open-folder').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: open_game_folder');
});

test('select language: handles persist error', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { save_settings: true } });
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#select-language').selectOption('pt');

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: save_settings');
});

test('delete backup: toast when no backup exists', async ({ page }) => {
  // バックアップ未存在状態の模擬
  await installTauriMock(page, { mockFailures: { backupExists: false } });
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#btn-delete-backup').click();

  await expect(page.locator('#toast-container')).toContainText('No backup to remove.');
});


test('delete backup: toast when delete fails', async ({ page }) => {
  await installTauriMock(page, { mockFailures: { delete_backup: true } });
  page.on('dialog', dialog => dialog.accept());
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#btn-delete-backup').click();

  await expect(page.locator('#toast-container')).toContainText('Mock error for command: delete_backup');
});

test('init: handles load_settings error', async ({ page }) => {
  // load_settingsの例外ハンドリング検証
  await installTauriMock(page, {
    initialSettings: { game_path: '' },
    mockFailures: { load_settings: true }
  });
  await page.goto(baseUrl);
  await expect(page.locator('#input-game-path')).toHaveValue('');
});

test('init: handles save_settings migration error', async ({ page }) => {
  // 初期化時のsave_settings例外ハンドリング検証
  await installTauriMock(page, {
    initialSettings: { game_path: 'E:\\Games\\007 First Light' },
    mockFailures: { save_settings: true }
  });
  await page.goto(baseUrl);
  await expect(page.locator('#input-game-path')).toHaveValue('E:\\Games\\007 First Light');
});

test('toast: removes toast element after fadeout animation', async ({ page }) => {
  // トーストのアニメーション終了と要素削除の検証
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#btn-save-settings').click();
  
  const toastContainer = page.locator('#toast-container');
  await expect(toastContainer).toContainText('Settings saved!');
  
  // トースト要素取得
  const toastEl = toastContainer.locator('.toast');
  await expect(toastEl).toBeVisible();

  // アニメーション終了イベント模擬
  await page.evaluate(() => {
    const el = document.querySelector('.toast');
    if (el) {
      el.dispatchEvent(new Event('animationend'));
    }
  });
  
  // 即座の消去確認
  await expect(toastEl).toHaveCount(0);
});



test('renderModPreview: displays placeholder on empty mod rpkg list', async ({ page }) => {
  // RPKGファイルがない場合のプレビュー表示検証
  await installTauriMock(page, {
    inspectModResponse: {
      file_name: 'Switch HUD Buttons.zip',
      package_type: 'zip',
      installable: false,
      has_packagedefinition: true,
      has_metadata: false,
      warnings: [],
      rpkg_files: []
    }
  });
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();
  await page.locator('#btn-browse-mod').click();
  await expect(page.locator('#mod-preview-list')).toContainText('No valid .rpkg files found in this package.');
});

test('renderModList: displays empty placeholder when no mods are installed', async ({ page }) => {
  // MODが空のときの表示検証
  await installTauriMock(page, {
    listModsResponse: []
  });
  await page.goto(baseUrl);
  await expect(page.locator('#mod-list-grid')).toContainText('No mods installed yet.');
});

test('renderModList: displays mod cards with empty optional metadata fields', async ({ page }) => {
  // 任意メタデータが空のときのMODカード描画検証
  await installTauriMock(page, {
    listModsResponse: [
      {
        id: 'chunk0patch2',
        filename: 'chunk0patch2.rpkg',
        original_filename: 'chunk0patch2.rpkg',
        name: '',
        author: '',
        description: '',
        version: '',
        active: true,
        chunk: 0,
        patch: 2
      }
    ]
  });
  await page.goto(baseUrl);
  await expect(page.locator('#mod-list-grid')).toContainText('chunk0patch2');
});

test('deleteMod: early exit on confirm rejection', async ({ page }) => {
  // 削除キャンセルの検証
  await installTauriMock(page);
  await page.goto(baseUrl);

  // 確認ダイアログのキャンセル模擬
  page.on('dialog', dialog => dialog.dismiss());
  await page.locator('.btn-delete-mod').click();
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 50)));

  // 削除処理 of 未実行確認
  const calls = await page.evaluate(() => window.__testCalls);
  const deleteCall = calls.find(c => c.command === 'delete_mod');
  expect(deleteCall).toBeUndefined();
});

test('game info: PT labels when game is not found', async ({ page }) => {
  // ポルトガル語設定かつゲーム未検出時のUI表示検証
  await installTauriMock(page, {
    initialSettings: { game_path: '', language: 'pt' }
  });
  await page.goto(baseUrl);
  await expect(page.locator('#game-path-display')).toContainText('Jogo não encontrado. Configure o caminho manualmente.');
  await expect(page.locator('#sc-game')).toContainText('❌ Não encontrado');
});

test('applyLanguage: fallback to en', async ({ page }) => {
  // 未サポート言語適用時の英語フォールバック検証
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.evaluate(() => {
    const select = document.getElementById('select-language');
    const option = document.createElement('option');
    option.value = 'fr';
    option.text = 'French';
    select.appendChild(option);
    select.value = 'fr';
    select.dispatchEvent(new Event('change'));
  });
  await expect(page.locator('[data-i18n="nav_home"]')).toContainText('Home');
});

test('install tab: manual game path selection click', async ({ page }) => {
  // 手動ゲームパス選択ボタンクリック時の挙動検証
  await installTauriMock(page, {
    initialSettings: { game_path: '' }
  });
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();
  await page.locator('#btn-manual-path').click();
  await expect(page.locator('#toast-container')).toContainText('Game directory configured!');
});

test('saveSettings: saves with empty game path', async ({ page }) => {
  // 空のゲームパス保存時の挙動検証
  await installTauriMock(page, {
    initialSettings: { game_path: '' }
  });
  await page.goto(baseUrl);
  await page.locator('#nav-settings').click();
  await page.locator('#btn-save-settings').click();
  await expect(page.locator('#toast-container')).toContainText('Settings saved!');
});



test('dragzone: dragover and dragleave styles', async ({ page }) => {
  // ドラッグオーバーとドラッグリーブのスタイルの検証
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();
  
  await page.evaluate(() => {
    const zone = document.getElementById('drop-zone');
    zone.dispatchEvent(new DragEvent('dragover', { bubbles: true }));
  });
  await expect(page.locator('#drop-zone')).toHaveClass(/drag-over/);

  await page.evaluate(() => {
    const zone = document.getElementById('drop-zone');
    zone.dispatchEvent(new DragEvent('dragleave', { bubbles: true }));
  });
  await expect(page.locator('#drop-zone')).not.toHaveClass(/drag-over/);
});

test('tauri native drag-drop: drag-enter and drag-leave styles', async ({ page }) => {
  // ネイティブdrag-enterシミュレーション
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.locator('#nav-install').click();

  await page.evaluate(() => {
    const listeners = window.__eventListeners['tauri://drag-enter'];
    if (listeners) {
      for (const cb of listeners) {
        cb();
      }
    }
  });
  await expect(page.locator('#drop-zone')).toHaveClass(/drag-over/);

  // ネイティブdrag-leaveシミュレーション
  await page.evaluate(() => {
    const listeners = window.__eventListeners['tauri://drag-leave'];
    if (listeners) {
      for (const cb of listeners) {
        cb();
      }
    }
  });
  await expect(page.locator('#drop-zone')).not.toHaveClass(/drag-over/);
});

test('tauri native drag-drop: drops a valid mod file and switches to install tab', async ({ page }) => {
  // 初期表示時のHomeタブ表示確認
  await installTauriMock(page);
  await page.goto(baseUrl);
  await expect(page.locator('#tab-home')).toHaveClass(/active/);

  // ネイティブdrag-dropシミュレーション
  await page.evaluate(() => {
    const listeners = window.__eventListeners['tauri://drag-drop'];
    if (listeners) {
      for (const cb of listeners) {
        cb({
          payload: {
            paths: ['C:\\Mods\\test_mod.rpkg']
          }
        });
      }
    }
  });

  // インストールタブへの自動遷移とファイル名表示確認
  await expect(page.locator('#tab-install')).toHaveClass(/active/);
  await expect(page.locator('#selected-file-display')).toBeVisible();
  await expect(page.locator('#selected-file-name')).toContainText('test_mod.rpkg');
});

test('tauri native drag-drop: drops an invalid file and shows toast', async ({ page }) => {
  // 不正拡張子ドロップシミュレーション
  await installTauriMock(page);
  await page.goto(baseUrl);

  await page.evaluate(() => {
    const listeners = window.__eventListeners['tauri://drag-drop'];
    if (listeners) {
      for (const cb of listeners) {
        cb({
          payload: {
            paths: ['C:\\Mods\\test_mod.txt']
          }
        });
      }
    }
  });

  // エラートースト表示確認
  await expect(page.locator('#toast-container')).toContainText('Please select a .rpkg or .zip file');
});

test('detectGame: handles error when detect_game fails', async ({ page }) => {
  // detect_game失敗時のエラーハンドリング検証
  await installTauriMock(page, { mockFailures: { detect_game: true } });
  await page.goto(baseUrl);
  await expect(page.locator('#toast-container')).toContainText('Error detecting game:');
});

test('getModStatus: handles error when get_mod_status fails', async ({ page }) => {
  // get_mod_status失敗時のエラーハンドリング検証
  await installTauriMock(page, { mockFailures: { get_mod_status: true } });
  await page.goto(baseUrl);
  await expect(page.locator('#nav-home')).toBeVisible();
});

test('renderModList: handles error when list_mods fails', async ({ page }) => {
  // list_mods失敗時のエラーハンドリング検証
  await installTauriMock(page, { mockFailures: { list_mods: true } });
  await page.goto(baseUrl);
  await expect(page.locator('#mod-list-section')).toBeHidden();
});

test('navigation: clicks about tab', async ({ page }) => {
  // Aboutタブへの遷移検証
  await installTauriMock(page);
  await page.goto(baseUrl);
  await page.locator('#nav-about').click();
  await expect(page.locator('#tab-about')).toHaveClass(/active/);
});

test('navigation: clicks footer and about links', async ({ page }) => {
  // 各種外部リンククリックの挙動検証
  await installTauriMock(page);
  await page.goto(baseUrl);
  
  await page.locator('#link-discord').click();
  await page.locator('#link-nexus').click();
  
  await page.locator('#nav-about').click();
  await page.locator('#abt-discord').click();
  await page.locator('#abt-nexus').click();
});




