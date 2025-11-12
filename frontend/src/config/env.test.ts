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
  it('reads from import.meta.env', () => {
    const original = import.meta.env.VITE_API_BASE_URL
    // @ts-expect-error - we intentionally mutate for the test
    import.meta.env.VITE_API_BASE_URL = 'https://demo/api/'
    expect(getAppEnvironment().apiBaseUrl).toBe('https://demo/api')
    // @ts-expect-error - restore value for subsequent tests
    import.meta.env.VITE_API_BASE_URL = original
  })
})
