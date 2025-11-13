import { useCallback, useMemo, useState } from 'react'

export type TagFilters = {
  tags: string[]
  attributes: Record<string, string[]>
}

export type AttributeParseResult = {
  attributes: Record<string, string[]>
  invalid: string[]
}

const HAS_ALPHANUMERIC = /[a-z0-9]/i

export function parseTagInput(raw: string): string[] {
  if (!raw) return []

  const normalized: string[] = []
  const seen = new Set<string>()
  raw
    .split(',')
    .map((token) => token.trim().toLowerCase())
    .forEach((token) => {
      if (!token) return
      if (!HAS_ALPHANUMERIC.test(token)) return
      if (seen.has(token)) return
      seen.add(token)
      normalized.push(token)
    })

  return normalized
}

export function parseAttributeInput(raw: string): AttributeParseResult {
  const attributes: Record<string, string[]> = {}
  const invalid: string[] = []
  if (!raw) {
    return { attributes, invalid }
  }

  const valuesSeen: Record<string, Set<string>> = {}
  raw
    .split(/[,\s]+/)
    .map((token) => token.trim())
    .filter(Boolean)
    .forEach((token) => {
      const delimiterIndex = token.indexOf(':')
      if (delimiterIndex <= 0 || delimiterIndex === token.length - 1) {
        invalid.push(token)
        return
      }

      const key = token.slice(0, delimiterIndex).trim().toLowerCase()
      const value = token.slice(delimiterIndex + 1).trim().toLowerCase()

      if (!key || !value) {
        invalid.push(token)
        return
      }

      const seenForKey = (valuesSeen[key] = valuesSeen[key] ?? new Set())
      if (seenForKey.has(value)) return

      seenForKey.add(value)
      attributes[key] = [...(attributes[key] ?? []), value]
    })

  return { attributes, invalid }
}

export function useTagFilters(initialValue: TagFilters = { tags: [], attributes: {} }) {
  const [filters, setFilters] = useState<TagFilters>(initialValue)
  const baseFilters = useMemo(() => initialValue, [initialValue])

  const setTagText = useCallback((raw: string) => {
    setFilters((prev) => ({ ...prev, tags: parseTagInput(raw) }))
  }, [])

  const setAttributeText = useCallback((raw: string) => {
    const parsed = parseAttributeInput(raw)
    setFilters((prev) => ({ ...prev, attributes: parsed.attributes }))
    return parsed.invalid
  }, [])

  const reset = useCallback(() => {
    setFilters(baseFilters)
  }, [baseFilters])

  return {
    filters,
    setTagText,
    setAttributeText,
    reset,
  }
}
