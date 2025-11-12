import { useCallback, useEffect, useState } from 'react'

export type PersistedFilters = {
  tags: string
  attributes: Record<string, string[]>
}

const STORAGE_KEY = 'galarie:searchFilters'

export function usePersistedFilters(defaultValue: PersistedFilters) {
  const [filters, setFilters] = useState<PersistedFilters>(() => {
    if (typeof window === 'undefined') {
      return defaultValue
    }
    try {
      const raw = window.sessionStorage.getItem(STORAGE_KEY)
      if (!raw) return defaultValue
      const parsed = JSON.parse(raw) as PersistedFilters
      if (!parsed.tags || typeof parsed.tags !== 'string') {
        return defaultValue
      }
      if (!parsed.attributes || typeof parsed.attributes !== 'object') {
        return defaultValue
      }
      return {
        tags: parsed.tags,
        attributes: parsed.attributes,
      }
    } catch {
      return defaultValue
    }
  })

  useEffect(() => {
    if (typeof window === 'undefined') return
    window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(filters))
  }, [filters])

  const updateTags = useCallback((nextTags: string) => {
    setFilters((prev) => ({ ...prev, tags: nextTags }))
  }, [])

  const updateAttributes = useCallback((nextAttributes: Record<string, string[]>) => {
    setFilters((prev) => ({ ...prev, attributes: nextAttributes }))
  }, [])

  const clearFilters = useCallback(() => {
    setFilters(defaultValue)
    if (typeof window !== 'undefined') {
      window.sessionStorage.removeItem(STORAGE_KEY)
    }
  }, [defaultValue])

  return {
    filters,
    setTags: updateTags,
    setAttributes: updateAttributes,
    clearFilters,
  }
}
