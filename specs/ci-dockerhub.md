# Docker Hub Publish Workflow

Git tags pushed to GitHub now trigger `.github/workflows/dockerhub-publish.yml`, which builds the repository's production Docker image (using the root `Dockerfile`) and pushes it to Docker Hub.

## Required Secrets

Add the following repository secrets before enabling the workflow:

| Secret | Description |
| ------ | ----------- |
| `DOCKERHUB_USERNAME` | Docker Hub username or org name that owns the target repository (e.g., `galarieorg`). |
| `DOCKERHUB_TOKEN` | Docker Hub access token or password that can push to the target repository. Generate a token under the Docker Hub security settings and grant at least `write` scope. |

The workflow derives the image name as `docker.io/<DOCKERHUB_USERNAME>/galarie`. If you need a different repository name, edit `env.IMAGE_NAME` in the workflow accordingly.

## Trigger and Tags

- Workflow trigger: `git push origin <tag>`.
- All tag names are accepted; tag `v1.2.3` produces a `docker.io/<user>/galarie:v1.2.3` image.
- The Docker build explicitly targets the `prod-runtime` stage in `Dockerfile`, ensuring the published artifact matches the production runtime image.
- The workflow does **not** automatically publish `latest`. Add an extra metadata rule if that convention is desired.

## Testing the Pipeline

1. Configure the secrets described above.
2. Create a tag locally, e.g. `git tag v0.1.0`.
3. Push the tag: `git push origin v0.1.0`.
4. Monitor the workflow run in the GitHub Actions tab; upon success, the image with the tag name appears in Docker Hub.

Because the workflow executes on GitHub-hosted runners, no additional credentials or runners are required locally.
