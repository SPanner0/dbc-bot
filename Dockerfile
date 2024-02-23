FROM messense/rust-musl-cross:x86_64-musl AS chef
ENV DISCORD_TOKEN = ${DISCORD_TOKEN}
ENV BRAWL_STARS_TOKEN = ${BRAWL_STARS_TOKEN}
ENV DATABASE_URL = ${DATABASE_URL}
RUN cargo install cargo-chef
WORKDIR /dbc-bot

RUN \
  apt-get update && \
  apt-get install -y ca-certificates && \
  export OPENSSL_DIR="/usr/lib/openssl" && \
  apt-get install -y ca-certificates && \
  apt-get clean

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /dbc-bot/recipe.json recipe.json
# Build cache dependencies
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json

# Copy assets and presets
COPY . .
RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target x86_64-unknown-linux-musl

# Build python image
FROM python:3
COPY requirements.txt ./
COPY src/bracket_tournament/bracket_generation.py ./src/bracket_tournament/bracket_generation.py
RUN pip install --no-cache-dir -r requirements.txt

# Create a minimal image
FROM scratch
COPY --from=builder /dbc-bot/target/x86_64-unknown-linux-musl/release/dbc-bot /dbc-bot
COPY assets ./
ENTRYPOINT ["/dbc-bot"]




