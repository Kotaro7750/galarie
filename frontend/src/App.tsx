import { AppBar, Box, Container, CssBaseline, Stack, ThemeProvider, Toolbar, Typography, createTheme } from '@mui/material'
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
      <AppBar
        position="sticky"
        color="transparent"
        elevation={0}
        sx={{
          backdropFilter: 'blur(12px)',
          borderBottom: '1px solid',
          borderColor: 'divider',
          bgcolor: 'rgba(2, 6, 23, 0.92)',
        }}
      >
        <Container maxWidth="lg">
          <Toolbar
            disableGutters
            sx={{
              minHeight: { xs: 72, md: 82 },
              py: { xs: 1.5, md: 2 },
            }}
          >
            <Typography variant="h5" sx={{ letterSpacing: '.25em', textTransform: 'uppercase' }}>
              Galarie
            </Typography>
          </Toolbar>
        </Container>
      </AppBar>
      <Box
        sx={{
          minHeight: '100vh',
          bgcolor: 'background.default',
          py: { xs: 4, md: 8 },
          mt: { xs: 2, md: 3 },
        }}
      >
        <Container maxWidth="lg">
          <Stack spacing={4}>
            <Box component="main" id="search">
              <SearchPage apiBaseUrl={apiBaseUrl} />
            </Box>
          </Stack>
        </Container>
      </Box>
    </ThemeProvider>
  )
}
