## Docker image

Container images are built and pushed using the `docker/build` script, which handles both Alpine and Debian variants for Docker Hub (`pactfoundation`) and GHCR (`ghcr.io/pact-foundation`).

### Environment variables

| Variable           | Description                                                          | Default                          |
| ------------------ | -------------------------------------------------------------------- | -------------------------------- |
| `CONTAINER_TAG`    | Version to build (e.g. `0.8.1`, without the `v` prefix). **Required.** |                                  |
| `PUSH_IMAGE`       | Set to `true` to push images to registries.                         | `false`                          |
| `TAG_LATEST`       | Set to `true` to also tag the base image as `latest`.               | `false`                          |
| `PLATFORMS`        | Comma-separated target platforms for all builds.                    | `linux/amd64,linux/arm64`        |
| `PLATFORMS_ALPINE` | Target platforms for Alpine builds. Overrides `PLATFORMS`.          | inherits `PLATFORMS`             |
| `PLATFORMS_DEBIAN` | Target platforms for Debian builds. Overrides `PLATFORMS`.          | inherits `PLATFORMS`             |

### Usage

Build locally (no push):

```shell
CONTAINER_TAG=0.8.1 docker/build
```

Build and push, also tagging as `latest`:

```shell
CONTAINER_TAG=0.8.1 PUSH_IMAGE=true TAG_LATEST=true docker/build
```

The script creates a `multiarch` buildx builder automatically and builds three tag variants per registry:
- `<version>` (Alpine, same as `<version>-alpine`)
- `<version>-alpine`
- `<version>-debian`
