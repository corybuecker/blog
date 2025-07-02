FROM rust@sha256:1928f85f204effc91fddc53875afd042b651552fde6ee11acaafde641942dd70 AS backend_builder
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

FROM node@sha256:8369522c586f6cafcf77e44630e7036e4972933892f8b45e42d9baeb012d521c AS frontend_builder
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
