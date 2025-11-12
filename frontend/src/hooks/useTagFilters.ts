export type TagFilters = {
  tags: string[]
  attributes: Record<string, string[]>
}

export type AttributeParseResult = {
  attributes: Record<string, string[]>
  invalid: string[]
}

export function parseTagInput(_raw: string): string[] {
  throw new Error('parseTagInput not implemented yet')
}

export function parseAttributeInput(_raw: string): AttributeParseResult {
  throw new Error('parseAttributeInput not implemented yet')
}

export function useTagFilters(): never {
  throw new Error('useTagFilters hook not implemented yet')
}
