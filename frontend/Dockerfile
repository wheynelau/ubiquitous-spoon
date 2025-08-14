FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

# Install `dx` early to cache this layer
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .


# Create the final bundle folder. Bundle always executes in release mode with optimizations enabled
RUN dx bundle --platform web --release --out-dir web

FROM nginx:alpine AS runtime
COPY --from=builder /app/web/public /usr/share/nginx/html
EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
