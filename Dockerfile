# 1. This tells docker to use the Rust official image
FROM rust:1.79.0 as builder
WORKDIR /usr/src/myapp

# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# 3. install build time dep for opus
RUN apt update && apt -y install cmake
 
# Build your program for release
RUN cargo install --path . 

# Prod stage, bookworm include libssl.so.3
FROM debian:bookworm-slim as runner
ARG DISCORD_TOKEN
ENV DISCORD_TOKEN=${DISCORD_TOKEN}

# py3 38.3MB, pip 370.88MB, ytdlp 14.5MB
RUN apt update && apt -y install python3 curl
# run time dep 1: yt-dlp
RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && chmod a+rx /usr/local/bin/yt-dlp

# run time dep 2: opus
RUN apt -y install libopus-dev

RUN rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/rusty-music-bot /usr/local/bin/rusty-music-bot

# Run the binary
CMD ["rusty-music-bot"]
