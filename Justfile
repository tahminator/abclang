build-wasm:
    wasm-pack build wasm \
      --target web \
      --out-dir ../app/src/lib/abclang \
      --out-name abclang \
      --release
    @echo "wasm bindings written to app/src/lib/abclang"

dev:
    just build-wasm && cd app && pnpm run dev

test:
    cargo test && pnpm run --dir=app test
