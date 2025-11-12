import {
  Box,
  Card,
  CardContent,
  Container,
  CssBaseline,
  Divider,
  Link,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Stack,
  ThemeProvider,
  Typography,
  createTheme,
} from '@mui/material'
import CheckCircleRoundedIcon from '@mui/icons-material/CheckCircleRounded'
import LaunchRoundedIcon from '@mui/icons-material/LaunchRounded'
import { useMemo } from 'react'

import { getAppEnvironment } from './config/env'

const setupSteps = [
  'Run `make frontend/install` in the devcontainer to install dependencies once.',
  'Copy `.env.example` → `.env.local` and adjust `VITE_API_BASE_URL` if the backend runs elsewhere.',
  'Start the backend (`cargo run …`) and this client (`npm run dev`) inside devcontainer terminals.',
  'Add search/favorites/video pages under `src/pages/` and shared hooks/services alongside them.',
]

const usefulLinks = [
  {
    name: 'Quickstart',
    description: 'Devcontainer, media fixtures, telemetry stack.',
    href: '/specs/galarie-media-platform/quickstart.md',
  },
  {
    name: 'API Spec',
    description: 'Contracts for /api/v1 endpoints.',
    href: '/specs/galarie-media-platform/contracts/openapi.yaml',
  },
  {
    name: 'Tasks',
    description: 'Implementation backlog and gating rules.',
    href: '/specs/galarie-media-platform/tasks.md',
  },
]

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
      <Box sx={{ minHeight: '100vh', bgcolor: 'background.default', py: { xs: 8, md: 12 } }}>
        <Container maxWidth="lg">
          <Stack spacing={4}>
            <header>
              <Typography
                variant="overline"
                sx={{ letterSpacing: '.35em', color: 'primary.light', fontWeight: 600 }}
              >
                Galarie
              </Typography>
              <Typography variant="h3" component="h1" sx={{ mt: 1, fontWeight: 600 }}>
                Frontend scaffold ready.
              </Typography>
              <Typography variant="body1" color="text.secondary" sx={{ mt: 2, maxWidth: 720 }}>
                This project ships with Vite, React 19, Vitest, ESLint, and Material UI so you can
                focus on the search/favorites/video flows from the backlog. Drop feature pages under{' '}
                <code>src/pages</code> and keep reusable logic in <code>src/hooks</code> or{' '}
                <code>src/services</code>.
              </Typography>
            </header>

            <Card variant="outlined">
              <CardContent>
                <Stack
                  spacing={3}
                  direction={{ xs: 'column', md: 'row' }}
                  alignItems={{ xs: 'flex-start', md: 'center' }}
                >
                  <Box flex={1}>
                    <Typography variant="overline" color="text.secondary">
                      API base URL
                    </Typography>
                    <Typography variant="h5" sx={{ mt: 1, fontWeight: 600 }}>
                      {apiBaseUrl}
                    </Typography>
                  </Box>
                  <Box maxWidth={{ xs: '100%', md: 320 }}>
                    <Typography variant="body2" color="text.secondary">
                      Update <code>VITE_API_BASE_URL</code> in <code>.env.local</code> when tunneling
                      or deploying the backend elsewhere.
                    </Typography>
                  </Box>
                </Stack>
              </CardContent>
            </Card>

            <Card variant="outlined">
              <CardContent>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  Next setup steps
                </Typography>
                <List dense sx={{ mt: 1 }}>
                  {setupSteps.map((step) => (
                    <ListItem key={step} disableGutters>
                      <ListItemIcon sx={{ minWidth: 36 }}>
                        <CheckCircleRoundedIcon color="primary" fontSize="small" />
                      </ListItemIcon>
                      <ListItemText primary={step} />
                    </ListItem>
                  ))}
                </List>
              </CardContent>
            </Card>

            <Card variant="outlined">
              <CardContent>
                <Typography variant="h6" sx={{ fontWeight: 600 }}>
                  Project references
                </Typography>
                <Stack
                  spacing={2}
                  sx={{ mt: 1 }}
                  direction={{ xs: 'column', md: 'row' }}
                  divider={<Box sx={{ display: { xs: 'block', md: 'none' } }} />}
                >
                  {usefulLinks.map((link) => (
                    <Box key={link.name} flex={1}>
                      <Stack spacing={1}>
                        <Typography variant="subtitle2" color="text.secondary">
                          {link.name}
                        </Typography>
                        <Typography variant="body2">{link.description}</Typography>
                        <Link
                          href={link.href}
                          underline="hover"
                          target="_blank"
                          rel="noreferrer"
                          sx={{ display: 'inline-flex', alignItems: 'center', gap: 0.5 }}
                        >
                          Open
                          <LaunchRoundedIcon fontSize="small" />
                        </Link>
                      </Stack>
                    </Box>
                  ))}
                </Stack>
              </CardContent>
            </Card>

            <Divider light />
            <Typography variant="caption" color="text.disabled">
              When ready, replace this helper shell with the actual search UI; keep env utilities and
              testing scaffold for future components.
            </Typography>
          </Stack>
        </Container>
      </Box>
    </ThemeProvider>
  )
}
