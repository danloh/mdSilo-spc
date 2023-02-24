FROM rust:alpine as backend
WORKDIR /home/rust/src
RUN apk --no-cache add musl-dev openssl-dev
COPY . .
RUN cargo test --release
RUN cargo build --release

FROM scratch
COPY --from=backend /home/rust/src/target/release/spc-lite .
USER 1000:1000
CMD [ "./spc-lite" ]
EXPOSE 8080