import js from '@eslint/js'
import globals from 'globals'
import reactHooks from 'eslint-plugin-react-hooks'
import reactRefresh from 'eslint-plugin-react-refresh'
import tseslint from 'typescript-eslint'
import { defineConfig, globalIgnores } from 'eslint/config'

const reactHooksConfig = reactHooks.configs['recommended-latest']
const reactRefreshConfig = reactRefresh.configs.vite

export default defineConfig([
  globalIgnores(['dist', 'coverage', 'node_modules', '.vite', 'playwright-report', 'test-results']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: {
      ecmaVersion: 2022,
      globals: globals.browser,
    },
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooksConfig.rules,
      ...reactRefreshConfig.rules,
    },
    settings: {
      ...reactHooksConfig.settings,
      ...reactRefreshConfig.settings,
    },
  },
])
