FROM rust@sha256:251cec8da4689d180f124ef00024c2f83f79d9bf984e43c180a598119e326b84 AS backend_builder
RUN mkdir -p /build/src
WORKDIR /build
COPY Cargo.lock Cargo.toml /build/
RUN echo "fn main(){}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
COPY templates /build/templates
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node@sha256:d1db2ecd11f417ab2ff4fef891b4d27194c367d101f9b9cd546a26e424e93d31 AS frontend_builder
RUN mkdir -p /static
COPY assets /assets
COPY templates /assets/templates
WORKDIR /assets
RUN npm install
RUN npx tailwindcss -i css/app.css -o app.css
RUN npx esbuild --bundle js/app.ts --format=esm > app.js

FROM debian@sha256:0d8498a0e9e6a60011df39aab78534cfe940785e7c59d19dfae1eb53ea59babe
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY --from=frontend_builder /assets/app.css /assets/app.js /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
