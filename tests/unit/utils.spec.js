// フロントエンド純粋関数のブラウザ動作検証テスト
// ページ上で動作する実関数の実行結果検証

import { test, expect } from '@playwright/test';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import http from 'node:http';
import fs from 'node:fs/promises';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const srcRoot = path.resolve(__dirname, '../../src');

let server;
let baseUrl;

function contentType(filePath) {
  if (filePath.endsWith('.html')) return 'text/html; charset=utf-8';
  if (filePath.endsWith('.js')) return 'text/javascript; charset=utf-8';
  if (filePath.endsWith('.css')) return 'text/css; charset=utf-8';
  return 'application/octet-stream';
}

test.beforeAll(async () => {
  server = http.createServer(async (req, res) => {
    const reqUrl = new URL(req.url || '/', 'http://127.0.0.1');
    const requested = reqUrl.pathname === '/' ? 'index.html' : reqUrl.pathname.slice(1);
    const filePath = path.resolve(srcRoot, requested);
    if (!filePath.startsWith(srcRoot)) { res.writeHead(403); res.end(); return; }
    try {
      const data = await fs.readFile(filePath);
      res.writeHead(200, { 'content-type': contentType(filePath) });
      res.end(data);
    } catch {
      res.writeHead(404); res.end('Not found');
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const address = server.address();
  baseUrl = `http://127.0.0.1:${address.port}`;
});

test.afterAll(async () => {
  await new Promise(resolve => server.close(resolve));
});

// Tauri環境のグローバルモック定義
test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    window.__TAURI__ = {
      core: {
        invoke: async (command) => {
          if (command === 'load_settings') {
            return {
              game_path: '',
              language: 'en',
              nexus_api_key: '',
              nexus_mod_id: '0',
              auto_check_updates: false
            };
          }
          if (command === 'detect_game') {
            return { found: false, path: '', platform: 'unknown' };
          }
          if (command === 'get_mod_status') {
            return { installed: false, version: '', backup_exists: false };
          }
          if (command === 'list_mods') {
            return [];
          }
          return null;
        }
      },
      dialog: {
        open: async () => null
      },
      opener: {
        openUrl: async () => null
      }
    };
  });
});

// ─── formatBytes: main.js 実装関数のブラウザ動作検証 ────────────

test('formatBytes: 0 または負数は "0 B" を返す', async ({ page }) => {
  await page.goto(baseUrl);
  const results = await page.evaluate(() => {
    return [
      window.formatBytes(0),
      window.formatBytes(-1),
      window.formatBytes(NaN),
      window.formatBytes(-Infinity)
    ];
  });
  expect(results[0]).toBe('0 B');
  expect(results[1]).toBe('0 B');
  expect(results[2]).toBe('0 B');
  expect(results[3]).toBe('0 B');
});

test('formatBytes: バイト境界値の正確な単位変換', async ({ page }) => {
  await page.goto(baseUrl);
  const results = await page.evaluate(() => {
    return [
      window.formatBytes(1),
      window.formatBytes(999),
      window.formatBytes(1024),
      window.formatBytes(1024 * 1024),
      window.formatBytes(1024 * 1024 * 1024)
    ];
  });
  expect(results[0]).toBe('1 B');
  expect(results[1]).toBe('999 B');
  expect(results[2]).toBe('1.0 KB');
  expect(results[3]).toBe('1.0 MB');
  expect(results[4]).toBe('1.0 GB');
});

test('formatBytes: 小数点1桁の表示 (10未満のKB)', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.formatBytes(1536));
  expect(result).toBe('1.5 KB');
});

test('formatBytes: 10以上の単位は小数点なし', async ({ page }) => {
  await page.goto(baseUrl);
  const results = await page.evaluate(() => {
    return [
      window.formatBytes(10 * 1024),
      window.formatBytes(100 * 1024)
    ];
  });
  expect(results[0]).toBe('10 KB');
  expect(results[1]).toBe('100 KB');
});

test('formatBytes: 大きなファイルサイズ (MODファイル相当)', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.formatBytes(50 * 1024 * 1024));
  expect(result).toBe('50 MB');
});

// ─── escapeHtml: main.js 実装関数のブラウザ動作検証 ─────────────

test('escapeHtml: XSS対策 — script タグを無効化', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.escapeHtml('<script>alert(1)</script>'));
  expect(result).toBe('&lt;script&gt;alert(1)&lt;/script&gt;');
});

test('escapeHtml: & < > " \' の5文字すべてをエスケープ', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.escapeHtml('& < > " \''));
  expect(result).toBe('&amp; &lt; &gt; &quot; &#039;');
});

test('escapeHtml: 通常の文字列はそのまま返す', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.escapeHtml('Hello World 007'));
  expect(result).toBe('Hello World 007');
});

test('escapeHtml: null/undefined は空文字として扱う', async ({ page }) => {
  await page.goto(baseUrl);
  const results = await page.evaluate(() => {
    return [
      window.escapeHtml(null),
      window.escapeHtml(undefined)
    ];
  });
  expect(results[0]).toBe('');
  expect(results[1]).toBe('');
});

test('escapeHtml: 数値は文字列に変換してからエスケープ', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.escapeHtml(42));
  expect(result).toBe('42');
});

// ─── normalizeSettings: main.js 実装関数のブラウザ動作検証 ──────

test('normalizeSettings: 空オブジェクトはデフォルト値を補完する', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => {
    localStorage.clear();
    return window.normalizeSettings({});
  });
  expect(result.language).toBe('en');
  expect(result.nexus_mod_id).toBe('0');
  expect(result.auto_check_updates).toBe(true);
  expect(result.game_path).toBe('');
});

test('normalizeSettings: 不正な language は "en" にフォールバック', async ({ page }) => {
  await page.goto(baseUrl);
  const results = await page.evaluate(() => {
    return [
      window.normalizeSettings({ language: 'zh' }),
      window.normalizeSettings({ language: 'ja' })
    ];
  });
  expect(results[0].language).toBe('en');
  expect(results[1].language).toBe('en');
});

test('normalizeSettings: "pt" は有効な言語として受け入れる', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.normalizeSettings({ language: 'pt' }));
  expect(result.language).toBe('pt');
});

test('normalizeSettings: "en" は有効な言語として受け入れる', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.normalizeSettings({ language: 'en' }));
  expect(result.language).toBe('en');
});

test('normalizeSettings: nexus_mod_id が undefined のとき "0" を使う', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => window.normalizeSettings({ nexus_mod_id: undefined }));
  expect(result.nexus_mod_id).toBe('0');
});

test('normalizeSettings: auto_check_updates の boolean 型を保持', async ({ page }) => {
  await page.goto(baseUrl);
  const results = await page.evaluate(() => {
    return [
      window.normalizeSettings({ auto_check_updates: true }),
      window.normalizeSettings({ auto_check_updates: false })
    ];
  });
  expect(results[0].auto_check_updates).toBe(true);
  expect(results[1].auto_check_updates).toBe(false);
});

test('normalizeSettings: localStorageの "false" は auto_check_updates=false になる', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => {
    localStorage.setItem('autoCheckUpdates', 'false');
    const res = window.normalizeSettings({});
    localStorage.clear();
    return res;
  });
  expect(result.auto_check_updates).toBe(false);
});

test('normalizeSettings: localStorageの他の値は auto_check_updates=true になる', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => {
    localStorage.setItem('autoCheckUpdates', 'true');
    const res = window.normalizeSettings({});
    localStorage.clear();
    return res;
  });
  expect(result.auto_check_updates).toBe(true);
});

test('normalizeSettings: game_path は設定値を優先する', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => {
    localStorage.setItem('gamePath', 'D:\\OldPath\\Game');
    const res = window.normalizeSettings({ game_path: 'E:\\NewPath\\Game' });
    localStorage.clear();
    return res;
  });
  expect(result.game_path).toBe('E:\\NewPath\\Game');
});

test('normalizeSettings: game_path が未設定のとき localStorage から読む', async ({ page }) => {
  await page.goto(baseUrl);
  const result = await page.evaluate(() => {
    localStorage.setItem('gamePath', 'D:\\FromStorage\\Game');
    const res = window.normalizeSettings({});
    localStorage.clear();
    return res;
  });
  expect(result.game_path).toBe('D:\\FromStorage\\Game');
});
