import { describe, expect, it } from 'vitest'

import { parseAttributeInput, parseTagInput } from './useTagFilters'

describe('parseTagInput', () => {
  it('normalizes casing, trims whitespace, and preserves insertion order', () => {
    const result = parseTagInput('  Sunset ,  COAST ,rating-5 , coast ')
    expect(result).toEqual(['sunset', 'coast', 'rating-5'])
  })

  it('ignores empty tokens and punctuation-only values', () => {
    const result = parseTagInput(', ,  ### ,  ')
    expect(result).toEqual([])
  })
})

describe('parseAttributeInput', () => {
  it('groups values by attribute key regardless of casing', () => {
    const { attributes, invalid } = parseAttributeInput(
      'rating:5 camera:Alpha Rating:3 camera:alpha'
    )
    expect(attributes.rating).toEqual(['5', '3'])
    expect(attributes.camera).toEqual(['alpha'])
    expect(invalid).toEqual([])
  })

  it('records invalid tokens without discarding valid pairs', () => {
    const { attributes, invalid } = parseAttributeInput('rating:5 ,badtoken, keyonly: , :valueonly ')
    expect(attributes.rating).toEqual(['5'])
    expect(invalid).toEqual(['badtoken', 'keyonly:', ':valueonly'])
  })
})
