FROM rust:1.84.0-slim AS backend_builder
RUN mkdir -p /build
COPY Cargo.lock Cargo.toml /build/
COPY src /build/src
WORKDIR /build
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node:alpine AS frontend_builder
RUN mkdir -p /static
COPY assets /assets
COPY templates /templates
WORKDIR /assets
RUN npm ci
RUN npx tailwindcss -i css/app.css -o app.css
RUN npx esbuild --bundle js/app.ts --external:highlight.js --external:htmx.org --format=esm > app.js

FROM rust:1.84.0-slim
COPY --from=backend_builder /build/blog /
COPY static /static
COPY templates /templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /static/
USER 1000
CMD ["/blog"]