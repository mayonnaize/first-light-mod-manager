import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['list'],
    ['monocart-reporter', {
      name: 'FLMM Frontend Coverage',
      outputFile: './test-results/report.html',
      coverage: {
        // カバレッジ計測対象ファイルをフロントエンドコードに限定
        entryFilter: (entry) => entry.url.includes('main.js') && !entry.url.includes('node_modules'),
        sourceFilter: (sourcePath) => sourcePath.includes('main.js') && !sourcePath.includes('node_modules'),
        lcov: true,
        reports: [
          ['v8'],
          ['console-summary'],
          ['html', {
            subdir: 'frontend-coverage'
          }]
        ]
      }
    }]
  ],
  use: {
    trace: 'on-first-retry',
  },
});
