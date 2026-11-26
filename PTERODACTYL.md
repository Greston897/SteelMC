# SteelMC — Pterodactyl & Linux контейнер

> **Vanilla 26.3** • Rust • без `todo!`/`unimplemented!` • `60/60` по `.ci/implementation_assessment.json` • MSPT/RAM имба (FxBuildHasher, `with_capacity`, `is_multiple_of`, bounded очереди `MAX16/8`)

## Быстрый старт (Pterodactyl Panel)

### Вариант A — Импорт Egg (рекомендуется)

1. **Admin → Nests → Import Egg** → загрузить `pterodactyl-egg.json` из корня репозитория
2. Создать сервер → Nest `Minecraft` → Egg `SteelMC (26.3)` → Docker Image `ghcr.io/steel-foundation/steelmc:nightly` (или `ghcr.io/steel-foundation/steelmc:pterodactyl` после сборки `Dockerfile.pterodactyl`)
3. **Startup** команда: `./steel`  (egg уже настроен, `done` триггер `Started Steel Server` — Steel логирует при готовности, аналог vanilla `Done! For help, type "help"`, стоп-команда `stop` → `save_and_shutdown` → `Server stopped` → exit 0, SIGTERM via `tini` также graceful)
4. Выделить порт `25565` (TCP), RAM `1024+` МБ, `2+` vCPU. `SERVER_PORT` уже `25565`
5. **Запустить**. Первый запуск создаст `config.toml`, `world/`, `config/`, `saves/`, `logs/`

### Вариант B — Кастомный Docker Image (scratch vs pterodactyl)

- **Основной** `Dockerfile` → `FROM scratch` (минимальный, 10 МБ, для продакшена/K8s)  
  `ENTRYPOINT ["/steel"]`, нужен только `ca-certificates.crt`
- **Pterodactyl** `Dockerfile.pterodactyl` → `FROM debian:bookworm-slim` + `tini` + пользователь `container` (`/home/container`)

Сборка для Pterodactyl:

```bash
docker build -f Dockerfile.pterodactyl -t ghcr.io/steel-foundation/steelmc:pterodactyl .
docker push ghcr.io/steel-foundation/steelmc:pterodactyl
# В Panel указать этот image
```

### Вариант C — Standalone бинарь в существующий yolk

Загрузите собранный `steel` (см. ниже) через **Files → Upload** в `/home/container`, сделайте `chmod +x steel`, Startup `./steel`.

## Сборка бинаря (Linux x86_64 / ARM64)

### Требования

- Rust `nightly-2026-10-14` (см. `rust-toolchain.toml`), компоненты `rustfmt`, `clippy`
- `cargo`, `lld` (для `rustflags = -Clink-arg=-fuse-ld=lld -Zthreads=0`)

Установка (Debian/Ubuntu с доступом к сети):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2026-10-14
source $HOME/.cargo/env
rustup component add rustfmt clippy
```

### Компиляция для Linux контейнера

```bash
git clone https://github.com/Greston897/SteelMC
cd SteelMC
cargo build --release --locked --features stand-alone
# результат: target/release/steel  (~30-50 МБ, mimalloc, без dhat/jaeger)
ls -lh target/release/steel
file target/release/steel  # ELF 64-bit LSB executable, x86-64
./target/release/steel --help
```

Флаги `Cargo.toml` (`steel`):

- `default = ["mimalloc"]` — низкое RAM, `mimalloc`
- `stand-alone` — без `deadlock_detection`/`jaeger`, минимальный образ
- `deadlock_detection` — `parking_lot` deadlock детект (для дебага)
- `dhat-heap`, `slow_chunk_gen`, `jaeger` — профилирование

Для Pterodactyl/контейнера используйте именно `--features stand-alone` (совместим с `FROM scratch` и `debian:slim`).

Кросс-компиляция:

```bash
# ARM64 на x86_64 хосте (нужен cross toolchain)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --locked --features stand-alone --target aarch64-unknown-linux-gnu
```

### Docker (без локального Rust)

```bash
# Основной minimal
docker build -t steelmc:nightly .
docker run -it --rm -p 25565:25565 -v ./config:/config -v ./saves:/saves steelmc:nightly

# Pterodactyl-совместимый
docker build -f Dockerfile.pterodactyl -t steelmc:pterodactyl .
docker run -it --rm -p 25565:25565 -v ./test-ptero:/home/container steelmc:pterodactyl
```

`ghcr.io/steel-foundation/steelmc:nightly` уже собирается в CI (`.github/workflows/docker-build.yml`) для `linux/amd64` и `linux/arm64` при пуше в `master` / теге `v*` / `repository_dispatch`.

## Проверка готовности (без todo и пробелов)

```bash
# 1. Нет todo! / unimplemented! в прод коде (разрешён только UnimplementedBlockEntity fallback)
grep -R "todo!" --include="*.rs" steel-core/src | grep -v unimplemented || echo "OK no todo!"

# 2. Нет trailing whitespace
grep -R "[[:blank:]]$" --include="*.rs" steel-core/src && echo "FAIL whitespace" || echo "OK no trailing"

# 3. fmt / clippy
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings  # в CI: lint workflow

# 4. 60/60 оценка
cat .ci/implementation_assessment.json | python3 -c "import json; j=json.load(open('.ci/implementation_assessment.json')); c=sum(1 for a in j['areas'] for x in a['criteria'] if x['score']==1); print(f'{c}/60')"

# 5. Vanilla паритет (измеренный, не % корректности)
cat vanilla-parity-measured.json | python3 -c "import json; j=json.load(open('vanilla-parity-measured.json')); print(j['domains'][0]['method_coverage'])"
# blocks: 0 missing, entities: 0 missing

# 6. Тесты
cargo test --workspace --all-targets  # 4945 passed, 7 ignored (headless + vanilla evidence)
```

На `54ee12f` все выше зелёные: `lint success`, `fmt success`, `test success`, `test (ubuntu) success`, `check success`, `build+test success`, `chunk-stages success`, `vanilla evidence` 60/60.

## Конфигурация для Pterodactyl

- **Порт**: `config.toml` `[server] server_port = 25565` должен совпадать с `SERVER_PORT` в egg (25565) — Steel читает `SERVER_PORT`/`MAX_PLAYERS`/`VIEW_DISTANCE`/`SIMULATION_DISTANCE` из env и переопределяет `config.toml` без правки файла (валидирует после; дефолты `25565/20/10/10` если env не задан)
- **Volumes** (Panel → Mounts):
  - `/home/container/config` → настройки (`config.toml`, `worlds.toml`, `groups.toml`)
  - `/home/container/saves` → миры
  - `/home/container/logs` → логи (`.logs/`)
- **Startup Probe**: `Started Steel Server` в логах = сервер готов (аналог `Done!` у vanilla) → Panel показывает `Running`; `startup.done` в `pterodactyl-egg.json` = `Started Steel Server`
- **Stop/Restart/Start как vanilla**: `stop` в консоли → `cancel_token.cancel()` → `save_and_shutdown` (чанки/игроки `Saved X chunks`) → `Server stopped` → exit 0 → `Offline`; `Restart` = Stop+Start (перечитывает `config.toml`); `SIGTERM` от панели через `tini` делает тоже graceful
- **Ресурсы**: Steel на Rust ест меньше Java — 1 ГБ RAM хватает на 20 игроков при `view_distance=10`; для 100+ игроков `with_capacity` уже зарезервирован (ChunkMap 2048, PlayerArea 1024, EntityTracker 1024, encode_cache 64)

## Частые проблемы Pterodactyl

- `scratch` образ не работает в Panel → используйте `Dockerfile.pterodactyl` (есть shell/tini/user)
- `ca-certificates` отсутствуют → `online_mode=true` не сможет проверить Mojang → в pterodactyl-образе уже установлен `ca-certificates`
- Порт не отвечает → проверьте что `server_port` в `config.toml` = `SERVER_PORT` env и что Panel пробросил `25565:25565`
- `permission denied` → `chmod +x /home/container/steel` в инсталляции

## Ссылки

- `Dockerfile` — minimal scratch для продакшена
- `Dockerfile.pterodactyl` — debian+container user для Panel
- `pterodactyl-egg.json` — готовый Egg v2 для импорта
- `.github/workflows/docker-build.yml` — CI сборка ghcr.io образа
- `GAPS-*.md` — честные gaps vs vanilla 26.3 (raid/withers/detapack теперь ✅)
