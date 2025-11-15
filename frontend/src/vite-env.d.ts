/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_BASE_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

interface GalarieRuntimeConfig {
  apiBaseUrl?: string
}

declare global {
  interface Window {
    __GALARIE_RUNTIME_CONFIG__?: GalarieRuntimeConfig
  }
}

export {}
