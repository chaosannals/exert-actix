# 编译
FROM rust:1.49 AS builder
WORKDIR /usr/src/exert
COPY . .
RUN cargo install --path ./server

# 运行
FROM rust:1.49-slim-buster
WORKDIR /app
RUN apt-get update \
    && apt-get install -y inetutils-ping \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/exert-actix-server /usr/local/bin/exert-actix-server
CMD ["exert-actix-server"]
