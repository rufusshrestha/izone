# syntax=docker/dockerfile:1
# iZone web UI. Build and run with: docker compose up -d --build

FROM rust:1-bookworm AS build
WORKDIR /src
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --system --uid 10001 --home-dir /data --shell /usr/sbin/nologin izone \
 && mkdir -p /data && chown izone:izone /data && chmod 700 /data
COPY --from=build /src/target/release/izone /usr/local/bin/izone
USER izone
# The changed web password (encrypted) and its key live in the /data volume.
ENV HOME=/data XDG_CONFIG_HOME=/data/config XDG_DATA_HOME=/data/share
VOLUME /data
EXPOSE 8080
ENTRYPOINT ["izone"]
CMD ["web", "--host", "0.0.0.0", "--port", "8080"]
