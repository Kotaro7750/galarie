import { useMemo, useState } from 'react'
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  CardMedia,
  Chip,
  CircularProgress,
  Divider,
  Grid,
  Stack,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material'
import RefreshRoundedIcon from '@mui/icons-material/RefreshRounded'

import { MediaSummary } from '../types/media'

type AttributeMap = Record<string, string[]>

type SearchPageProps = {
  apiBaseUrl: string
}

export function SearchPage({ apiBaseUrl }: SearchPageProps) {
  const [tagInput, setTagInput] = useState('sunset, coast')
  const [attrKey, setAttrKey] = useState('rating')
  const [attrValue, setAttrValue] = useState('5')
  const [attributes, setAttributes] = useState<AttributeMap>({ rating: ['5'] })
  const [results, setResults] = useState<MediaSummary[]>(SAMPLE_RESULTS)
  const [isLoading, setIsLoading] = useState(false)

  const handleAddAttribute = () => {
    const key = attrKey.trim().toLowerCase()
    const value = attrValue.trim()
    if (!key || !value) return
    setAttributes((prev) => {
      const existing = prev[key] ?? []
      if (existing.includes(value)) {
        return prev
      }
      return { ...prev, [key]: [...existing, value] }
    })
    setAttrValue('')
  }

  const handleRemoveAttribute = (key: string, value: string) => {
    setAttributes((prev) => {
      const values = prev[key]?.filter((item) => item !== value) ?? []
      if (values.length === 0) {
        const { [key]: _unused, ...rest } = prev
        return rest
      }
      return { ...prev, [key]: values }
    })
  }

  const handleSearch = () => {
    setIsLoading(true)
    // Placeholder for T115 – replace with TanStack Query once the media client exists.
    setTimeout(() => {
      setResults(SAMPLE_RESULTS)
      setIsLoading(false)
    }, 600)
  }

  const attributeChips = useMemo(() => Object.entries(attributes), [attributes])

  return (
    <Stack spacing={4} sx={{ py: { xs: 4, md: 6 } }}>
      <Card variant="outlined">
        <CardContent>
          <Stack spacing={3}>
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
              <TextField
                fullWidth
                label="Tags (comma separated)"
                helperText="Every tag is required (AND semantics)"
                value={tagInput}
                onChange={(event) => setTagInput(event.target.value)}
              />
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={2} width={{ xs: '100%', md: 380 }}>
                <TextField
                  fullWidth
                  label="Attribute key"
                  value={attrKey}
                  onChange={(event) => setAttrKey(event.target.value)}
                />
                <TextField
                  fullWidth
                  label="Value"
                  value={attrValue}
                  onChange={(event) => setAttrValue(event.target.value)}
                />
                <Button variant="outlined" onClick={handleAddAttribute} sx={{ whiteSpace: 'nowrap' }}>
                  Add
                </Button>
              </Stack>
            </Stack>

            {attributeChips.length > 0 && (
              <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                {attributeChips.flatMap(([key, values]) =>
                  values.map((value) => (
                    <Chip
                      key={`${key}-${value}`}
                      label={`${key}:${value}`}
                      color="primary"
                      variant="outlined"
                      onDelete={() => handleRemoveAttribute(key, value)}
                      sx={{ mb: 1 }}
                    />
                  )),
                )}
              </Stack>
            )}

            <Stack direction="row" justifyContent="flex-end">
              <Button
                variant="contained"
                color="primary"
                endIcon={<RefreshRoundedIcon />}
                onClick={handleSearch}
                disabled={isLoading}
              >
                {isLoading ? 'Searching…' : 'Run search'}
              </Button>
            </Stack>
          </Stack>
        </CardContent>
      </Card>

      <Card variant="outlined">
        <CardContent>
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={2}>
            <Typography variant="h6" sx={{ fontWeight: 600 }}>
              Results ({results.length})
            </Typography>
            {isLoading && (
              <Stack direction="row" spacing={1} alignItems="center" color="text.secondary">
                <CircularProgress size={18} thickness={5} />
                <Typography variant="body2">Fetching latest media…</Typography>
              </Stack>
            )}
          </Stack>
          <Divider sx={{ mb: 2 }} />
          {results.length === 0 ? (
            <Alert severity="info">No media yet – adjust filters and try again.</Alert>
          ) : (
            <Grid container spacing={2}>
              {results.map((media) => (
                <Grid item xs={12} sm={6} md={4} lg={3} key={media.id}>
                  <MediaCard media={media} apiBaseUrl={apiBaseUrl} />
                </Grid>
              ))}
            </Grid>
          )}
        </CardContent>
      </Card>
    </Stack>
  )
}

type MediaCardProps = {
  media: MediaSummary
  apiBaseUrl: string
}

function MediaCard({ media, apiBaseUrl }: MediaCardProps) {
  const thumbnailSrc = resolveThumbnail(media.thumbnailPath, apiBaseUrl)
  const tagNames = media.tags.map((tag) => tag.normalized)

  return (
    <Card variant="outlined" sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <CardMedia
        component="img"
        height={180}
        image={thumbnailSrc}
        alt={media.relativePath}
        onError={(event) => {
          ;(event.target as HTMLImageElement).src = 'https://placehold.co/320x200?text=Media'
        }}
      />
      <CardContent sx={{ flexGrow: 1 }}>
        <Stack spacing={1}>
          <Stack direction="row" justifyContent="space-between" alignItems="center">
            <Box sx={{ minWidth: 0 }}>
              <Tooltip title={media.relativePath}>
                <Typography variant="subtitle2" noWrap sx={{ fontWeight: 600 }}>
                  {media.relativePath}
                </Typography>
              </Tooltip>
            </Box>
            <Chip size="small" label={media.mediaType.toUpperCase()} sx={{ textTransform: 'uppercase' }} />
          </Stack>
          <Typography variant="caption" color="text.secondary">
            {media.filesize.toLocaleString()} bytes
          </Typography>
          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            {tagNames.map((tag) => (
              <Chip key={tag} label={tag} size="small" sx={{ mb: 0.5 }} />
            ))}
          </Stack>
        </Stack>
      </CardContent>
    </Card>
  )
}

function resolveThumbnail(path: string | null | undefined, apiBaseUrl: string) {
  if (!path) {
    return 'https://placehold.co/320x200?text=Media'
  }
  if (path.startsWith('http')) {
    return path
  }
  return `${apiBaseUrl}${path}`
}

const SAMPLE_RESULTS: MediaSummary[] = [
  {
    id: 'sunset_A',
    relativePath: 'photos/sunsets/sunset_A.png',
    mediaType: 'image',
    filesize: 24567,
    thumbnailPath: 'https://placehold.co/320x200/4338ca/FFFFFF?text=Sunset',
    attributes: { rating: '5', location: 'okinawa' },
    tags: [
      { rawToken: 'sunset', type: 'simple', name: 'sunset', value: null, normalized: 'sunset' },
      { rawToken: 'coast', type: 'simple', name: 'coast', value: null, normalized: 'coast' },
      { rawToken: 'rating-5', type: 'kv', name: 'rating', value: '5', normalized: 'rating=5' },
    ],
  },
  {
    id: 'macro_B',
    relativePath: 'macro/macro_leaf.gif',
    mediaType: 'gif',
    filesize: 15234,
    thumbnailPath: 'https://placehold.co/320x200/0f172a/FFFFFF?text=Macro',
    attributes: { rating: '4', subject: 'nature' },
    tags: [
      { rawToken: 'macro', type: 'simple', name: 'macro', value: null, normalized: 'macro' },
      { rawToken: 'rating-4', type: 'kv', name: 'rating', value: '4', normalized: 'rating=4' },
    ],
  },
  {
    id: 'video_C',
    relativePath: 'video/skate_session.mp4',
    mediaType: 'video',
    filesize: 80567,
    thumbnailPath: 'https://placehold.co/320x200/0891b2/FFFFFF?text=Video',
    attributes: { rating: '3', type: 'skate' },
    tags: [
      { rawToken: 'skate', type: 'simple', name: 'skate', value: null, normalized: 'skate' },
      { rawToken: 'rating-3', type: 'kv', name: 'rating', value: '3', normalized: 'rating=3' },
    ],
  },
]
