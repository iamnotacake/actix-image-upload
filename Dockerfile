FROM rust:1.44-buster as builder

RUN apt update && apt upgrade -y && apt install -y libclang-dev clang libopencv-dev

WORKDIR /usr/src/actix-image-upload
COPY Cargo.toml Cargo.lock ./
COPY src ./src/
RUN sed -i 's/"opencv-4"/"opencv-32"/' Cargo.toml
RUN cargo install --path .

FROM debian:buster
RUN apt update && apt upgrade -y && apt install -y libopencv-imgcodecs3.2
COPY --from=builder /usr/local/cargo/bin/actix_image_upload /usr/local/bin/actix_image_upload
ENV RUST_LOG="actix_image_upload=debug"
CMD ["actix_image_upload"]
