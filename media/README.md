# Media Samples

This directory hosts local media fixtures that mirror the tagging scheme used by Galarie. Files are **not** checked into git (see `.gitignore`) so you can mount your personal library without risking accidental commits. Use the curated sample set under `sample-media/` (tracked in git) as a quick starting point.

## Sample Naming Conventions

```
<tag-1>_<tag-2>_..._<tag-n><extension>
```

- Plain tags are lowercase words (`sunset`, `macro`).
- Attribute tags encode `key=value` using a colon substitute (e.g., `rating-5` or `camera-fujifilm`) that remains filesystem-safe.
- Separate tags with underscores. Order is arbitrary but consistent ordering simplifies diffs.

## Sample Set

Copy the pre-generated fixtures into this directory:

```bash
cp sample-media/* media/
```

The sample set contains:

| File | Media Type | Tags Encoded |
|------|------------|--------------|
| `sunset_coast+location-okinawa_rating-5.png` | Photo (320×200 PNG) | sunset, coast, location=okinawa, rating=5 |
| `macro_leaf+subject-nature_rating-4.gif` | Animated GIF (test pattern) | macro, leaf, subject=nature, rating=4 |
| `skate_session+type-video_rating-3.mp4` | 2s MP4 clip with tone | skate, session, type=video, rating=3 |

Feel free to drop additional personal media next to these fixtures. Everything under `media/` stays untracked by git.
