# Sample Media Fixtures

This directory stores small, redistributable assets that follow Galarie's tag naming convention. Copy them into `media/` when you need a predictable dataset for local development, CI, or demos:

```bash
cp sample-media/* media/
```

## Included Assets

| File | Type | Notes |
|------|------|-------|
| `sunset_coast+location-okinawa_rating-5.png` | 320×200 PNG | Flat goldenrod gradient for thumbnail tests |
| `macro_leaf+subject-nature_rating-4.gif` | Animated GIF | 240×180 color bars with 2s loop |
| `skate_session+type-video_rating-3.mp4` | 2s MP4 + tone | 640×360 cyan frame + 880Hz sine for streaming tests |

The files were generated via `ffmpeg` inside the devcontainer so they remain platform-neutral and royalty-free.
