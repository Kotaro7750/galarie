type RuntimeConfig = {
  apiBaseUrl?: string
}

type BuildEnv = {
  VITE_API_BASE_URL?: string
}

const DEFAULT_API_BASE_URL = 'http://localhost:8080/api/v1'

const stripTrailingSlash = (value: string) => value.replace(/\/+$/, '')

function readRuntimeConfig(): RuntimeConfig | undefined {
  if (typeof window === 'undefined') {
    return undefined
  }
  return window.__GALARIE_RUNTIME_CONFIG__ ?? undefined
}

export function normalizeApiBaseUrl(value?: string | null): string {
  const trimmed = value?.trim()
  if (!trimmed) {
    return DEFAULT_API_BASE_URL
  }

  try {
    const normalized = new URL(trimmed)
    return stripTrailingSlash(normalized.toString())
  } catch {
    console.warn(
      `[env] Invalid API base "${trimmed}", falling back to ${DEFAULT_API_BASE_URL}`,
    )
    return DEFAULT_API_BASE_URL
  }
}

type GetAppEnvironmentOptions = {
  runtimeConfig?: RuntimeConfig
  buildEnv?: BuildEnv
}

export function getAppEnvironment(options?: GetAppEnvironmentOptions) {
  const runtimeConfig = options?.runtimeConfig ?? readRuntimeConfig()
  const buildEnv = options?.buildEnv ?? import.meta.env
  return {
    apiBaseUrl: normalizeApiBaseUrl(runtimeConfig?.apiBaseUrl ?? buildEnv.VITE_API_BASE_URL),
  }
}

export type AppEnvironment = ReturnType<typeof getAppEnvironment>

export type { GetAppEnvironmentOptions, RuntimeConfig }
