import React from 'react'
import ReactDOM from 'react-dom/client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

import App from './App'
import type { RuntimeConfig } from './config/env'
import './index.css'

const queryClient = new QueryClient()

async function loadRuntimeConfig() {
  if (typeof window === 'undefined') {
    return
  }
  const runtimeConfigUrl = `${import.meta.env.BASE_URL ?? '/'}runtime-env.json`
  try {
    const response = await fetch(runtimeConfigUrl, { cache: 'no-store' })
    if (!response.ok) {
      return
    }
    const payload = (await response.json()) as RuntimeConfig
    window.__GALARIE_RUNTIME_CONFIG__ = payload
  } catch (error) {
    console.warn('[env] Failed to load runtime config', error)
  }
}

async function bootstrap() {
  await loadRuntimeConfig()

  ReactDOM.createRoot(document.getElementById('root')!).render(
    <React.StrictMode>
      <QueryClientProvider client={queryClient}>
        <App />
      </QueryClientProvider>
    </React.StrictMode>,
  )
}

void bootstrap()
