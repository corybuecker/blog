FROM rust@sha256:bed2d7f8140d73c26f16c298c91ae8487a09f40d3840c0d8d139537e1b51e148 AS backend_builder
RUN mkdir -p /build/src
WORKDIR /build
COPY Cargo.lock Cargo.toml /build/
RUN echo "fn main(){}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node@sha256:c7afcee8493fc98aaa1fb5c5b9ed396ed98ffe596921feba4cae0c6a63c8cfeb AS frontend_builder
RUN mkdir -p /assets
COPY package.json package-lock.json /assets/
COPY css /assets/css
COPY js /assets/js
COPY templates /assets/templates
WORKDIR /assets
RUN npm install
RUN npx tailwindcss --minify --input css/app.css --output app.css
RUN npx esbuild --sourcemap --minify --bundle --format=esm --outdir=/assets js/app.ts
RUN gzip -k9 app.css app.js app.js.map

FROM debian@sha256:5cf544fad978371b3df255b61e209b373583cb88b733475c86e49faa15ac2104
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY content /opt/blog/content
COPY templates /opt/blog/templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /assets/app.js.map /assets/app.css.gz /assets/app.js.gz /assets/app.js.map.gz /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
