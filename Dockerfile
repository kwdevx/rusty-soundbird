# 1. This tells docker to use the Rust official image
FROM rust:1.79.0 as builder
WORKDIR /usr/src/myapp

# 2. Copy the files in your machine to the Docker image
COPY ./ ./

# 3. install cmake for build time 
RUN apt update
RUN apt -y install cmake

# build dep to run in alpine
RUN apt -y install musl

# Build your program for release
# RUN cargo build --release 
RUN cargo install --path . 

# Prod stage
FROM debian as runner
ARG DISCORD_TOKEN

RUN apt update
# run time dep 1
RUN apt -y install libopus-dev
RUN apt -y install curl 
# run time dep 2
RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp
RUN chmod a+rx /usr/local/bin/yt-dlp
# to run yt-dlp
RUN apt -y install python3 
RUN rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/rs-bot /usr/local/bin/rs-bot

# Run the binary
CMD ["rs-bot"]
