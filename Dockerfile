FROM rust:1.92-alpine3.23 AS build

WORKDIR /app

COPY Cargo.* .
COPY src/ ./src/

RUN apk add --no-cache openssl-dev openssl-libs-static
RUN cargo build --release

FROM alpine:3.23

WORKDIR /

COPY --from=build /app/target/release/gitignore .

ENTRYPOINT [ "/gitignore" ]
