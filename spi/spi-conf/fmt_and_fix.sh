git add .
cargo fmt -p bios-spi-conf
git add .
cargo clippy -p bios-spi-conf --fix --allow-staged
git add .
cargo fmt -p bios-spi-conf
git add .