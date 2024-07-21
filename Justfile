


compat *args:
  cd klaver-compat; pnpm {{args}}

dts:
  deno run --allow-write --allow-read scripts/dts.ts


cross-build:
    cross build -p klaver-cli --target x86_64-unknown-linux-musl --release
