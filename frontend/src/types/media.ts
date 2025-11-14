export type MediaTag = {
  rawToken: string
  type: 'simple' | 'keyvalue'
  name: string
  value?: string | null
  normalized: string
}

export type MediaSummary = {
  id: string
  relativePath: string
  mediaType: 'image' | 'gif' | 'video' | 'audio' | 'pdf' | 'unknown'
  tags: MediaTag[]
  attributes: Record<string, string>
  filesize: number
  thumbnailPath?: string | null
}
