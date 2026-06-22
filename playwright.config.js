// @ts-check
const { defineConfig } = require('@playwright/test');

module.exports = defineConfig({
  test_dir: 'e2e/tests',
  timeout: 30000,
  expect: {
    timeout: 10000,
  },
  fullyParallel: false,
  retries: 1,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000',
    headless: true,
    screenshot: 'only-on-failure',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'api',
      testMatch: /api\.spec\.ts/,
    },
    {
      name: 'ui',
      testMatch: /ui\.spec\.ts/,
    },
  ],
});
