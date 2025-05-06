FROM rust@sha256:300ec56abce8cc9448ddea2172747d048ed902a3090e6b57babb2bf19f754081 AS backend_builder
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

FROM node@sha256:2b118779369d8a18f8f767511c8af05d4f80047408ec16ab0d7df2149dd58a93 AS frontend_builder
RUN mkdir -p /static
COPY assets /assets
COPY templates /assets/templates
WORKDIR /assets
RUN npm install
RUN npx tailwindcss -i css/app.css -o app.css
RUN npx esbuild --bundle js/app.ts --format=esm > app.js

FROM debian@sha256:264982ff4d18000fa74540837e2c43ca5137a53a83f8f62c7b3803c0f0bdcd56
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY --from=frontend_builder /assets/app.css /assets/app.js /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
