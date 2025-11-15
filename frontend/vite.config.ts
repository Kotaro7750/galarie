import react from '@vitejs/plugin-react'
import { configDefaults, defineConfig } from 'vitest/config'

const ensureTrailingSlash = (value: string) => (value.endsWith('/') ? value : `${value}/`)
const base = ensureTrailingSlash(process.env.VITE_BASE_PATH ?? '/')

export default defineConfig({
  base,
  plugins: [react()],
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    exclude: [...configDefaults.exclude, 'e2e/**', 'tests/**'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html'],
      reportsDirectory: 'coverage/unit',
      include: ['src/**/*.{ts,tsx}'],
      exclude: ['src/main.tsx', 'src/App.tsx', 'src/vite-env.d.ts'],
    },
  },
})
