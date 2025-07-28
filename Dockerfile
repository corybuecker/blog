FROM rust@sha256:5771a3cc2081935c59ac52b92d49c9e164d4fed92c9f6420aa8cc50364aead6e AS backend_builder
RUN mkdir -p /build/src
WORKDIR /build
COPY Cargo.lock Cargo.toml /build/
RUN echo "fn main(){}" > /build/src/main.rs
RUN cargo build --release
COPY src /build/src
RUN touch /build/src/main.rs
RUN cargo build --release
RUN cp /build/target/release/blog /build/blog

FROM node@sha256:c7a63f857d6dc9b3780ceb1874544cc22f3e399333c82de2a46de0049e841729 AS frontend_builder
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

FROM debian@sha256:d42b86d7e24d78a33edcf1ef4f65a20e34acb1e1abd53cabc3f7cdf769fc4082
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY content /opt/blog/content
COPY templates /opt/blog/templates
COPY --from=frontend_builder /assets/app.css /assets/app.js /assets/app.js.map /assets/app.css.gz /assets/app.js.gz /assets/app.js.map.gz /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
