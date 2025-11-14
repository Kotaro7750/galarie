import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
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
import { useInfiniteQuery } from '@tanstack/react-query'

import type { MediaSummary } from '../types/media'
import { fetchMedia } from '../services/mediaClient'
import type { MediaSearchRequest } from '../services/mediaClient'
import { usePersistedFilters, type PersistedFilters } from '../hooks/usePersistedFilters'
import { resolveStreamUrl, resolveThumbnailUrl } from '../utils/mediaUrls'

type AttributeMap = Record<string, string[]>

type SearchPageProps = {
  apiBaseUrl: string
}

export function SearchPage({ apiBaseUrl }: SearchPageProps) {
  const { filters, setTags: persistTags, setAttributes: persistAttributes } = usePersistedFilters({
    tags: '',
    attributes: {},
  })
  const tagInput = filters.tags
  const [attrKey, setAttrKey] = useState('rating')
  const [attrValue, setAttrValue] = useState('5')
  const attributes = filters.attributes
  const [toastOpen, setToastOpen] = useState(false)
  const [previewMedia, setPreviewMedia] = useState<MediaSummary | null>(null)
  const [appliedFilters, setAppliedFilters] = useState<PersistedFilters>(() => ({
    tags: filters.tags,
    attributes: cloneAttributes(filters.attributes),
  }))
  const loadMoreRef = useRef<HTMLDivElement | null>(null)
  const parsedAppliedTags = useMemo(() => parseTagInput(appliedFilters.tags), [appliedFilters.tags])
  const searchQuery = useInfiniteQuery({
    queryKey: ['media-search', apiBaseUrl, appliedFilters],
    initialPageParam: 1,
    queryFn: ({ pageParam }) => {
      const payload: MediaSearchRequest = {
        attributes: appliedFilters.attributes,
        page: pageParam,
        pageSize: 60,
      }
      if (parsedAppliedTags.length > 0) {
        payload.tags = parsedAppliedTags
      }
      return fetchMedia(payload, apiBaseUrl)
    },
    getNextPageParam: (lastPage) => {
      const hasMore = lastPage.page * lastPage.pageSize < lastPage.total
      return hasMore ? lastPage.page + 1 : undefined
    },
  })
  const {
    data,
    fetchNextPage,
    hasNextPage,
    isFetching,
    isFetchingNextPage,
    isError,
    error,
    refetch,
    status,
  } = searchQuery
  const flattenedItems = data?.pages.flatMap((page) => page.items) ?? []
  const totalResults = data?.pages[0]?.total ?? 0
  const isInitialLoading = status === 'loading' && !data
  const isRefreshing = isFetching && !isFetchingNextPage

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
      const nextAttributes = { ...attributes }
      delete nextAttributes[key]
      persistAttributes(nextAttributes)
    } else {
      persistAttributes({ ...attributes, [key]: values })
    }
  }

  const handleSearch = useCallback(() => {
    setToastOpen(false)
    setAppliedFilters({
      tags: filters.tags,
      attributes: cloneAttributes(filters.attributes),
    })
  }, [filters.attributes, filters.tags])

  const attributeChips = useMemo(() => Object.entries(attributes), [attributes])

  useEffect(() => {
    if (!isError) return
    const id = window.setTimeout(() => setToastOpen(true), 0)
    return () => window.clearTimeout(id)
  }, [isError])

  useEffect(() => {
    const node = loadMoreRef.current
    if (!node || !hasNextPage) {
      return
    }
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasNextPage && !isFetchingNextPage) {
          fetchNextPage()
        }
      },
      { rootMargin: '200px 0px' },
    )
    observer.observe(node)
    return () => observer.disconnect()
  }, [fetchNextPage, hasNextPage, isFetchingNextPage])

  return (
    <Stack spacing={4} sx={{ py: { xs: 4, md: 6 } }}>
      <Card variant="outlined">
        <CardContent>
          <Stack spacing={3}>
            <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
              <TextField
                fullWidth
                label="Tags (comma separated)"
                helperText="Optional: AND matches on tag names (simple or KV keys)"
                value={tagInput}
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
                disabled={isRefreshing}
              >
                {isRefreshing ? 'Refreshing…' : 'Apply filters'}
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
            {(isInitialLoading || isRefreshing) && (
              <Stack direction="row" spacing={1} alignItems="center" color="text.secondary">
                <CircularProgress size={18} thickness={5} />
                <Typography variant="body2">
                  {isInitialLoading ? 'Loading media…' : 'Fetching latest media…'}
                </Typography>
              </Stack>
            )}
          </Stack>
          <Divider sx={{ mb: 2 }} />
          {isInitialLoading ? (
            <Stack alignItems="center" spacing={2} py={6}>
              <CircularProgress />
              <Typography variant="body2" color="text.secondary">
                Loading catalog…
              </Typography>
            </Stack>
          ) : flattenedItems.length === 0 ? (
            <Alert severity="info">No media matched. Add filters or try browsing without tags.</Alert>
          ) : (
            <>
              <Grid container spacing={2}>
                {flattenedItems.map((media) => (
                  <Grid item xs={12} sm={6} md={4} lg={3} key={media.id}>
                    <MediaCard
                      media={media}
                      apiBaseUrl={apiBaseUrl}
                      onPreview={() => setPreviewMedia(media)}
                    />
                  </Grid>
                ))}
              </Grid>
              <Box ref={loadMoreRef} sx={{ height: 8 }} />
              {isFetchingNextPage && (
                <Stack direction="row" spacing={1} alignItems="center" justifyContent="center" mt={3}>
                  <CircularProgress size={18} thickness={5} />
                  <Typography variant="body2" color="text.secondary">
                    Loading more…
                  </Typography>
                </Stack>
              )}
              {!hasNextPage && flattenedItems.length > 0 && (
                <Typography variant="caption" color="text.secondary" display="block" textAlign="center" mt={3}>
                  End of results
                </Typography>
              )}
            </>
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
                refetch()
              }}
            >
              Retry
            </Button>
          }
        >
          {resolveErrorMessage(error)}
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

function MediaPreviewOverlay({ media, apiBaseUrl, onClose }: MediaPreviewOverlayProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onClose])

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

  const handleBackdropClick = (event: React.MouseEvent<HTMLDivElement>) => {
    if (event.target === event.currentTarget) {
      onClose()
    }
  }

  return (
    <Portal>
      <Box
        onClick={handleBackdropClick}
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

function parseTagInput(input: string): string[] {
  return input
    .split(',')
    .map((token) => token.trim().toLowerCase())
    .filter((token) => token.length > 0)
}

function cloneAttributes(source: AttributeMap): AttributeMap {
  const entries = Object.entries(source).map(([key, values]) => [key, [...values]] as const)
  return Object.fromEntries(entries)
}
