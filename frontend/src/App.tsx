import { Box, Container, CssBaseline, Stack, ThemeProvider, Typography, createTheme } from '@mui/material'
import { useMemo } from 'react'

import { getAppEnvironment } from './config/env'
import { SearchPage } from './pages/SearchPage'

const theme = createTheme({
  palette: {
    mode: 'dark',
    primary: { main: '#6366f1' },
    background: {
      default: '#020617',
      paper: '#0f172a',
    },
  },
  typography: {
    fontFamily:
      "'Inter', 'Segoe UI', system-ui, -apple-system, BlinkMacSystemFont, sans-serif",
  },
  shape: { borderRadius: 18 },
})

export default function App() {
  const { apiBaseUrl } = useMemo(() => getAppEnvironment(), [])

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <Box sx={{ minHeight: '100vh', bgcolor: 'background.default', py: { xs: 6, md: 10 } }}>
        <Container maxWidth="lg">
          <Stack spacing={3}>
            <Stack spacing={1}>
              <Typography
                variant="overline"
                sx={{ letterSpacing: '.35em', color: 'primary.light', fontWeight: 600 }}
              >
                Galarie
              </Typography>
              <Typography variant="h3" component="h1" sx={{ fontWeight: 600 }}>
                Tag-based search
              </Typography>
              <Typography variant="body1" color="text.secondary" sx={{ maxWidth: 720 }}>
                Refine your library with AND-based tags and attribute filters. Thumbnails render from
                the backend on demand; upcoming tasks will wire this UI to live API data.
              </Typography>
            </Stack>

            <SearchPage apiBaseUrl={apiBaseUrl} />
          </Stack>
        </Container>
      </Box>
    </ThemeProvider>
  )
}
