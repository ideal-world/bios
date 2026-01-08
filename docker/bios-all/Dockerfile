# 用于编译可执行文件
FROM rust:1.86-alpine AS builder
WORKDIR /app

RUN apk update && \
    apk add --no-cache -U openssl-dev \
    build-base \
    openssl-libs-static &&\
    rm -rf /var/cache/apk/*

ENV OPENSSL_LIB_DIR=/usr/lib
ENV OPENSSL_INCLUDE_DIR=/usr/include
ENV OPENSSL_STATIC=1

COPY . .

RUN --mount=type=cache,id=bios_cargo_target,target=/app/target/ \
    --mount=type=cache,id=bios_cargo_registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=bios_cargo_git,target=/usr/local/cargo/git \
    cargo build --release -p bios-serv-all \
    && cp target/release/bios-serv-all /app/bios-serv-all

# RUN cargo build --release -p bios-serv-all
# RUN cp target/release/bios-serv-all /app/bios-serv-all

# 用于运行可执行文件
FROM alpine:latest
LABEL org.opencontainers.image.source=https://github.com/ideal-world/bios
# 使用最新的 Alpine 镜像
ENV TZ=Asia/Shanghai

WORKDIR /home

RUN mkdir -p /home/config


# 更新软件包，安装 tzdata，设置时区为上海，删除 tzdata 并清理缓存
RUN apk update && \
    apk add --no-cache tzdata && \
    cp /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && \
    echo "Asia/Shanghai" > /etc/timezone && \
    apk del tzdata && \
    apk add curl && \
    rm -rf /var/cache/apk/*

COPY backend/services/bios-all/config/ /home/config
COPY --from=builder /app/bios-serv-all /home/bios-serv-all

EXPOSE 8080

CMD ["./bios-serv-all"]