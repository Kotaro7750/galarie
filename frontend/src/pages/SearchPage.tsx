import { useEffect, useMemo, useState } from 'react'
import {
  Alert,
  Box,
  Button,
  Card,
  CardActions,
  CardContent,
  CardMedia,
  Chip,
  CircularProgress,
  Divider,
  Grid,
  Snackbar,
  Stack,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material'
import RefreshRoundedIcon from '@mui/icons-material/RefreshRounded'
import { useMutation } from '@tanstack/react-query'

import type { MediaSummary } from '../types/media'
import { fetchMedia } from '../services/mediaClient'
import type { MediaSearchRequest } from '../services/mediaClient'
import { usePersistedFilters } from '../hooks/usePersistedFilters'
import { resolveStreamUrl, resolveThumbnailUrl } from '../utils/mediaUrls'

type AttributeMap = Record<string, string[]>

type SearchPageProps = {
  apiBaseUrl: string
}

export function SearchPage({ apiBaseUrl }: SearchPageProps) {
  const { filters, setTags: persistTags, setAttributes: persistAttributes } = usePersistedFilters({
    tags: 'sunset, coast',
    attributes: { rating: ['5'] },
  })
  const tagInput = filters.tags
  const [attrKey, setAttrKey] = useState('rating')
  const [attrValue, setAttrValue] = useState('5')
  const attributes = filters.attributes
  const [formError, setFormError] = useState<string | null>(null)
  const [toastOpen, setToastOpen] = useState(false)

  const searchMutation = useMutation({
    mutationFn: (payload: MediaSearchRequest) => fetchMedia(payload, apiBaseUrl),
  })

  const isLoading = searchMutation.isPending

  const handleAddAttribute = () => {
    const key = attrKey.trim().toLowerCase()
    const value = attrValue.trim()
    if (!key || !value) return
    const existing = attributes[key] ?? []
    if (existing.includes(value)) {
      return
    }
    persistAttributes({ ...attributes, [key]: [...existing, value] })
    setAttrValue('')
  }

  const handleRemoveAttribute = (key: string, value: string) => {
    const values = attributes[key]?.filter((item) => item !== value) ?? []
    if (values.length === 0) {
      const { [key]: _unused, ...rest } = attributes
      persistAttributes(rest)
    } else {
      persistAttributes({ ...attributes, [key]: values })
    }
  }

  const handleSearch = () => {
    const tags = tagInput
      .split(',')
      .map((token) => token.trim().toLowerCase())
      .filter((token) => token.length > 0)
    if (tags.length === 0) {
      setFormError('Enter at least one tag to search')
      return
    }
    setFormError(null)
    searchMutation.mutate({ tags, attributes, page: 1, pageSize: 60 })
  }

  const attributeChips = useMemo(() => Object.entries(attributes), [attributes])
  const fetchedItems = searchMutation.data?.items ?? []
  const totalResults = searchMutation.data?.total ?? 0

  useEffect(() => {
    if (searchMutation.isError) {
      setToastOpen(true)
    }
  }, [searchMutation.isError])

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
                error={Boolean(formError)}
                FormHelperTextProps={{ sx: formError ? { color: 'error.main' } : undefined }}
                helperText={formError ?? 'Every tag is required (AND semantics)'}
                onChange={(event) => persistTags(event.target.value)}
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
              Results ({totalResults})
            </Typography>
            {isLoading && (
              <Stack direction="row" spacing={1} alignItems="center" color="text.secondary">
                <CircularProgress size={18} thickness={5} />
                <Typography variant="body2">Fetching latest media…</Typography>
              </Stack>
            )}
          </Stack>
          <Divider sx={{ mb: 2 }} />
          {fetchedItems.length === 0 ? (
            <Alert severity="info">No media yet – adjust filters and try again.</Alert>
          ) : (
            <Grid container spacing={2}>
              {fetchedItems.map((media) => (
                <Grid item xs={12} sm={6} md={4} lg={3} key={media.id}>
                  <MediaCard media={media} apiBaseUrl={apiBaseUrl} />
                </Grid>
              ))}
            </Grid>
          )}
        </CardContent>
      </Card>

      <Snackbar
        open={toastOpen}
        autoHideDuration={5000}
        onClose={() => setToastOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
      >
        <Alert
          severity="error"
          onClose={() => setToastOpen(false)}
          action={
            <Button
              color="inherit"
              size="small"
              onClick={() => {
                setToastOpen(false)
                handleSearch()
              }}
            >
              Retry
            </Button>
          }
        >
          {resolveErrorMessage(searchMutation.error)}
        </Alert>
      </Snackbar>
    </Stack>
  )
}

type MediaCardProps = {
  media: MediaSummary
  apiBaseUrl: string
}

function MediaCard({ media, apiBaseUrl }: MediaCardProps) {
  const thumbnailSrc = resolveThumbnailUrl(media.thumbnailPath, apiBaseUrl)
  const tagNames = media.tags.map((tag) => tag.normalized)
  const inlineStreamUrl = resolveStreamUrl(apiBaseUrl, media.id)
  const downloadUrl = resolveStreamUrl(apiBaseUrl, media.id, 'attachment')

  const openInlinePreview = () => {
    if (typeof window === 'undefined') return
    window.open(inlineStreamUrl, '_blank', 'noopener,noreferrer')
  }

  return (
    <Card variant="outlined" sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <CardMedia
        component="img"
        height={180}
        image={thumbnailSrc}
        alt={media.relativePath}
        onClick={openInlinePreview}
        sx={{ cursor: 'pointer' }}
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
      <CardActions sx={{ justifyContent: 'flex-end', pt: 0 }}>
        <Button size="small" onClick={openInlinePreview}>
          Open
        </Button>
        <Button
          size="small"
          component="a"
          href={downloadUrl}
          target="_blank"
          rel="noopener noreferrer"
        >
          Download
        </Button>
      </CardActions>
    </Card>
  )
}

function resolveErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }
  return 'Unable to load media results'
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
