
build a single arch image

```sh
docker buildx build -t pact-foundation/pact:$DOCKER_TAG-alpine --build-arg VERSION=$DOCKER_TAG --platform linux/arm . -f Dockerfile.alpine --load
```

Run the image

```sh
docker run --platform=linux/arm -p 8080:8080 --rm --init pact-foundation/pact:0.0.9-alpine mock start
```

Docker multi arch available args

```console
BUILDPLATFORM — matches the current machine. (e.g. linux/amd64)

BUILDOS — os component of BUILDPLATFORM, e.g. linux

BUILDARCH — e.g. amd64, arm64, riscv64

BUILDVARIANT — used to set ARM variant, e.g. v7

TARGETPLATFORM — The value set with --platform flag on build

TARGETOS - OS component from --platform, e.g. linux

TARGETARCH - Architecture from --platform, e.g. arm64

TARGETVARIANT - Variant from the --platform e.g. v7
```

## Docker targets

### Alpine

<https://hub.docker.com/_/alpine/tags>

## Rust Platforms

### rust platform support

- <https://doc.rust-lang.org/rustc/platform-support.html>
