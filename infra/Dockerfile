FROM rust:1-bookworm AS wasm-interp-builder

RUN curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY wasm ./wasm
COPY interpreter ./interpreter
COPY repl ./repl
RUN wasm-pack build wasm \
  --target web \
  --out-dir ../app/src/lib/abclang \
  --out-name abclang \
  --release

FROM node:22-slim AS fe-builder
RUN corepack enable
WORKDIR /app
COPY app/package.json app/pnpm-lock.yaml ./
RUN pnpm install --frozen-lockfile
COPY app ./
COPY --from=wasm-interp-builder /build/app/src/lib/abclang ./src/lib/abclang
RUN pnpm build

FROM nginx:alpine AS final-runtime
COPY --from=fe-builder /app/dist /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
