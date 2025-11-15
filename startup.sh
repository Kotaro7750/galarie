#!/bin/bash

cat >/app/frontend/runtime-env.json <<EOF
{
  "apiBaseUrl": "${GALARIE_BACKEND_API_BASE:-http://backend/api/v1}"
}
EOF

/app/galarie-backend
