FROM rust@sha256:f2a0f2b3529c9bbbf5479d131611451a3cc3956d9a11374d6d4ba96f059c1dce AS backend_builder
RUN mkdir -p /build/src
WORKDIR /build
COPY Cargo.lock Cargo.toml /build/
RUN echo "fn main(){}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node@sha256:ccfc02deb6abb1b70b6ef21d3d93b3f671c0de6f463ff331cf0ea0a28ad875c9 AS frontend_builder
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

FROM debian@sha256:3352c2e13876c8a5c5873ef20870e1939e73cb9a3c1aeba5e3e72172a85ce9ed
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY content /opt/blog/content
COPY templates /opt/blog/templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /assets/app.js.map /assets/app.css.gz /assets/app.js.gz /assets/app.js.map.gz /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
