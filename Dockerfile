FROM rust:1.78.0 AS backend_builder
RUN mkdir -p /build
COPY Cargo.lock Cargo.toml /build/
COPY src /build/src
WORKDIR /build
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/target \
    cargo build --release
RUN --mount=type=cache,target=/build/target cp /build/target/release/blog /build/blog
RUN useradd -u 5000 scratch

FROM node:alpine as frontend_builder
RUN mkdir -p /static
COPY assets /assets
COPY templates /templates
WORKDIR /assets
RUN npm ci
RUN npx tailwindcss -i css/app.css -o app.css
RUN npx esbuild --bundle js/app.ts --external:highlight.js --external:htmx.org --format=esm > app.js

FROM scratch

COPY --from=backend_builder /lib/x86_64-linux-gnu/libgcc_s.so.1 /lib/x86_64-linux-gnu/libgcc_s.so.1
COPY --from=backend_builder /lib/x86_64-linux-gnu/libm.so.6 /lib/x86_64-linux-gnu/libm.so.6
COPY --from=backend_builder /lib/x86_64-linux-gnu/libc.so.6 /lib/x86_64-linux-gnu/libc.so.6
COPY --from=backend_builder /lib64/ld-linux-x86-64.so.2 /lib64/ld-linux-x86-64.so.2
COPY --from=backend_builder /etc/passwd /etc/passwd
COPY --from=backend_builder /build/blog /

COPY static /static
COPY templates /templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /static/

USER 5000

CMD ["/blog"]