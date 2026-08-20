FROM rust:1.97.1-trixie@sha256:b1b3c9c0d921d7fa0a6d1f9ec7e4eab87f8c8ec97644c3d791450f131dec813f AS backend_builder
RUN mkdir -p /build/src
WORKDIR /build
COPY Cargo.lock Cargo.toml /build/
RUN echo "fn main(){}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node:lts-trixie@sha256:83bd9709839251476a4caa7b5a7139d5ca372affcd35eccac688b04aa0e93667 AS frontend_builder
RUN mkdir -p /assets
COPY pnpm-lock.yaml pnpm-workspace.yaml package.json /assets/
COPY css /assets/css
COPY js /assets/js
COPY templates /assets/templates
WORKDIR /assets
RUN npm install -g pnpm@11.5.3
RUN pnpm install
RUN npx tailwindcss --minify --input css/app.css --output app.css
RUN npx esbuild --sourcemap --minify --bundle --format=esm --outdir=/assets js/app.ts
RUN gzip -k9 app.css app.js app.js.map

FROM debian:trixie-slim@sha256:28de0877c2189802884ccd20f15ee41c203573bd87bb6b883f5f46362d24c5c2
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY content /opt/blog/content
COPY templates /opt/blog/templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /assets/app.js.map /assets/app.css.gz /assets/app.js.gz /assets/app.js.map.gz /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
