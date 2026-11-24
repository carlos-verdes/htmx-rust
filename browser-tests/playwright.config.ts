import { defineConfig, devices } from '@playwright/test';

const port = 3210;

export default defineConfig({
  testDir: 'tests',
  forbidOnly: !!process.env.CI,
  reporter: 'list',
  use: { baseURL: `http://127.0.0.1:${port}` },
  // Every scenario runs twice: without JavaScript a swap is a full load, and
  // what the reader sees is the same.
  projects: [
    { name: 'javascript', use: devices['Desktop Chrome'] },
    {
      name: 'no-javascript',
      use: { ...devices['Desktop Chrome'], javaScriptEnabled: false },
    },
  ],
  webServer: {
    command: 'cargo run --package htmx-rust-demo',
    cwd: '..',
    env: { PORT: String(port) },
    url: `http://127.0.0.1:${port}`,
    timeout: 300_000,
    reuseExistingServer: !process.env.CI,
  },
});
