import { describe, expect, it, vi } from 'vitest'

import { getAppEnvironment, normalizeApiBaseUrl } from './env'

describe('normalizeApiBaseUrl', () => {
  it('returns default when value missing', () => {
    expect(normalizeApiBaseUrl(undefined)).toBe('http://localhost:8080/api/v1')
    expect(normalizeApiBaseUrl('')).toBe('http://localhost:8080/api/v1')
  })

  it('strips trailing slashes and preserves protocol/host', () => {
    expect(normalizeApiBaseUrl('https://example.com/api/v1/')).toBe(
      'https://example.com/api/v1',
    )
  })

  it('falls back on invalid URLs', () => {
    const warn = vi.spyOn(console, 'warn').mockImplementation(() => {})
    expect(normalizeApiBaseUrl('example[broken]')).toBe('http://localhost:8080/api/v1')
    expect(warn).toHaveBeenCalled()
    warn.mockRestore()
  })
})

describe('getAppEnvironment', () => {
  it('prefers runtime config when provided', () => {
    expect(
      getAppEnvironment({
        runtimeConfig: { apiBaseUrl: 'https://runtime/api/' },
        buildEnv: { VITE_API_BASE_URL: 'https://build/api/' },
      }).apiBaseUrl,
    ).toBe('https://runtime/api')
  })

  it('falls back to build-time env when runtime config missing', () => {
    expect(
      getAppEnvironment({
        runtimeConfig: undefined,
        buildEnv: { VITE_API_BASE_URL: 'https://build/api/' },
      }).apiBaseUrl,
    ).toBe('https://build/api')
  })
})
