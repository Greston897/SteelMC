#!/bin/bash
set -euo pipefail
# Сборка SteelMC для Linux контейнера и Pterodactyl Panel
# Требует Rust nightly-2026-10-14, как в rust-toolchain.toml
# Результат: target/release/steel (ELF x86_64, mimalloc, stand-alone)

echo "=== SteelMC 26.3 — сборка для Linux / Pterodactyl ==="
echo "Проверка окружения..."
if ! command -v cargo >/dev/null 2>&1; then
  echo "ERROR: cargo не найден. Установите Rust:"
  echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2026-10-14"
  echo "  rustup component add rustfmt clippy"
  exit 1
fi
cargo --version
rustc --version

echo ""
echo "1/4 cargo fmt --check (пробелы)..."
cargo fmt --all -- --check || (echo "fmt failed, запустите cargo fmt --all"; exit 1)
echo "OK fmt"

echo ""
echo "2/4 cargo clippy (без todo, -D warnings)..."
cargo clippy --workspace --all-targets -- -D warnings || (echo "clippy failed"; exit 1)
echo "OK clippy"

echo ""
echo "3/4 cargo test (проверка без недоработок)..."
# quick check — large_holes и a_huenchik должны пройти
cargo test --workspace --all-targets --quiet -- --nocapture 2>&1 | tail -n 20
echo "OK test"

echo ""
echo "4/4 cargo build --release --locked --features stand-alone (Linux контейнер)..."
cargo build --release --locked --features stand-alone
ls -lh target/release/steel
file target/release/steel

echo ""
echo "=== Сборка завершена ==="
echo "Бинарь: target/release/steel"
echo "Запуск: ./target/release/steel"
echo "Docker: docker build -t steelmc:nightly . && docker run -p 25565:25565 -v ./config:/config -v ./saves:/saves steelmc:nightly"
echo "Pterodactyl:"
echo "  - Dockerfile.pterodactyl -> docker build -f Dockerfile.pterodactyl -t steelmc:pterodactyl ."
echo "  - Egg: pterodactyl-egg.json -> Panel > Nests > Import Egg"
echo "  - Или загрузите target/release/steel в /home/container/steel в Panel"
echo ""
echo "Проверка vanilla 26.3:"
echo "  cat .ci/implementation_assessment.json | python3 -c \"import json; j=json.load(open('.ci/implementation_assessment.json')); print(sum(1 for a in j['areas'] for c in a['criteria'] if c['score']==1), '/60')\""
echo "  grep -R \"todo!\" --include=\"*.rs\" steel-core/src | grep -v unimplemented || echo \"OK no todo!\""
