import type { MediaSummary } from '../types/media'

export type MediaSearchRequest = {
  tags?: string[]
  attributes?: Record<string, string[]>
  page?: number
  pageSize?: number
}

export type MediaSearchResponse = {
  items: MediaSummary[]
  total: number
  page: number
  pageSize: number
}

export async function fetchMedia(
  request: MediaSearchRequest,
  apiBaseUrl: string,
): Promise<MediaSearchResponse> {
  const params = new URLSearchParams()
  params.set('page', String(request.page ?? 1))
  params.set('pageSize', String(request.pageSize ?? 60))
  if (request.tags && request.tags.length > 0) {
    params.set('tags', request.tags.join(','))
  }

  Object.entries(request.attributes ?? {}).forEach(([key, values]) => {
    if (values.length === 0) return
    params.set(`attributes[${key}]`, values.join(','))
  })

  const response = await fetch(`${apiBaseUrl}/media?${params.toString()}`)
  if (!response.ok) {
    throw new Error(`Search failed with status ${response.status}`)
  }

  const payload = (await response.json()) as MediaSearchResponse
  return payload
}
