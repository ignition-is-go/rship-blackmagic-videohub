FROM rust:1.88-bookworm AS build

RUN user=root cargo new --bin rship-blackmagic-videohub
WORKDIR /rship-blackmagic-videohub

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN cargo build --release

FROM debian:bookworm-slim

COPY --from=build /rship-blackmagic-videohub/target/release/rship-blackmagic-videohub .

CMD ["./rship-blackmagic-videohub"]