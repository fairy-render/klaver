
dts:
  deno run --allow-write --allow-read scripts/dts.ts



compat *args:
  cd klaver-compat; pnpm {{args}}


cross-build:
    cross build -p klaver-cli --target x86_64-unknown-linux-musl --release
