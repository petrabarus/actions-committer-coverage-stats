# Dockerfile for building the application.
# See: https://docs.docker.com/language/rust/develop/

# Create a stage for building the application.
ARG RUST_VERSION=1.70.0
ARG APP_NAME=github_action_committer_coverage_stats

FROM rust:${RUST_VERSION}-slim-bullseye AS build
ARG APP_NAME

WORKDIR /app

# install the required dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    libgit2-dev \
    && rm -rf /var/lib/apt/lists/*

# Leverage a cache mount to /usr/local/cargo/registry/
# for downloaded dependencies and a cache mount to /app/target/ for 
# compiled dependencies which will speed up subsequent builds.
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    <<EOF
set -e
cargo build --release
cp ./target/release/$APP_NAME /bin/app
EOF

# Create a stage for running the application.
FROM debian:bullseye-slim AS final


# Create a non-privileged user that the app will run under.
# See https://docs.docker.com/develop/develop-images/dockerfile_best-practices/   #user
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# Copy the executable from the "build" stage.
COPY --from=build /bin/app /bin/app

ENV RUST_LOG=info
ENV RUST_BACKTRACE=full

CMD [ "app" ]
