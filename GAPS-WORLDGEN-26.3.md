# Gap: Worldgen Features — сравнение с vanilla 26.3

## Статус: частично реализовано

`steel-worldgen/` содержит генерацию мира: биомы, климат-семплеры, шум (Perlin, Simplex, Improved Noise, Blended Noise), плотность, поверхность, state resolver, структуры (дома, форты, храмы, особняки, пирамиды,	end-городы, шахты, fuertes, океанские руины, цитадели, бараки, мусорники, оазисы),	jigsaw assembly, aleatory placement. Требуется метод-к-методу сравнение с vanilla sources (`minecraft-src/`, commit `74720c28`) — в текущем沙боксе исходники не скачаны, точный список 7 расхождений не может быть выведен здесь.

| Домен | Steel реализация | Vanilla (minecraft-src) | Статус |
|---|---|---|---|
| Биомы | `biome_source.rs`, `climate_sampler.rs` | `WorldGenBiome`, `BiomeSource` | ⚠️ требует сверки |
| Климат | `climate_sampler.rs`, `nether_climate_sampler.rs` | `ClimateSampler`, `NoiseClimateSampler` | ⚠️ требует сверки |
| Шум (overworld) | `perlin_noise.rs`, `simplex_noise.rs`, `blended_noise.rs` | `PerlinNoise`, `SimplexNoise`, `BlendedNoise` | ⚠️ требует сверки |
| Шум (nether/end) | `nether_climate_sampler.rs`, `end_islands.rs` | `NetherClimateSampler`, `EndIslandNoise` | ⚠️ требует сверки |
| Плотность | `density/mod.rs`, `spline_eval.rs`, `traits.rs` | `DensityFunction`, `SplineEvaluator` | ⚠️ требует сверки |
| Поверхность | `surface.rs`, `surface_partial.rs` | `SurfaceBuilder`, `SurfaceRules` | ⚠️ требует сверки |
| State resolver | `state_resolver.rs` | `BlockStateProvider`, `StateResolver` | ⚠️ требует сверки |
| Структуры | `structure/` (20+ файлов) | `Structure`, `StructurePiece`, `JigsawStructure` | ⚠️ требует сверки |
| Jigsaw | `jigsaw.rs`, `jigsaw_assembly.rs` (bench) | `JigsawPlacement`, `JigsawPattern` | ⚠️ требует сверки |
| Рудные жилы | `ore_veinifier.rs` | `OreVegetation` / `OreFeature` | ⚠️ требует сверки |
| Акваферы | `aquifer.rs` | `Aquifer` | ⚠️ требует сверки |
| Chunk stages | `steel-core/src/chunk/` — chunk status pipeline | `ChunkStatus`, `ChunkGenerator` | ❌ Flint worldgen не зелёные |

## Что требуется

1. **Скачать `minecraft-src/`** (commit `74720c28`, см. `update-minecraft-src.sh` через GitCraft)
2. **Метод-к-методу сравнение** классов генерации чанков vanilla vs Steel
3. **Выявить точные 7 расхождений** features Overworld
4. **Пройти полный external эталон** chunk-stages / Flint worldgen тесты
5. **Исправить расхождения по одному**

## Связанные файлы

- `steel-worldgen/src/` — реализация генерации мира (биомы, шум, плотность, поверхность, структуры)
- `steel-core/src/chunk/` — чанки, статус чанков, генерация чанков
- `steel-core/src/chunk/chunk_generation_task.rs` — задача генерации чанка