export type StreamDisposition = 'inline' | 'attachment'

export function resolveStreamUrl(
  apiBaseUrl: string,
  mediaId: string,
  disposition: StreamDisposition = 'inline',
): string {
  const normalizedBase = apiBaseUrl.replace(/\/$/, '')
  const encodedId = encodeURIComponent(mediaId)
  const baseUrl = `${normalizedBase}/media/${encodedId}/stream`

  if (disposition === 'inline') {
    return baseUrl
  }

  return `${baseUrl}?disposition=${disposition}`
}

export function resolveThumbnailUrl(path: string | null | undefined, apiBaseUrl: string): string {
  if (!path) {
    return 'https://placehold.co/320x200?text=Media'
  }
  if (path.startsWith('http')) {
    return path
  }
  const normalizedBase = apiBaseUrl.replace(/\/$/, '')
  return `${normalizedBase}${path}`
}
