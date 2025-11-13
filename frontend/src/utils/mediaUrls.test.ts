import { describe, expect, it } from 'vitest'

import { resolveStreamUrl, resolveThumbnailUrl } from './mediaUrls'

describe('media url helpers', () => {
  describe('resolveStreamUrl', () => {
    it('returns inline url by default', () => {
      expect(resolveStreamUrl('http://localhost:8080/api/v1', 'abc')).toBe(
        'http://localhost:8080/api/v1/media/abc/stream',
      )
    })

    it('appends disposition for attachments', () => {
      expect(resolveStreamUrl('http://localhost:8080/api/v1/', 'abc', 'attachment')).toBe(
        'http://localhost:8080/api/v1/media/abc/stream?disposition=attachment',
      )
    })

    it('escapes media identifiers', () => {
      expect(resolveStreamUrl('http://localhost/api', 'sp aces')).toBe(
        'http://localhost/api/media/sp%20aces/stream',
      )
    })
  })

  describe('resolveThumbnailUrl', () => {
    it('returns placeholder when path missing', () => {
      expect(resolveThumbnailUrl(null, 'http://localhost/api')).toContain('placehold.co')
    })

    it('passes through absolute urls', () => {
      expect(resolveThumbnailUrl('https://cdn.example.com/thumb.jpg', 'http://localhost/api')).toBe(
        'https://cdn.example.com/thumb.jpg',
      )
    })

    it('prefixes relative thumbnails with API base', () => {
      expect(resolveThumbnailUrl('/media/abc/thumbnail', 'http://localhost/api/')).toBe(
        'http://localhost/api/media/abc/thumbnail',
      )
    })
  })
})
