FROM rust@sha256:9c1ef35ab804dc78361948794f60748e79a7a2e297580604b288590bc52ebdaa AS backend_builder
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

FROM node@sha256:bac8ff0b5302b06924a5e288fb4ceecef9c8bb0bb92515985d2efdc3a2447052 AS frontend_builder
RUN mkdir -p /static
COPY assets /assets
COPY templates /assets/templates
WORKDIR /assets
RUN npm install
RUN npx tailwindcss -i css/app.css -o app.css
RUN npx esbuild --bundle js/app.ts --format=esm > app.js

FROM debian@sha256:1209d8fd77def86ceb6663deef7956481cc6c14a25e1e64daec12c0ceffcc19d
RUN mkdir -p /opt/blog
WORKDIR /opt/blog
COPY --from=backend_builder /build/blog /opt/blog/
COPY static /opt/blog/static
COPY --from=frontend_builder /assets/app.css /assets/app.js /opt/blog/static/
USER 1000
ENTRYPOINT ["/opt/blog/blog"]
