import { defineConfig, devices } from '@playwright/test'

const PORT = Number(process.env.PLAYWRIGHT_PORT ?? 4173)
const HOST = process.env.PLAYWRIGHT_HOST ?? '127.0.0.1'
const protocol = process.env.PLAYWRIGHT_HTTPS === 'true' ? 'https' : 'http'
const baseURL = process.env.PLAYWRIGHT_BASE_URL ?? `${protocol}://${HOST}:${PORT}`

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  timeout: 30 * 1000,
  expect: {
    timeout: 5000,
  },
  retries: process.env.CI ? 1 : 0,
  reporter: process.env.CI ? [['html', { outputFolder: 'playwright-report' }], ['list']] : 'list',
  use: {
    baseURL,
    trace: 'on-first-retry',
    video: 'retain-on-failure',
    screenshot: 'only-on-failure',
    browserName: 'chromium',
    headless: process.env.CI ? true : undefined,
  },
  projects: [
    {
      name: 'Chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],
  webServer: {
    command: `npm run dev -- --host ${HOST} --port ${PORT}`,
    url: baseURL,
    reuseExistingServer: !process.env.CI,
    stdout: 'pipe',
    stderr: 'pipe',
  },
})
