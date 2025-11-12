const DEFAULT_API_BASE_URL = 'http://localhost:8080/api/v1'

const stripTrailingSlash = (value: string) => value.replace(/\/+$/, '')

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
      `[env] Invalid VITE_API_BASE_URL "${trimmed}", falling back to ${DEFAULT_API_BASE_URL}`,
    )
    return DEFAULT_API_BASE_URL
  }
}

export function getAppEnvironment() {
  return {
    apiBaseUrl: normalizeApiBaseUrl(import.meta.env.VITE_API_BASE_URL),
  }
}

export type AppEnvironment = ReturnType<typeof getAppEnvironment>
