
cross-build:
    cross build -p klaver-cli --target x86_64-unknown-linux-musl --features cross --release


build-aarch64:
    cross build -p klaver-cli --target aarch64-unknown-linux-gnu --features cross --release