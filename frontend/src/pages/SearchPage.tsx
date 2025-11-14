import { useCallback, useEffect, useMemo, useRef, useState, type KeyboardEvent } from 'react'
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
  InputAdornment,
  Portal,
  Snackbar,
  Stack,
  TextField,
  Tooltip,
  Typography,
} from '@mui/material'
import CloseRoundedIcon from '@mui/icons-material/CloseRounded'
import InfoOutlinedIcon from '@mui/icons-material/InfoOutlined'
import VisibilityRoundedIcon from '@mui/icons-material/VisibilityRounded'
import OpenInNewRoundedIcon from '@mui/icons-material/OpenInNewRounded'
import DownloadRoundedIcon from '@mui/icons-material/DownloadRounded'
import FullscreenRoundedIcon from '@mui/icons-material/FullscreenRounded'
import AddRoundedIcon from '@mui/icons-material/AddRounded'
import { useInfiniteQuery } from '@tanstack/react-query'

import type { MediaSummary, MediaTag } from '../types/media'
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
    tags: [],
    attributes: {},
  })
  const confirmedTags = filters.tags
  const [tagDraft, setTagDraft] = useState('')
  const [attrValue, setAttrValue] = useState('')
  const tagInputRef = useRef<HTMLInputElement | null>(null)
  const tagGuidance = 'Press Enter or comma to confirm plain tags. Provide a value to create key:value filters.'
  const valueGuidance = 'When a value is present, the left input is treated as the key (camera + nikon => camera:nikon).'
  const attributes = filters.attributes
  const [toastOpen, setToastOpen] = useState(false)
  const [previewIndex, setPreviewIndex] = useState<number | null>(null)
  const [appliedFilters, setAppliedFilters] = useState<PersistedFilters>(() => ({
    tags: cloneTags(filters.tags),
    attributes: cloneAttributes(filters.attributes),
  }))
  const loadMoreRef = useRef<HTMLDivElement | null>(null)
  const appliedTags = appliedFilters.tags
  const searchQuery = useInfiniteQuery({
    queryKey: ['media-search', apiBaseUrl, appliedFilters],
    initialPageParam: 1,
    queryFn: ({ pageParam }) => {
      const payload: MediaSearchRequest = {
        attributes: appliedFilters.attributes,
        page: pageParam,
        pageSize: 60,
      }
      if (appliedTags.length > 0) {
        payload.tags = appliedTags
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
  const itemsCount = flattenedItems.length
  const totalResults = data?.pages[0]?.total ?? 0
  const isInitialLoading = status === 'loading' && !data
  const isRefreshing = isFetching && !isFetchingNextPage
  const previewMedia = typeof previewIndex === 'number' ? flattenedItems[previewIndex] ?? null : null

  useEffect(() => {
    if (previewIndex === null) return
    if (itemsCount === 0) {
      setPreviewIndex(null)
      return
    }
    if (!flattenedItems[previewIndex]) {
      setPreviewIndex((prev) => {
        if (prev === null) return prev
        const nextIndex = Math.min(prev, itemsCount - 1)
        return nextIndex >= 0 ? nextIndex : null
      })
    }
  }, [flattenedItems, itemsCount, previewIndex])

  useEffect(() => {
    setToastOpen(false)
    setAppliedFilters({
      tags: cloneTags(filters.tags),
      attributes: cloneAttributes(filters.attributes),
    })
  }, [filters.attributes, filters.tags])

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

  const openPreviewAt = useCallback((index: number) => {
    setPreviewIndex(index)
  }, [])

  const closePreview = useCallback(() => setPreviewIndex(null), [])

  const handleNavigatePreview = useCallback(
    (direction: 'next' | 'previous') => {
      setPreviewIndex((current) => {
        if (current === null || itemsCount === 0) {
          return current
        }
        if (direction === 'next') {
          return (current + 1) % itemsCount
        }
        return (current - 1 + itemsCount) % itemsCount
      })
    },
    [itemsCount],
  )

  const handleCommitTagOrAttribute = useCallback(() => {
    const normalized = normalizeTag(tagDraft)
    const value = attrValue.trim()
    if (!normalized) {
      setTagDraft('')
      return
    }
    const focusTagInput = () => tagInputRef.current?.focus()
    if (value) {
      const existing = attributes[normalized] ?? []
      if (existing.includes(value)) {
        setAttrValue('')
        setTagDraft('')
        focusTagInput()
        return
      }
      persistAttributes({ ...attributes, [normalized]: [...existing, value] })
      setAttrValue('')
      setTagDraft('')
      focusTagInput()
      return
    }
    if (confirmedTags.includes(normalized)) {
      setTagDraft('')
      setAttrValue('')
      focusTagInput()
      return
    }
    persistTags([...confirmedTags, normalized])
    setTagDraft('')
    setAttrValue('')
    focusTagInput()
  }, [attrValue, attributes, confirmedTags, persistAttributes, persistTags, tagDraft])

  const handleRemoveTag = useCallback(
    (tag: string) => {
      persistTags(confirmedTags.filter((item) => item !== tag))
    },
    [confirmedTags, persistTags],
  )

  const handleAppendTagFromCard = useCallback(
    (tag: MediaTag) => {
      if (tag.type === 'keyvalue' && tag.value) {
        const key = normalizeTag(tag.name)
        const value = tag.value.trim()
        if (!key || !value) {
          return
        }
        const existingValues = attributes[key] ?? []
        if (existingValues.includes(value)) {
          return
        }
        persistAttributes({ ...attributes, [key]: [...existingValues, value] })
        tagInputRef.current?.focus()
        return
      }
      const normalized = normalizeTag(tag.normalized || tag.name)
      if (!normalized || confirmedTags.includes(normalized)) {
        return
      }
      persistTags([...confirmedTags, normalized])
      tagInputRef.current?.focus()
    },
    [attributes, confirmedTags, persistAttributes, persistTags],
  )

  const handleTagKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    if (event.key === 'Enter' || event.key === ',') {
      event.preventDefault()
      handleCommitTagOrAttribute()
    }
  }

  const attributeChips = useMemo(() => Object.entries(attributes), [attributes])
  const hasAnyFilterChips = confirmedTags.length > 0 || attributeChips.length > 0

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
            <Stack spacing={2}>
              <Stack direction={{ xs: 'column', md: 'row' }} spacing={1}>
                <TextField
                  fullWidth
                  label="Tag or key"
                  placeholder="e.g. nature or camera"
                  value={tagDraft}
                  onChange={(event) => setTagDraft(event.target.value)}
                  onKeyDown={handleTagKeyDown}
                  inputRef={tagInputRef}
                  InputProps={{
                    endAdornment: (
                      <InputAdornment position="end">
                        <Tooltip title={tagGuidance}>
                          <IconButton
                            size="small"
                            edge="end"
                            aria-label="Tag entry tips"
                            tabIndex={-1}
                            disableRipple
                            disableFocusRipple
                          >
                            <InfoOutlinedIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                      </InputAdornment>
                    ),
                  }}
                />
                <TextField
                  fullWidth
                  label="Value (optional)"
                  placeholder="e.g. nikon"
                  value={attrValue}
                  onChange={(event) => setAttrValue(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === 'Enter') {
                      event.preventDefault()
                      handleCommitTagOrAttribute()
                    }
                  }}
                  InputProps={{
                    endAdornment: (
                      <InputAdornment position="end">
                        <Tooltip title={valueGuidance}>
                          <IconButton
                            size="small"
                            edge="end"
                            aria-label="Value usage tips"
                            tabIndex={-1}
                            disableRipple
                            disableFocusRipple
                          >
                            <InfoOutlinedIcon fontSize="small" />
                          </IconButton>
                        </Tooltip>
                      </InputAdornment>
                    ),
                  }}
                />
                <Box sx={{ display: 'flex', alignItems: 'center' }}>
                  <Tooltip title={attrValue.trim() ? 'Add key:value filter' : 'Add tag'}>
                    <span>
                      <IconButton
                        color="primary"
                        onClick={handleCommitTagOrAttribute}
                        disabled={!tagDraft.trim()}
                        sx={{ alignSelf: 'center' }}
                      >
                        <AddRoundedIcon />
                      </IconButton>
                    </span>
                  </Tooltip>
                </Box>
              </Stack>
              {hasAnyFilterChips && (
                <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
                  {confirmedTags.map((tag) => (
                    <Chip
                      key={tag}
                      label={tag}
                      color="secondary"
                      variant="outlined"
                      onDelete={() => handleRemoveTag(tag)}
                      sx={{ mb: 1 }}
                    />
                  ))}
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
                {flattenedItems.map((media, index) => (
                  <Grid item xs={12} sm={6} md={4} lg={3} key={media.id}>
                    <MediaCard
                      media={media}
                      apiBaseUrl={apiBaseUrl}
                      onPreview={() => openPreviewAt(index)}
                      onTagSelect={handleAppendTagFromCard}
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
          onClose={closePreview}
          onNavigate={handleNavigatePreview}
          onTagSelect={handleAppendTagFromCard}
        />
      )}
    </Stack>
  )
}

type MediaCardProps = {
  media: MediaSummary
  apiBaseUrl: string
  onPreview: () => void
  onTagSelect: (tag: MediaTag) => void
}

function MediaCard({ media, apiBaseUrl, onPreview, onTagSelect }: MediaCardProps) {
  const thumbnailSrc = resolveThumbnailUrl(media.thumbnailPath, apiBaseUrl)
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
            {media.tags.map((tag) => (
              <Chip
                key={tag.rawToken}
                label={tag.value ? `${tag.name}:${tag.value}` : tag.name}
                size="small"
                onClick={() => onTagSelect(tag)}
                sx={{ mb: 0.5, cursor: 'pointer' }}
              />
            ))}
          </Stack>
        </Stack>
      </CardContent>
      <CardActions sx={{ justifyContent: 'flex-end', pt: 0 }}>
        <Tooltip title="Preview">
          <IconButton aria-label="Preview" onClick={onPreview}>
            <VisibilityRoundedIcon fontSize="small" />
          </IconButton>
        </Tooltip>
        <Tooltip title="Open in new tab">
          <IconButton aria-label="Open in new tab" onClick={openNewTab}>
            <OpenInNewRoundedIcon fontSize="small" />
          </IconButton>
        </Tooltip>
        <Tooltip title="Download">
          <IconButton
            aria-label="Download"
            component="a"
            href={downloadUrl}
            target="_blank"
            rel="noopener noreferrer"
          >
            <DownloadRoundedIcon fontSize="small" />
          </IconButton>
        </Tooltip>
      </CardActions>
    </Card>
  )
}

type MediaPreviewOverlayProps = {
  media: MediaSummary | null
  apiBaseUrl: string
  onClose: () => void
  onNavigate: (direction: 'next' | 'previous') => void
  onTagSelect: (tag: MediaTag) => void
}

function MediaPreviewOverlay({ media, apiBaseUrl, onClose, onNavigate, onTagSelect }: MediaPreviewOverlayProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const swipeStartX = useRef<number | null>(null)

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose()
        return
      }
      if (event.key === 'ArrowRight') {
        event.preventDefault()
        onNavigate('next')
        return
      }
      if (event.key === 'ArrowLeft') {
        event.preventDefault()
        onNavigate('previous')
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onClose, onNavigate])

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

  const handlePointerDown = (event: React.PointerEvent<HTMLDivElement>) => {
    swipeStartX.current = event.clientX
  }

  const handlePointerUp = (event: React.PointerEvent<HTMLDivElement>) => {
    if (swipeStartX.current === null) {
      return
    }
    const deltaX = event.clientX - swipeStartX.current
    const threshold = 40
    if (Math.abs(deltaX) > threshold) {
      if (deltaX > 0) {
        onNavigate('previous')
      } else {
        onNavigate('next')
      }
    }
    swipeStartX.current = null
  }

  const handlePointerLeave = () => {
    swipeStartX.current = null
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
        onPointerDown={handlePointerDown}
        onPointerUp={handlePointerUp}
        onPointerLeave={handlePointerLeave}
        onPointerCancel={handlePointerLeave}
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

          <Stack direction="row" spacing={1} flexWrap="wrap" useFlexGap>
            {media.tags.map((tag) => (
              <Chip
                key={`${media.id}-${tag.rawToken}`}
                label={tag.value ? `${tag.name}:${tag.value}` : tag.name}
                size="small"
                onClick={() => onTagSelect(tag)}
                sx={{ cursor: 'pointer' }}
              />
            ))}
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

          <Stack direction="row" spacing={1} justifyContent="flex-end">
            <Tooltip title="Fullscreen">
              <IconButton aria-label="Fullscreen" onClick={handleFullscreen}>
                <FullscreenRoundedIcon />
              </IconButton>
            </Tooltip>
            <Tooltip title="Open in new tab">
              <IconButton
                aria-label="Open in new tab"
                component="a"
                href={inlineStreamUrl}
                target="_blank"
                rel="noopener noreferrer"
              >
                <OpenInNewRoundedIcon />
              </IconButton>
            </Tooltip>
            <Tooltip title="Download">
              <IconButton
                aria-label="Download"
                component="a"
                href={downloadUrl}
                target="_blank"
                rel="noopener noreferrer"
              >
                <DownloadRoundedIcon />
              </IconButton>
            </Tooltip>
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

function cloneAttributes(source: AttributeMap): AttributeMap {
  const entries = Object.entries(source).map(([key, values]) => [key, [...values]] as const)
  return Object.fromEntries(entries)
}

function cloneTags(source: string[]): string[] {
  return [...source]
}

function normalizeTag(value: string): string {
  return value.trim().toLowerCase()
}
