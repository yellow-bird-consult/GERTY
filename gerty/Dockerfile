FROM rust:1.55 as builder

# RUN apt-get update -yqq && apt-get install -yqq cmake g++

COPY ./src ./src
COPY ./clean_up.sh ./clean_up.sh
COPY ./Cargo.toml ./Cargo.toml

WORKDIR .

RUN cargo build --release

FROM debian:buster-slim

COPY --from=builder ./target/release/gerty .
COPY --from=builder ./clean_up.sh . 
RUN chmod +x ./gerty

EXPOSE 3000

CMD ["./gerty"]

