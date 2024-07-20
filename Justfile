


compat *args:
  cd klaver-compat; pnpm {{args}}

dts:
  deno run --allow-write --allow-read scripts/dts.ts
