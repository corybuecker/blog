FROM rust:1.94.1-trixie@sha256:652612f07bfbbdfa3af34761c1e435094c00dde4a98036132fca28c7bb2b165c AS backend_builder
RUN mkdir -p /build/src
WORKDIR /build
COPY Cargo.lock Cargo.toml /build/
RUN echo "fn main(){}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node:lts-trixie@sha256:e4ceb04a1f1dd4823a1ab6ef8d2182c09d6299b507c70f20bd0eb9921a78354d AS frontend_builder
RUN mkdir -p /assets
COPY pnpm-lock.yaml pnpm-workspace.yaml package.json /assets/
COPY css /assets/css
COPY js /assets/js
COPY templates /assets/templates
WORKDIR /assets
RUN npm install -g pnpm@10.33.0
RUN pnpm install
RUN npx tailwindcss --minify --input css/app.css --output app.css
RUN npx esbuild --sourcemap --minify --bundle --format=esm --outdir=/assets js/app.ts
RUN gzip -k9 app.css app.js app.js.map

FROM debian:trixie-slim@sha256:4ffb3a1511099754cddc70eb1b12e50ffdb67619aa0ab6c13fcd800a78ef7c7a
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY content /opt/blog/content
COPY templates /opt/blog/templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /assets/app.js.map /assets/app.css.gz /assets/app.js.gz /assets/app.js.map.gz /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
