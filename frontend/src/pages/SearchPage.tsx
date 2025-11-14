import { useMemo, useState } from 'react'
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
  IconButton,
  Portal,
  Snackbar,
  Stack,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material'
import RefreshRoundedIcon from '@mui/icons-material/RefreshRounded'
import CloseRoundedIcon from '@mui/icons-material/CloseRounded'
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
  const [previewMedia, setPreviewMedia] = useState<MediaSummary | null>(null)

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
                  <MediaCard
                    media={media}
                    apiBaseUrl={apiBaseUrl}
                    onPreview={() => setPreviewMedia(media)}
                  />
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
      {previewMedia && (
        <MediaPreviewOverlay
          media={previewMedia}
          apiBaseUrl={apiBaseUrl}
          onClose={() => setPreviewMedia(null)}
        />
      )}
    </Stack>
  )
}

type MediaCardProps = {
  media: MediaSummary
  apiBaseUrl: string
  onPreview: () => void
}

function MediaCard({ media, apiBaseUrl, onPreview }: MediaCardProps) {
  const thumbnailSrc = resolveThumbnailUrl(media.thumbnailPath, apiBaseUrl)
  const tagNames = media.tags.map((tag) => tag.normalized)
  const inlineStreamUrl = resolveStreamUrl(apiBaseUrl, media.id)
  const downloadUrl = resolveStreamUrl(apiBaseUrl, media.id, 'attachment')

  const openNewTab = () => {
    if (typeof window === 'undefined') return
    window.open(inlineStreamUrl, '_blank', 'noopener,noreferrer')
  }

  return (
    <Card
      variant="outlined"
      sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}
    >
      <CardMedia
        component="img"
        height={180}
        image={thumbnailSrc}
        alt={media.relativePath}
        onClick={onPreview}
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
        <Button size="small" onClick={onPreview}>
          Preview
        </Button>
        <Button
          size="small"
          onClick={openNewTab}
        >
          Open in new tab
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

type MediaPreviewOverlayProps = {
  media: MediaSummary | null
  apiBaseUrl: string
  onClose: () => void
}

function MediaPreviewOverlay({
  media,
  apiBaseUrl,
  onClose,
}: MediaPreviewOverlayProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)

  if (!media || typeof document === 'undefined') {
    return null
  }

  const inlineStreamUrl = resolveStreamUrl(apiBaseUrl, media.id)
  const downloadUrl = resolveStreamUrl(apiBaseUrl, media.id, 'attachment')

  const handleFullscreen = () => {
    const node = containerRef.current
    if (node && 'requestFullscreen' in node) {
      // @ts-expect-error: requestFullscreen exists in modern browsers
      node.requestFullscreen?.().catch(() => {
        /* ignore */
      })
    }
  }

  return (
    <Portal>
      <Box
        sx={{
          position: 'fixed',
          inset: 0,
          bgcolor: 'rgba(8, 15, 30, 0.85)',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          zIndex: (theme) => theme.zIndex.modal,
          transition: 'opacity 120ms ease',
        }}
      >
        <Card
          ref={containerRef}
          sx={{
            width: 'min(90vw, 960px)',
            maxHeight: '90vh',
            display: 'flex',
            flexDirection: 'column',
            gap: 2,
            p: 3,
          }}
        >
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Stack spacing={0.5}>
              <Typography variant="subtitle1" fontWeight={600} noWrap>
                {media.relativePath}
              </Typography>
              <Typography variant="caption" color="text.secondary">
                {media.mediaType.toUpperCase()} · {media.filesize.toLocaleString()} bytes
              </Typography>
            </Stack>
            <IconButton aria-label="Close preview" onClick={onClose} size="small">
              <CloseRoundedIcon />
            </IconButton>
          </Stack>

          <Box
            sx={{
              flexGrow: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              minHeight: { xs: 260, md: 380 },
              bgcolor: 'grey.900',
              borderRadius: 2,
              overflow: 'hidden',
            }}
          >
            {renderPreviewMedia(media, inlineStreamUrl)}
          </Box>

          <Stack direction={{ xs: 'column', sm: 'row' }} justifyContent="flex-end" spacing={1.5}>
            <Button onClick={handleFullscreen} variant="outlined">
              Fullscreen
            </Button>
            <Button component="a" href={inlineStreamUrl} target="_blank" rel="noopener noreferrer">
              Open in new tab
            </Button>
            <Button
              component="a"
              href={downloadUrl}
              target="_blank"
              rel="noopener noreferrer"
              variant="contained"
            >
              Download
            </Button>
          </Stack>
        </Card>
      </Box>
    </Portal>
  )
}

function renderPreviewMedia(media: MediaSummary, streamUrl: string) {
  const commonStyles = {
    maxWidth: '100%',
    maxHeight: '80vh',
  }

  switch (media.mediaType) {
    case 'image':
    case 'gif':
      return (
        <Box
          component="img"
          src={streamUrl}
          alt={media.relativePath}
          sx={{ ...commonStyles, objectFit: 'contain' }}
        />
      )
    case 'video':
      return (
        <Box component="video" src={streamUrl} controls autoPlay muted loop sx={commonStyles} />
      )
    case 'audio':
      return (
        <Stack spacing={2} alignItems="center" width="100%">
          <Typography variant="body2" color="text.secondary">
            Audio preview
          </Typography>
          <audio src={streamUrl} controls autoPlay style={{ width: '100%' }} />
        </Stack>
      )
    case 'pdf':
      return (
        <Box
          component="iframe"
          src={streamUrl}
          sx={{ border: 0, width: '100%', height: '70vh', bgcolor: 'white' }}
        />
      )
    default:
      return (
        <Stack spacing={2} alignItems="center">
          <Typography color="text.secondary">
            Preview not available. Use Open or Download to view this media.
          </Typography>
        </Stack>
      )
  }
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
