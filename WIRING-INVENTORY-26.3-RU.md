# Инвентаризация подключения исходников

**Не доказательство поведенческого паритета.** Список охватывает входы генератора объектов и общие AI goals, а не каждую механику vanilla. Аннотация, импорт и ссылка не означают, что поведение исполняется правильно. Отсутствие точной аннотации может означать наследование через генератор; это не автоматически отсутствующая механика.

Воспроизведение: `python3 .ci/wiring_inventory.py`.

## Общие AI goals

| Goal | Реализация | Производственные ссылки (без собственного файла и тестов) |
|---|---|---|
| AvoidEntityGoal | `steel-core/src/entity/ai/goal/avoid_entity.rs:11` | `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:59`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:61`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:63`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:116`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:123`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:53`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:55`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:57`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:110`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:53`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:55`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:57`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:110`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:52`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:54`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:56`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:109`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:109`, `steel-core/src/entity/entities/mobs/passive/villager.rs:111`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:116`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:120`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:134` |
| BreathAirGoal | `steel-core/src/entity/ai/goal/breath_air.rs:22` | `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:140`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:141`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:139`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:141` |
| BreedGoal | `steel-core/src/entity/ai/goal/breed_goal.rs:12` | `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:106`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:100`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:100`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:96`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:100`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:100`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:100`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:94`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:100`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:112`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:106`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:186`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:116`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:111`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:147`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:120`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:136` |
| ClimbOnTopOfPowderSnowGoal | `steel-core/src/entity/ai/goal/climb_on_top_of_powder_snow.rs:8` | `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:95` |
| DoorInteractGoal | `steel-core/src/entity/ai/goal/door_interact.rs:13` | `steel-core/src/entity/ai/goal/open_door.rs:1`, `steel-core/src/entity/ai/goal/open_door.rs:9`, `steel-core/src/entity/ai/goal/open_door.rs:19`, `steel-core/src/entity/ai/goal/open_door.rs:27`, `steel-core/src/entity/ai/goal/restrict_open_door.rs:6`, `steel-core/src/entity/ai/goal/restrict_open_door.rs:13`, `steel-core/src/entity/ai/goal/restrict_open_door.rs:21`, `steel-core/src/entity/ai/goal/restrict_open_door.rs:28`, `steel-core/src/entity/ai/goal/raid_break_door.rs:2`, `steel-core/src/entity/ai/goal/raid_break_door.rs:8`, `steel-core/src/entity/ai/goal/raid_break_door.rs:15` |
| EatBlockGoal | `steel-core/src/entity/ai/goal/eat_block_goal.rs:31` | `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:134` |
| FleeSunGoal | `steel-core/src/entity/ai/goal/flee_sun.rs:11` | `steel-core/src/entity/entities/mobs/hostile/bogged.rs:8`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:113`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:9`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:107`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:9`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:107`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:8`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:106` |
| FloatGoal | `steel-core/src/entity/ai/goal/float_goal.rs:6` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:99`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:94`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:77`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:78`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:111`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:86`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:86`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:126`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:142`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:120`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:82`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:114`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:86`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:79`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:78`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:87`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:86`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:82`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:79`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:85`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:117`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:105`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:166`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:87`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:82`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:98`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:79`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:88`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:86`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:105`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:101`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:82`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:86`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:104`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:78`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:86`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:90`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:79`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:82`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:86`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:82`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:117`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:87`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:110`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:105`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:104`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:98`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:113`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:98`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:116`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:98`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:94`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:98`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:98`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:98`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:92`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:98`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:107`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:106`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:104`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:184`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:114`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:109`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:145`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:118`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:163`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:114`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:98`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:100`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:98`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:113`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:118`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:130` |
| FollowMobGoal | `steel-core/src/entity/ai/goal/follow_mob.rs:10` | `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:122`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:126` |
| FollowParentGoal | `steel-core/src/entity/ai/goal/follow_parent.rs:11` | `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:113`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:107`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:101`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:115`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:100`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:101`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:97`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:101`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:101`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:101`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:95`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:101`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:120`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:107`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:199`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:129`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:132`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:160`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:137`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:166`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:116`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:100`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:102`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:100`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:116`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:121` |
| HurtByTargetGoal | `steel-core/src/entity/ai/goal/hurt_by_target.rs:12` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:106`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:102`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:84`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:85`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:136`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:93`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:93`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:133`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:149`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:143`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:89`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:121`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:93`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:86`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:85`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:94`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:93`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:89`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:86`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:93`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:123`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:130`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:173`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:94`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:89`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:105`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:86`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:95`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:94`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:130`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:107`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:89`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:93`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:129`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:85`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:95`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:97`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:86`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:89`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:93`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:89`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:124`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:94` |
| InteractGoal | `steel-core/src/entity/ai/goal/interact.rs:6` | `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:121` |
| LeapAtTargetGoal | `steel-core/src/entity/ai/goal/leap_at_target.rs:14` | `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:60`, `steel-core/src/entity/entities/mobs/passive/frog.rs:119` |
| LookAtPlayerGoal | `steel-core/src/entity/ai/goal/look_at_player.rs:19` | `steel-core/src/entity/ai/goal/interact.rs:1`, `steel-core/src/entity/ai/goal/interact.rs:7`, `steel-core/src/entity/ai/goal/interact.rs:14`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:102`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:98`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:80`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:81`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:65`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:132`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:89`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:89`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:129`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:145`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:72`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:135`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:85`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:117`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:89`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:82`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:81`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:90`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:89`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:85`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:82`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:89`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:119`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:59`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:126`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:169`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:90`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:85`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:101`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:82`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:91`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:90`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:59`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:126`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:103`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:85`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:89`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:58`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:125`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:81`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:91`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:93`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:82`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:85`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:89`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:85`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:120`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:90`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:120`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:107`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:109`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:103`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:117`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:102`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:123`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:147`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:103`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:99`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:103`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:103`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:103`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:97`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:103`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:125`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:122`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:109`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:201`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:131`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:134`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:162`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:136`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:139`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:168`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:107`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:118`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:107`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:107`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:102`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:102`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:128`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:131`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:144` |
| MeleeAttackGoal | `steel-core/src/entity/ai/goal/melee_attack.rs:22` | `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:96`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:78`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:79`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:87`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:11`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:87`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:127`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:143`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:133`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:83`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:115`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:87`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:80`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:79`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:88`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:87`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:83`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:80`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:86`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:167`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:88`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:83`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:80`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:89`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:87`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:83`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:87`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:79`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:89`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:91`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:80`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:83`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:87`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:83`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:118`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:88`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:111`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:70`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:143`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:141` |
| SilverfishMergeWithStoneGoal | `steel-core/src/entity/ai/goal/merge_with_stone.rs:29` | `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:9`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:30`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:88` |
| MoveToBlockGoal | `steel-core/src/entity/ai/goal/move_to_block.rs:18` | `steel-core/src/entity/ai/goal/sleep_in_bed.rs:18`, `steel-core/src/entity/ai/goal/sleep_in_bed.rs:35`, `steel-core/src/entity/ai/goal/sleep_in_bed.rs:45`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:7` |
| MoveTowardsRestrictionGoal | `steel-core/src/entity/ai/goal/move_towards_restriction.rs:9` | `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:115` |
| MoveTowardsTargetGoal | `steel-core/src/entity/ai/goal/move_towards_target.rs:9` | `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:28`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:87` |
| NearestAttackableTargetGoal | `steel-core/src/entity/ai/goal/nearest_attackable_target.rs:17` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:109`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:20`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:105`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:87`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:88`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:139`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:143`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:96`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:96`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:136`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:152`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:35`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:141`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:92`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:124`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:89`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:88`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:97`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:96`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:92`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:89`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:98`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:126`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:130`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:133`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:137`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:176`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:97`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:92`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:108`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:112`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:119`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:89`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:98`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:97`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:133`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:137`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:110`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:114`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:92`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:96`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:132`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:136`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:88`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:98`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:100`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:100`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:89`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:92`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:96`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:92`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:127`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:97` |
| OpenDoorGoal | `steel-core/src/entity/ai/goal/open_door.rs:8` | `steel-core/src/entity/ai/goal/restrict_open_door.rs:7`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:88`, `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:116` |
| PanicGoal | `steel-core/src/entity/ai/goal/panic_goal.rs:14` | `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:112`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:105`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:142`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:99`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:114`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:99`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:54`, `steel-core/src/entity/entities/mobs/passive/frog.rs:117`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:99`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:95`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:99`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:99`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:99`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:93`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:99`, `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:108`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:119`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:105`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:185`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:115`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:110`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:146`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:119`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:134`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:107`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:187`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:470`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:41`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:182`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:42`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:175`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:27`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:130`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:48`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:174`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:164`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:115`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:99`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:101`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:99`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:114`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:119`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:131`, `steel-core/src/entity/mob/pathfinder.rs:320` |
| RaidBreakDoorGoal | `steel-core/src/entity/ai/goal/raid_break_door.rs:7` | `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:87` |
| RaidCelebrationGoal | `steel-core/src/entity/ai/goal/raid_celebration.rs:6` | `steel-core/src/entity/entities/mobs/hostile/evoker.rs:78`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:78`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:97`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:78`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:85`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:78` |
| RandomLookAroundGoal | `steel-core/src/entity/ai/goal/random_look_around.rs:11` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:26`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:103`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:20`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:99`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:22`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:81`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:22`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:82`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:133`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:27`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:90`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:90`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:38`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:130`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:28`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:146`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:35`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:136`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:26`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:86`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:29`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:118`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:28`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:90`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:22`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:83`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:82`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:91`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:90`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:26`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:86`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:22`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:83`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:28`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:90`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:46`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:120`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:127`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:27`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:170`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:91`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:26`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:86`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:29`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:102`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:83`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:28`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:92`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:30`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:91`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:127`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:42`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:104`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:26`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:86`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:90`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:126`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:22`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:82`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:92`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:29`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:94`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:22`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:83`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:26`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:86`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:90`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:26`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:86`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:37`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:121`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:91`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:26`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:122`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:108`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:110`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:104`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:118`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:103`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:125`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:148`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:104`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:100`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:104`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:104`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:104`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:98`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:104`, `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:126`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:110`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:202`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:132`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:135`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:163`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:31`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:137`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:140`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:169`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:108`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:119`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:108`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:108`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:103`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:130`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:146` |
| RandomStrollGoal | `steel-core/src/entity/ai/goal/random_stroll.rs:10` | `steel-core/src/entity/ai/goal/random_swimming.rs:5`, `steel-core/src/entity/ai/goal/random_swimming.rs:16`, `steel-core/src/entity/ai/goal/random_swimming.rs:23`, `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:4`, `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:11`, `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:24`, `steel-core/src/entity/entities/mobs/passive/frog.rs:31`, `steel-core/src/entity/entities/mobs/passive/frog.rs:56`, `steel-core/src/entity/entities/mobs/passive/frog.rs:120` |
| RandomSwimmingGoal | `steel-core/src/entity/ai/goal/random_swimming.rs:15` | `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:146`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:145`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:110`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:188`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:471`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:41`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:185`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:42`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:178`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:27`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:133`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:48`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:177` |
| RangedAttackGoal | `steel-core/src/entity/ai/goal/ranged_attack.rs:23` | `steel-core/src/entity/ranged_attack.rs:3`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:100`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:9`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:125`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:10`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:119`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:10`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:119`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:9`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:118`, `steel-core/src/entity/mob/pathfinder.rs:106` |
| RestrictOpenDoorGoal | `steel-core/src/entity/ai/goal/restrict_open_door.rs:20` | `steel-core/src/entity/entities/mobs/passive/villager.rs:28`, `steel-core/src/entity/entities/mobs/passive/villager.rs:117`, `steel-core/src/entity/entities/mobs/passive/villager.rs:119` |
| RestrictSunGoal | `steel-core/src/entity/ai/goal/restrict_sun.rs:5` | `steel-core/src/entity/entities/mobs/hostile/bogged.rs:8`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:112`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:9`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:106`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:9`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:106`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:8`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:105` |
| SleepInBedGoal | `steel-core/src/entity/ai/goal/sleep_in_bed.rs:34` | `steel-core/src/entity/entities/mobs/passive/villager.rs:28`, `steel-core/src/entity/entities/mobs/passive/villager.rs:121`, `steel-core/src/entity/entities/mobs/passive/villager.rs:123` |
| SquidFleeGoal | `steel-core/src/entity/ai/goal/squid_flee_goal.rs:16` | `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:40`, `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:301`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:38`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:205` |
| SquidRandomMovementGoal | `steel-core/src/entity/ai/goal/squid_random_movement_goal.rs:25` | `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:32`, `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:300`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:30`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:204` |
| TemptGoal | `steel-core/src/entity/ai/goal/tempt_goal.rs:17` | `steel-core/src/entity/entities/mobs/passive/frog.rs:31`, `steel-core/src/entity/entities/mobs/passive/frog.rs:58`, `steel-core/src/entity/entities/mobs/passive/frog.rs:118`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:189`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:119`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:114`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:122`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:42`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:150`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:31`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:123`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:135`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:165`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:115`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:120`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:140` |
| TryFindLiquidGoal | `steel-core/src/entity/ai/goal/try_find_liquid.rs:14` | `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:142` |
| WaterAvoidingRandomStrollGoal | `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:10` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:26`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:101`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:20`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:97`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:22`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:79`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:22`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:80`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:38`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:131`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:27`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:88`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:88`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:38`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:128`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:28`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:144`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:35`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:134`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:26`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:84`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:29`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:116`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:28`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:88`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:22`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:81`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:80`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:89`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:88`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:26`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:84`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:22`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:81`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:28`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:88`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:46`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:118`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:33`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:125`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:27`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:168`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:89`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:26`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:84`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:29`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:100`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:81`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:28`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:90`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:30`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:89`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:33`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:125`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:42`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:102`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:26`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:84`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:88`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:32`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:124`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:22`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:80`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:90`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:29`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:92`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:22`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:81`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:26`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:84`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:88`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:26`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:84`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:37`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:119`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:89`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:26`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:116`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:106`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:29`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:108`, `steel-core/src/entity/entities/mobs/passive/bee.rs:31`, `steel-core/src/entity/entities/mobs/passive/bee.rs:102`, `steel-core/src/entity/entities/mobs/passive/camel.rs:34`, `steel-core/src/entity/entities/mobs/passive/camel.rs:116`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:27`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:101`, `steel-core/src/entity/entities/mobs/passive/goat.rs:31`, `steel-core/src/entity/entities/mobs/passive/goat.rs:102`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:27`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:98`, `steel-core/src/entity/entities/mobs/passive/panda.rs:31`, `steel-core/src/entity/entities/mobs/passive/panda.rs:102`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:31`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:102`, `steel-core/src/entity/entities/mobs/passive/strider.rs:31`, `steel-core/src/entity/entities/mobs/passive/strider.rs:102`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:25`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:96`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:31`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:102`, `steel-core/src/entity/entities/mobs/passive/villager.rs:28`, `steel-core/src/entity/entities/mobs/passive/villager.rs:124`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:27`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:120`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:28`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:108`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:200`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:130`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:42`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:161`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:31`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:135`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:138`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:167`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:28`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:106`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:39`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:117`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:28`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:106`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:28`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:106`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:101`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:101`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:117`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:128`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:29`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:141` |

## entities: входы генератора

| Объект | Vanilla class | Точная аннотация реализации |
|---|---|---|
| acacia_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| acacia_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| allay | Allay | `steel-core/src/entity/entities/mobs/passive/allay.rs:59` |
| area_effect_cloud | AreaEffectCloud | `steel-core/src/entity/entities/objects/effects/area_effect_cloud.rs:88` |
| armadillo | Armadillo | `steel-core/src/entity/entities/mobs/passive/armadillo.rs:48` |
| armor_stand | ArmorStand | `steel-core/src/entity/entities/objects/decoration/armor_stand.rs:74` |
| arrow | Arrow | `steel-core/src/entity/entities/objects/projectiles/arrow.rs:667` |
| axolotl | Axolotl | `steel-core/src/entity/entities/mobs/passive/axolotl.rs:83` |
| bamboo_chest_raft | ChestRaft | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:212` |
| bamboo_raft | Raft | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1108` |
| bat | Bat | `steel-core/src/entity/entities/mobs/ambient/bat.rs:59` |
| bee | Bee | `steel-core/src/entity/entities/mobs/passive/bee.rs:45` |
| birch_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| birch_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| blaze | Blaze | `steel-core/src/entity/entities/mobs/hostile/blaze.rs:36` |
| block_display | BlockDisplay | `steel-core/src/entity/entities/objects/display_ui/block_display.rs:27` |
| bogged | Bogged | `steel-core/src/entity/entities/mobs/hostile/bogged.rs:69` |
| breeze | Breeze | `steel-core/src/entity/entities/mobs/hostile/breeze.rs:44` |
| breeze_wind_charge | BreezeWindCharge | `steel-core/src/entity/entities/objects/projectiles/breeze_wind_charge.rs:42` |
| camel | Camel | `steel-core/src/entity/entities/mobs/passive/camel.rs:56` |
| camel_husk | CamelHusk | `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:45` |
| cat | Cat | `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:80` |
| cave_spider | CaveSpider | `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:44` |
| cherry_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| cherry_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| chest_minecart | MinecartChest | `steel-core/src/entity/entities/objects/vehicles/chest_minecart.rs:38` |
| chicken | Chicken | `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:124` |
| cod | Cod | `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:46` |
| copper_golem | CopperGolem | `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:81` |
| command_block_minecart | MinecartCommandBlock | `steel-core/src/entity/entities/objects/vehicles/command_block_minecart.rs:34` |
| cow | Cow | `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:61` |
| creaking | Creaking | `steel-core/src/entity/entities/mobs/hostile/creaking.rs:42` |
| creeper | Creeper | `steel-core/src/entity/entities/mobs/hostile/creeper.rs:76` |
| cushion | Cushion | `steel-core/src/entity/entities/objects/decoration/cushion.rs:61` |
| dark_oak_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| dark_oak_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| dolphin | Dolphin | `steel-core/src/entity/entities/mobs/passive/dolphin.rs:82` |
| donkey | Donkey | `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:48` |
| dragon_fireball | DragonFireball | `steel-core/src/entity/entities/objects/projectiles/dragon_fireball.rs:49` |
| drowned | Drowned | `steel-core/src/entity/entities/mobs/hostile/drowned.rs:40` |
| egg | ThrownEgg | `steel-core/src/entity/entities/objects/projectiles/thrown_egg.rs:50` |
| elder_guardian | ElderGuardian | `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:50` |
| enderman | Enderman | `steel-core/src/entity/entities/mobs/hostile/enderman.rs:43` |
| endermite | Endermite | `steel-core/src/entity/entities/mobs/hostile/endermite.rs:36` |
| ender_dragon | EnderDragon | `steel-core/src/entity/entities/mobs/bosses/ender_dragon/mod.rs:133` |
| ender_pearl | ThrownEnderpearl | `steel-core/src/entity/entities/objects/projectiles/ender_pearl.rs:49` |
| end_crystal | EndCrystal | `steel-core/src/entity/entities/objects/explosives/end_crystal.rs:31` |
| evoker | Evoker | `steel-core/src/entity/entities/mobs/hostile/evoker.rs:36` |
| evoker_fangs | EvokerFangs | `steel-core/src/entity/entities/objects/effects/evoker_fangs.rs:35` |
| experience_bottle | ThrownExperienceBottle | `steel-core/src/entity/entities/objects/projectiles/experience_bottle.rs:50` |
| experience_orb | ExperienceOrb | `steel-core/src/entity/entities/objects/items/experience_orb.rs:65` |
| eye_of_ender | EyeOfEnder | `steel-core/src/entity/entities/objects/projectiles/eye_of_ender.rs:70` |
| falling_block | FallingBlockEntity | `steel-core/src/entity/entities/objects/items/falling_block.rs:73` |
| fireball | LargeFireball | `steel-core/src/entity/entities/objects/projectiles/large_fireball.rs:29` |
| firework_rocket | FireworkRocketEntity | `steel-core/src/entity/entities/objects/projectiles/firework_rocket.rs:53` |
| fox | Fox | `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:110` |
| frog | Frog | `steel-core/src/entity/entities/mobs/passive/frog.rs:65` |
| furnace_minecart | MinecartFurnace | `steel-core/src/entity/entities/objects/vehicles/furnace_minecart.rs:37` |
| ghast | Ghast | `steel-core/src/entity/entities/mobs/hostile/ghast.rs:36` |
| happy_ghast | HappyGhast | `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:41` |
| giant | Giant | `steel-core/src/entity/entities/mobs/hostile/giant.rs:35` |
| glow_item_frame | GlowItemFrame | `steel-core/src/entity/entities/objects/display_ui/glow_item_frame.rs:43` |
| glow_squid | GlowSquid | `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:70` |
| goat | Goat | `steel-core/src/entity/entities/mobs/passive/goat.rs:45` |
| guardian | Guardian | `steel-core/src/entity/entities/mobs/hostile/guardian.rs:42` |
| hoglin | Hoglin | `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:44` |
| hopper_minecart | MinecartHopper | `steel-core/src/entity/entities/objects/vehicles/hopper_minecart.rs:53` |
| horse | Horse | `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:58` |
| husk | Husk | `steel-core/src/entity/entities/mobs/hostile/husk.rs:40` |
| illusioner | Illusioner | `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:36` |
| interaction | Interaction | `steel-core/src/entity/entities/objects/technical/interaction.rs:47` |
| iron_golem | IronGolem | `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:42` |
| item | ItemEntity | Нет точной аннотации: проверить наследование/генератор/fallback |
| item_display | ItemDisplay | `steel-core/src/entity/entities/objects/decoration/display.rs:259` |
| item_frame | ItemFrame | `steel-core/src/entity/entities/objects/display_ui/item_frame.rs:45` |
| jungle_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| jungle_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| leash_knot | LeashFenceKnotEntity | `steel-core/src/entity/entities/objects/display_ui/leash_fence_knot.rs:29` |
| lightning_bolt | LightningBolt | `steel-core/src/entity/entities/objects/effects/lightning_bolt.rs:40` |
| llama | Llama | `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:48` |
| llama_spit | LlamaSpit | `steel-core/src/entity/entities/objects/projectiles/llama_spit.rs:25` |
| magma_cube | MagmaCube | `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:74` |
| mangrove_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| mangrove_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| mannequin | Mannequin | `steel-core/src/entity/entities/objects/decoration/mannequin.rs:35` |
| marker | Marker | `steel-core/src/entity/entities/objects/technical/marker.rs:16` |
| minecart | Minecart | `steel-core/src/entity/entities/objects/vehicles/minecart.rs:40` |
| mooshroom | MushroomCow | `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:80` |
| mule | Mule | `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:48` |
| nautilus | Nautilus | `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:129` |
| oak_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| oak_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| ocelot | Ocelot | `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:60` |
| ominous_item_spawner | OminousItemSpawner | `steel-core/src/entity/entities/objects/technical/ominous_item_spawner.rs:27` |
| painting | Painting | `steel-core/src/entity/entities/objects/decoration/painting.rs:55` |
| pale_oak_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| pale_oak_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| panda | Panda | `steel-core/src/entity/entities/mobs/passive/panda.rs:45` |
| parched | Parched | `steel-core/src/entity/entities/mobs/hostile/parched.rs:63` |
| parrot | Parrot | `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:64` |
| phantom | Phantom | `steel-core/src/entity/entities/mobs/hostile/phantom.rs:123` |
| pig | Pig | `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:57` |
| piglin | Piglin | `steel-core/src/entity/entities/mobs/hostile/piglin.rs:45` |
| piglin_brute | PiglinBrute | `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:40` |
| pillager | Pillager | `steel-core/src/entity/entities/mobs/hostile/pillager.rs:50` |
| polar_bear | PolarBear | `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:57` |
| poplar_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| poplar_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| splash_potion | ThrownSplashPotion | `steel-core/src/entity/entities/objects/projectiles/splash_potion.rs:34` |
| lingering_potion | ThrownLingeringPotion | `steel-core/src/entity/entities/objects/projectiles/lingering_potion.rs:33` |
| pufferfish | Pufferfish | `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:100` |
| rabbit | Rabbit | `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:77` |
| ravager | Ravager | `steel-core/src/entity/entities/mobs/hostile/ravager.rs:36` |
| salmon | Salmon | `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:84` |
| sheep | Sheep | `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:66` |
| shulker | Shulker | `steel-core/src/entity/entities/mobs/hostile/shulker.rs:46` |
| shulker_bullet | ShulkerBullet | `steel-core/src/entity/entities/objects/projectiles/shulker_bullet.rs:39` |
| silverfish | Silverfish | `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:44` |
| skeleton | Skeleton | `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:63` |
| skeleton_horse | SkeletonHorse | `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:45` |
| slime | Slime | `steel-core/src/entity/entities/mobs/hostile/slime.rs:58` |
| small_fireball | SmallFireball | `steel-core/src/entity/entities/objects/projectiles/small_fireball.rs:30` |
| sniffer | Sniffer | `steel-core/src/entity/entities/mobs/passive/sniffer.rs:45` |
| snowball | Snowball | `steel-core/src/entity/entities/objects/projectiles/snowball.rs:36` |
| snow_golem | SnowGolem | `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:40` |
| spawner_minecart | MinecartSpawner | `steel-core/src/entity/entities/objects/vehicles/spawner_minecart.rs:28` |
| spectral_arrow | SpectralArrow | `steel-core/src/entity/entities/objects/projectiles/arrow.rs:715` |
| spider | Spider | `steel-core/src/entity/entities/mobs/hostile/spider.rs:44` |
| spruce_boat | Boat | `steel-core/src/entity/entities/objects/vehicles/boat.rs:1068` |
| spruce_chest_boat | ChestBoat | `steel-core/src/entity/entities/objects/vehicles/chest_boat.rs:149` |
| squid | Squid | `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:76` |
| stray | Stray | `steel-core/src/entity/entities/mobs/hostile/stray.rs:62` |
| strider | Strider | `steel-core/src/entity/entities/mobs/passive/strider.rs:45` |
| sulfur_cube | SulfurCube | `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:39` |
| tadpole | Tadpole | `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:69` |
| text_display | TextDisplay | `steel-core/src/entity/entities/objects/decoration/display.rs:295` |
| tnt | PrimedTnt | `steel-core/src/entity/entities/objects/explosives/primed_tnt.rs:133` |
| tnt_minecart | MinecartTNT | `steel-core/src/entity/entities/objects/vehicles/tnt_minecart.rs:39` |
| trader_llama | TraderLlama | `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:45` |
| trident | ThrownTrident | `steel-core/src/entity/entities/objects/projectiles/arrow.rs:783` |
| tropical_fish | TropicalFish | `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:110` |
| turtle | Turtle | `steel-core/src/entity/entities/mobs/passive/turtle.rs:45` |
| vex | Vex | `steel-core/src/entity/entities/mobs/hostile/vex.rs:36` |
| villager | Villager | `steel-core/src/entity/entities/mobs/passive/villager.rs:56` |
| vindicator | Vindicator | `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:43` |
| wandering_trader | WanderingTrader | `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:46` |
| warden | Warden | `steel-core/src/entity/entities/mobs/hostile/warden.rs:43` |
| wind_charge | WindCharge | `steel-core/src/entity/entities/objects/projectiles/wind_charge.rs:42` |
| witch | Witch | `steel-core/src/entity/entities/mobs/hostile/witch.rs:36` |
| wither | WitherBoss | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:52` |
| wither_skeleton | WitherSkeleton | `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:40` |
| wither_skull | WitherSkull | `steel-core/src/entity/entities/objects/projectiles/wither_skull.rs:46` |
| wolf | Wolf | `steel-core/src/entity/entities/mobs/passive/wolf.rs:51` |
| zoglin | Zoglin | `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:44` |
| zombie | Zombie | `steel-core/src/entity/entities/mobs/hostile/zombie.rs:40` |
| zombie_horse | ZombieHorse | `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:45` |
| zombie_nautilus | ZombieNautilus | `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:416` |
| zombie_villager | ZombieVillager | `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:61` |
| zombified_piglin | ZombifiedPiglin | `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:45` |
| player | Player | Нет точной аннотации: проверить наследование/генератор/fallback |
| fishing_bobber | FishingHook | `steel-core/src/entity/entities/objects/projectiles/fishing_hook.rs:58` |

## blocks: входы генератора

| Объект | Vanilla class | Точная аннотация реализации |
|---|---|---|
| air | AirBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| grass_block | GrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dirt | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| coarse_dirt | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| podzol | SnowyBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_propagule | MangrovePropaguleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_sapling | SaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bedrock | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| water | LiquidBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lava | LiquidBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sand | SandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| suspicious_sand | BrushableBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sand | SandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gravel | ColoredFallingBlock | `steel-core/src/behavior/blocks/falling/falling_block.rs:19` |
| suspicious_gravel | BrushableBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_gold_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_iron_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| coal_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_coal_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_gold_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_roots | MangroveRootsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| muddy_mangrove_roots | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_block | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_spruce_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_birch_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_jungle_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_acacia_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_cherry_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_dark_oak_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_pale_oak_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_oak_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_mangrove_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_poplar_log | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_bamboo_block | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_oak_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_spruce_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_birch_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_jungle_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_acacia_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_cherry_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_dark_oak_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_pale_oak_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_mangrove_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_poplar_wood | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_leaves | TintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_leaves | LeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_leaves | TintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_leaves | TintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_leaves | TintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_leaves | TintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_leaves | MangroveLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_poplar_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_poplar_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_poplar_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| azalea_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| flowering_azalea_leaves | UntintedParticleLeavesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sponge | SpongeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wet_sponge | WetSpongeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| glass | TransparentBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lapis_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_lapis_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lapis_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dispenser | DispenserBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| note_block | NoteBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_bed | BedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| straw_bed | StrawBedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| powered_rail | PoweredRailBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| detector_rail | DetectorRailBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sticky_piston | PistonBaseBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobweb | WebBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| short_grass | TallGrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fern | TallGrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bush | DryVegetationBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bush | BushBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_shrub | BushBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| short_dry_grass | ShortDryGrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tall_dry_grass | TallDryGrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| seagrass | SeagrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tall_seagrass | TallSeagrassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| piston | PistonBaseBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| piston_head | PistonHeadBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wool | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wool_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wool_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| moving_piston | MovingPistonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dandelion | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_dandelion | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| torchflower | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poppy | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_orchid | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| allium | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| azure_bluet | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_tulip | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_tulip | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_tulip | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_tulip | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxeye_daisy | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cornflower | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wither_rose | WitherRoseBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lily_of_the_valley | FlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_mushroom | MushroomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_mushroom | MushroomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tnt | TntBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bookshelf | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_bookshelf | ChiseledBookShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_shelf | ShelfBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| obsidian | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| torch | TorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wall_torch | WallTorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire | FireBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_fire | SoulFireBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spawner | SpawnerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| creaking_heart | CreakingHeartBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chest | ChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_wire | RedstoneWireBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_diamond_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| crafting_table | CraftingTableBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wheat | CropBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| farmland | FarmlandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| furnace | FurnaceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| ladder | LadderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| rail | RailBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_hanging_sign | CeilingHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_wall_hanging_sign | WallHangingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lever | LeverBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_ore | RedStoneOreBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_redstone_ore | RedStoneOreBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_torch | RedstoneTorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_wall_torch | RedstoneWallTorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| snow | SnowLayerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| ice | IceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| snow_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cactus | CactusBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cactus_flower | CactusFlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| clay | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sugar_cane | SugarCaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jukebox | JukeboxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherrack | NetherrackBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_sand | SoulSandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_soil | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| basalt | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_basalt | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_torch | TorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_wall_torch | WallTorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_torch | TorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_wall_torch | WallTorchBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| glowstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_portal | NetherPortalBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| carved_pumpkin | CarvedPumpkinBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jack_o_lantern | CarvedPumpkinBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cake | CakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| repeater | RepeaterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_stained_glass | StainedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| packed_mud | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_stone | InfestedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_cobblestone | InfestedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_stone_bricks | InfestedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_mossy_stone_bricks | InfestedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_cracked_stone_bricks | InfestedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_chiseled_stone_bricks | InfestedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_mushroom_block | HugeMushroomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_mushroom_block | HugeMushroomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mushroom_stem | HugeMushroomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_bars | IronBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_bars | WeatheringCopperBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_bars | WeatheringCopperBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_bars | WeatheringCopperBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_bars | WeatheringCopperBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_bars | IronBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_bars | IronBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_bars | IronBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_bars | IronBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_chain | ChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_chain | WeatheringCopperChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_chain | WeatheringCopperChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_chain | WeatheringCopperChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_chain | WeatheringCopperChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_chain | ChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_chain | ChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_chain | ChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_chain | ChainBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| glass_pane | IronBarsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pumpkin | PumpkinBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| melon | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| attached_pumpkin_stem | AttachedStemBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| attached_melon_stem | AttachedStemBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pumpkin_stem | StemBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| melon_stem | StemBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| vine | VineBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| glow_lichen | GlowLichenBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_clump | MultifaceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mycelium | MyceliumBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lily_pad | LilyPadBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_resin_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_wart | NetherWartBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| enchanting_table | EnchantingTableBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brewing_stand | BrewingStandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cauldron | CauldronBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| water_cauldron | LayeredCauldronBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lava_cauldron | LavaCauldronBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| powder_snow_cauldron | LayeredCauldronBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_portal | EndPortalBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_portal_frame | EndPortalFrameBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_egg | DragonEggBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_lamp | RedstoneLampBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cocoa | CocoaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| shelf_mushroom | ShelfMushroomBlock | `steel-core/src/behavior/blocks/vegetation/shelf_mushroom_block.rs:29` |
| sandstone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| emerald_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_emerald_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| ender_chest | EnderChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tripwire_hook | TripWireHookBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tripwire | TripWireBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| emerald_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| command_block | CommandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| beacon | BeaconBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| flower_pot | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_torchflower | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_oak_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_spruce_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_birch_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_jungle_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_acacia_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_cherry_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_dark_oak_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_pale_oak_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_poplar_sapling | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_mangrove_propagule | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_fern | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_dandelion | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_golden_dandelion | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_poppy | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_blue_orchid | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_allium | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_azure_bluet | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_red_tulip | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_orange_tulip | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_white_tulip | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_pink_tulip | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_oxeye_daisy | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_cornflower | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_lily_of_the_valley | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_wither_rose | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_red_mushroom | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_brown_mushroom | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_dead_bush | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_cactus | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| carrots | CarrotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potatoes | PotatoBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| skeleton_skull | SkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| skeleton_wall_skull | WallSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wither_skeleton_skull | WitherSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wither_skeleton_wall_skull | WitherWallSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| zombie_head | SkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| zombie_wall_head | WallSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| player_head | PlayerHeadBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| player_wall_head | PlayerWallHeadBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| creeper_head | SkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| creeper_wall_head | WallSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_head | SkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_wall_head | WallSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| piglin_head | SkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| piglin_wall_head | PiglinWallSkullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| anvil | AnvilBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chipped_anvil | AnvilBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| damaged_anvil | AnvilBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| trapped_chest | TrappedChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_weighted_pressure_plate | WeightedPressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| heavy_weighted_pressure_plate | WeightedPressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| comparator | ComparatorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| daylight_detector | DaylightDetectorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_block | PoweredBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_quartz_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| hopper | HopperBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_quartz_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_pillar | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| activator_rail | PoweredRailBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dropper | DropperBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_stained_glass_pane | StainedGlassPaneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| slime_block | SlimeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| barrier | BarrierBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light | LightBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sea_lantern | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| hay_block | HayBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_carpet | WoolCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| coal_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| packed_ice | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sunflower | TallFlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lilac | TallFlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| rose_bush | TallFlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| peony | TallFlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tall_grass | DoublePlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| large_fern | DoublePlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_banner | BannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wall_banner | WallBannerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_sandstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| petrified_oak_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_red_sandstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_stone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_stone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_rod | EndRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chorus_plant | ChorusPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chorus_flower | ChorusFlowerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_pillar | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| torchflower_crop | TorchflowerCropBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pitcher_crop | PitcherCropBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pitcher_plant | DoublePlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| beetroots | BeetrootBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dirt_path | PathBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_gateway | EndGatewayBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| repeating_command_block | CommandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chain_command_block | CommandBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| frosted_ice | FrostedIceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magma_block | MagmaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_wart_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bone_block | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| structure_void | StructureVoidBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| observer | ObserverBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_shulker_box | ShulkerBoxBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_glazed_terracotta | GlazedTerracottaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete_powder | ConcretePowderBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| kelp | KelpBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| kelp_plant | KelpPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dried_kelp_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| turtle_egg | TurtleEggBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sniffer_egg | SnifferEggBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dried_ghast | DriedGhastBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral_block | CoralBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral_block | CoralBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral_block | CoralBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral_block | CoralBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral_block | CoralBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral | BaseCoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral | BaseCoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral | BaseCoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral | BaseCoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral | BaseCoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral | CoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral | CoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral | CoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral | CoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral | CoralPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral_fan | BaseCoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral_fan | BaseCoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral_fan | BaseCoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral_fan | BaseCoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral_fan | BaseCoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral_fan | CoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral_fan | CoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral_fan | CoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral_fan | CoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral_fan | CoralFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral_wall_fan | BaseCoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral_wall_fan | BaseCoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral_wall_fan | BaseCoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral_wall_fan | BaseCoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral_wall_fan | BaseCoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral_wall_fan | CoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral_wall_fan | CoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral_wall_fan | CoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral_wall_fan | CoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral_wall_fan | CoralWallFanBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sea_pickle | SeaPickleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_ice | HalfTransparentBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| conduit | ConduitBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_sapling | BambooSaplingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo | BambooStalkBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_bamboo | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| void_air | AirBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cave_air | AirBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_column | BubbleColumnBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_sandstone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_sandstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| scaffolding | ScaffoldingBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| loom | LoomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| barrel | BarrelBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smoker | SmokerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blast_furnace | BlastFurnaceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cartography_table | CartographyTableBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| fletching_table | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| grindstone | GrindstoneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lectern | LecternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smithing_table | SmithingTableBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stonecutter | StonecutterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bell | BellBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lantern | LanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_lantern | LanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_lantern | WeatheringLanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_lantern | WeatheringLanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_lantern | WeatheringLanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_lantern | WeatheringLanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_lantern | LanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_lantern | LanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_lantern | LanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_lantern | LanternBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| campfire | CampfireBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_campfire | CampfireBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sweet_berry_bush | SweetBerryBushBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_stem | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_warped_stem | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_hyphae | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_warped_hyphae | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_nylium | NyliumBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fungus | NetherFungusBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_wart_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_roots | NetherRootsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_sprouts | NetherSproutsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_stem | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_crimson_stem | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_hyphae | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_crimson_hyphae | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_nylium | NyliumBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_fungus | NetherFungusBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| shroomlight | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| weeping_vines | WeepingVinesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weeping_vines_plant | WeepingVinesPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| twisting_vines | TwistingVinesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| twisting_vines_plant | TwistingVinesPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_roots | NetherRootsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fence | FenceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fence_gate | FenceGateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_sign | StandingSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_wall_sign | WallSignBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| structure_block | StructureBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| jigsaw | JigsawBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| test_block | TestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| test_instance_block | TestInstanceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| composter | ComposterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| target | TargetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| bee_nest | BeehiveBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| beehive | BeehiveBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| honey_block | HoneyBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| honeycomb_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| ancient_debris | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| crying_obsidian | CryingObsidianBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| respawn_anchor | RespawnAnchorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_crimson_fungus | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_warped_fungus | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_crimson_roots | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_warped_roots | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lodestone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_polished_blackstone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_polished_blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gilded_blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_pressure_plate | PressurePlateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_button | ButtonBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_candle | CandleBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_candle_cake | CandleCakeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| amethyst_block | AmethystBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| budding_amethyst | BuddingAmethystBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| amethyst_cluster | AmethystClusterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| large_amethyst_bud | AmethystClusterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| medium_amethyst_bud | AmethystClusterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| small_amethyst_bud | AmethystClusterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_tuff | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_tuff_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| potent_sulfur | PotentSulfurBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_sulfur | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_cinnabar | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| calcite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tinted_glass | TintedGlassBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| powder_snow | PowderSnowBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_sensor | SculkSensorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| calibrated_sculk_sensor | CalibratedSculkSensorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk | SculkBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_vein | SculkVeinBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_catalyst | SculkCatalystBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_shrieker | SculkShriekerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_block | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_copper_ore | DropExperienceBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_cut_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_cut_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_cut_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_chiseled_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_chiseled_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_chiseled_copper | WeatheringCopperFullBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper_stairs | WeatheringCopperStairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_cut_copper_stairs | WeatheringCopperStairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_cut_copper_stairs | WeatheringCopperStairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_cut_copper_stairs | WeatheringCopperStairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_cut_copper_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper_slab | WeatheringCopperSlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_cut_copper_slab | WeatheringCopperSlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_cut_copper_slab | WeatheringCopperSlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_cut_copper_slab | WeatheringCopperSlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_cut_copper_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_door | WeatheringCopperDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_door | WeatheringCopperDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_door | WeatheringCopperDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_door | WeatheringCopperDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_door | DoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_trapdoor | WeatheringCopperTrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_trapdoor | WeatheringCopperTrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_trapdoor | WeatheringCopperTrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_trapdoor | WeatheringCopperTrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_trapdoor | TrapDoorBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_grate | WeatheringCopperGrateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_grate | WeatheringCopperGrateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_grate | WeatheringCopperGrateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_grate | WeatheringCopperGrateBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_grate | WaterloggedTransparentBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_grate | WaterloggedTransparentBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_grate | WaterloggedTransparentBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_grate | WaterloggedTransparentBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_bulb | WeatheringCopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_bulb | WeatheringCopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_bulb | WeatheringCopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_bulb | WeatheringCopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_bulb | CopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_bulb | CopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_bulb | CopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_bulb | CopperBulbBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_chest | WeatheringCopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_chest | WeatheringCopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_chest | WeatheringCopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_chest | WeatheringCopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_chest | CopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_chest | CopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_chest | CopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_chest | CopperChestBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_golem_statue | WeatheringCopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_golem_statue | WeatheringCopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_golem_statue | WeatheringCopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_golem_statue | WeatheringCopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_golem_statue | CopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_golem_statue | CopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_golem_statue | CopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_golem_statue | CopperGolemStatueBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| lightning_rod | WeatheringLightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_lightning_rod | WeatheringLightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_lightning_rod | WeatheringLightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_lightning_rod | WeatheringLightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_lightning_rod | LightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_lightning_rod | LightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_lightning_rod | LightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_lightning_rod | LightningRodBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| dripstone_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pointed_dripstone | PointedDripstoneBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_spike | SulfurSpikeBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cave_vines | CaveVinesBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cave_vines_plant | CaveVinesPlantBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| spore_blossom | SporeBlossomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| azalea | AzaleaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| flowering_azalea | AzaleaBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| moss_carpet | CarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_petals | FlowerBedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| wildflowers | FlowerBedBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| leaf_litter | LeafLitterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| moss_block | BonemealableFeaturePlacerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| big_dripleaf | BigDripleafBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| big_dripleaf_stem | BigDripleafStemBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| small_dripleaf | SmallDripleafBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| hanging_roots | HangingRootsBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| rooted_dirt | RootedDirtBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud | MudBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tiles | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_stairs | StairBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_slab | SlabBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_wall | WallBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_deepslate_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_deepslate_tiles | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_deepslate | InfestedRotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_basalt | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_iron_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_copper_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_gold_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_azalea_bush | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_flowering_azalea_bush | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| ochre_froglight | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| verdant_froglight | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pearlescent_froglight | RotatedPillarBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| frogspawn | FrogspawnBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| reinforced_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| decorated_pot | DecoratedPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| crafter | CrafterBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| trial_spawner | TrialSpawnerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| vault | VaultBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| heavy_core | HeavyCoreBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_moss_block | BonemealableFeaturePlacerBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_moss_carpet | MossyCarpetBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_hanging_moss | HangingMossBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| open_eyeblossom | EyeblossomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| closed_eyeblossom | EyeblossomBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_open_eyeblossom | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_closed_eyeblossom | FlowerPotBlock | Нет точной аннотации: проверить наследование/генератор/fallback |
| firefly_bush | FireflyBushBlock | Нет точной аннотации: проверить наследование/генератор/fallback |

## items: входы генератора

| Объект | Vanilla class | Точная аннотация реализации |
|---|---|---|
| air | AirItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| calcite | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_tuff | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_tuff_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| potent_sulfur | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_sulfur | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_cinnabar | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dripstone_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| grass_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dirt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| coarse_dirt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| podzol | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| rooted_dirt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_nylium | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_nylium | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_planks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_propagule | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_sapling | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bedrock | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sand | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| suspicious_sand | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| suspicious_gravel | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sand | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gravel | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| coal_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_coal_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_iron_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_copper_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_gold_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_redstone_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| emerald_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_emerald_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lapis_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_lapis_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_diamond_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_gold_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_quartz_ore | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ancient_debris | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| coal_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_iron_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_copper_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_gold_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| heavy_core | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| amethyst_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| budding_amethyst | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_chiseled_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_roots | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| muddy_mangrove_roots | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_stem | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_stem | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_oak_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_spruce_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_birch_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_jungle_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_acacia_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_cherry_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_dark_oak_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_pale_oak_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_mangrove_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_poplar_log | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_crimson_stem | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_warped_stem | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_oak_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_spruce_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_birch_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_jungle_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_acacia_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_cherry_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_dark_oak_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_pale_oak_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_mangrove_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_poplar_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_crimson_hyphae | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_warped_hyphae | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stripped_bamboo_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_wood | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_hyphae | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_hyphae | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_poplar_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_poplar_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_poplar_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| azalea_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| flowering_azalea_leaves | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sponge | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wet_sponge | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tinted_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lapis_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobweb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| short_grass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| fern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bush | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_shrub | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| azalea | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| flowering_azalea | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bush | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| firefly_bush | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| short_dry_grass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tall_dry_grass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| seagrass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sea_pickle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wool | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wool_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_wool_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dandelion | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_dandelion | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| open_eyeblossom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| closed_eyeblossom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poppy | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_orchid | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| allium | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| azure_bluet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_tulip | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_tulip | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_tulip | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_tulip | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxeye_daisy | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cornflower | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lily_of_the_valley | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wither_rose | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| torchflower | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pitcher_plant | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spore_blossom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_mushroom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_mushroom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| shelf_mushroom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_fungus | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fungus | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_roots | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_roots | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_sprouts | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weeping_vines | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| twisting_vines | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sugar_cane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| kelp | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_petals | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wildflowers | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| leaf_litter | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| moss_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| moss_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_moss_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_hanging_moss | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_moss_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| hanging_roots | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| big_dripleaf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| small_dripleaf | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_stone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_sandstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| petrified_oak_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_red_sandstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_stone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_shelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bookshelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_bookshelf | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| decorated_pot | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| obsidian | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| torch | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chorus_plant | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chorus_flower | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_pillar | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spawner | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| creaking_heart | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crafting_table | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| farmland | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| furnace | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ladder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| snow | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ice | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| snow_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cactus | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cactus_flower | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| clay | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jukebox | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pumpkin | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| carved_pumpkin | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jack_o_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherrack | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_sand | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_soil | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| basalt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_basalt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_basalt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_torch | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_torch | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glowstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_stone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_cobblestone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_mossy_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_cracked_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_chiseled_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_deepslate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| packed_mud | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_deepslate_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tiles | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_deepslate_tiles | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_deepslate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| reinforced_deepslate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_mushroom_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_mushroom_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mushroom_stem | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_bars | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_chain | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| melon | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| vine | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glow_lichen | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_clump | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_resin_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mycelium | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lily_pad | PlaceOnWaterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_nether_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_nether_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_fence | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_vein | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_catalyst | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_shrieker | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| enchanting_table | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_portal_frame | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_egg | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ender_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| emerald_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| command_block | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| beacon | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobblestone_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sandstone_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_wall | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| anvil | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chipped_anvil | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| damaged_anvil | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_quartz_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_pillar | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| barrier | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| hay_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_carpet | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| packed_ice | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dirt_path | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sunflower | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lilac | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| rose_bush | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| peony | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tall_grass | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| large_fern | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_stained_glass | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_stained_glass_pane | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sea_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_red_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_red_sandstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| repeating_command_block | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chain_command_block | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magma_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_wart_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_wart_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bone_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| structure_void | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_shulker_box | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_glazed_terracotta | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_concrete_powder | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| turtle_egg | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sniffer_egg | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dried_ghast | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brain_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bubble_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| horn_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_tube_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral_fan | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_ice | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| conduit | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_sandstone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_cobblestone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_stone_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_sandstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| scaffolding | ScaffoldingBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_torch | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| repeater | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| comparator | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| piston | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sticky_piston | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| slime_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| honey_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| observer | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| hopper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dispenser | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dropper | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lectern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| target | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lever | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_lightning_rod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| daylight_detector | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sculk_sensor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| calibrated_sculk_sensor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tripwire_hook | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| trapped_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tnt | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| redstone_lamp | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| note_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_button | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_weighted_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| heavy_weighted_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_pressure_plate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_door | DoubleHighBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_trapdoor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fence_gate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| powered_rail | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| detector_rail | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| rail | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| activator_rail | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| saddle | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_harness | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| chest_minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| furnace_minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| tnt_minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| hopper_minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| carrot_on_a_stick | FoodOnAStickItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_fungus_on_a_stick | FoodOnAStickItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| phantom_membrane | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| elytra | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| oak_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| spruce_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| spruce_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| birch_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| birch_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| jungle_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| jungle_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| acacia_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| acacia_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| cherry_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| cherry_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| dark_oak_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| dark_oak_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| pale_oak_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| pale_oak_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| mangrove_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| mangrove_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| poplar_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| poplar_chest_boat | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| bamboo_raft | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| bamboo_chest_raft | BoatItem | `steel-core/src/behavior/items/boat.rs:27` |
| structure_block | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jigsaw | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| test_block | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| test_instance_block | GameMasterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| turtle_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| turtle_scute | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| armadillo_scute | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wolf_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| flint_and_steel | FlintAndSteelItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bowl | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| apple | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| bow | BowItem | `steel-core/src/behavior/items/bow.rs:37` |
| arrow | ArrowItem | `steel-core/src/behavior/items/arrow.rs:21` |
| coal | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| charcoal | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| emerald | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| lapis_lazuli | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| amethyst_shard | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_iron | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_ingot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_copper | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_ingot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_gold | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_ingot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_ingot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_scrap | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wooden_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wooden_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wooden_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wooden_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wooden_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_sword | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_shovel | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_pickaxe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_axe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_hoe | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stick | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| mushroom_stew | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| string | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| feather | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| gunpowder | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wheat_seeds | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wheat | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| bread | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| leather_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| leather_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| leather_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| leather_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chainmail_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chainmail_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chainmail_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chainmail_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_chestplate | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_leggings | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_boots | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| flint | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| porkchop | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_porkchop | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| painting | HangingEntityItem | `steel-core/src/behavior/items/hanging_entity_item.rs:23` |
| golden_apple | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| enchanted_golden_apple | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_sign | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_hanging_sign | HangingSignItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bucket | BucketItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| water_bucket | BucketItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lava_bucket | BucketItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| powder_snow_bucket | SolidBucketItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| snowball | SnowballItem | `steel-core/src/behavior/items/snowball.rs:30` |
| leather | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| milk_bucket | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pufferfish_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| salmon_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| cod_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| tropical_fish_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| axolotl_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| sulfur_cube_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| tadpole_bucket | MobBucketItem | `steel-core/src/behavior/items/mob_bucket.rs:30` |
| brick | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| clay_ball | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| dried_kelp_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| paper | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| book | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| slime_ball | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| egg | EggItem | `steel-core/src/behavior/items/egg.rs:30` |
| blue_egg | EggItem | `steel-core/src/behavior/items/egg.rs:30` |
| brown_egg | EggItem | `steel-core/src/behavior/items/egg.rs:30` |
| compass | CompassItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| recovery_compass | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| white_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| orange_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| magenta_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| light_blue_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| yellow_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| lime_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| pink_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| gray_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| light_gray_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| cyan_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| purple_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| blue_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| brown_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| green_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| red_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| black_bundle | BundleItem | `steel-core/src/behavior/items/bundle.rs:40` |
| white_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| orange_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| magenta_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| light_blue_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| yellow_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| lime_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| pink_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| gray_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| light_gray_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| cyan_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| purple_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| blue_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| brown_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| green_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| red_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| black_cushion | CushionItem | `steel-core/src/behavior/items/cushion.rs:33` |
| fishing_rod | FishingRodItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| clock | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| spyglass | SpyglassItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glowstone_dust | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| salmon | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| tropical_fish | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pufferfish | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_cod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_salmon | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ink_sac | InkSacItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glow_ink_sac | GlowInkSacItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cocoa_beans | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| orange_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| magenta_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| light_blue_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| yellow_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| lime_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| pink_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| gray_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| light_gray_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| cyan_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| purple_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| blue_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| brown_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| green_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| red_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| black_dye | DyeItem | `steel-core/src/behavior/items/dye.rs:23` |
| bone_meal | BoneMealItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bone | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| sugar | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cake | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| straw_bed | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cookie | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| crafter | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| filled_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| ocean_monument_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| woodland_mansion_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| buried_trial_chambers_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| jungle_pyramid_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| swamp_hut_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| desert_village_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| plains_village_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| savanna_village_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| snowy_village_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| taiga_village_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| buried_treasure_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| buried_ancient_city_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| buried_mineshaft_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| desert_pyramid_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| abandoned_camp_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| warm_ocean_ruins_map | MapItem | `steel-core/src/behavior/items/filled_map.rs:52` |
| shears | ShearsItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| melon_slice | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| dried_kelp | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pumpkin_seeds | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| melon_seeds | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| beef | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_beef | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chicken | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_chicken | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| rotten_flesh | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ender_pearl | EnderpearlItem | `steel-core/src/behavior/items/ender_pearl.rs:30` |
| blaze_rod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ghast_tear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_nugget | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_wart | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glass_bottle | BottleItem | `steel-core/src/behavior/items/bottle.rs:23` |
| potion | PotionItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spider_eye | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| fermented_spider_eye | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| blaze_powder | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| magma_cream | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| brewing_stand | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cauldron | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ender_eye | EnderEyeItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glistering_melon_slice | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chicken_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| cow_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| pig_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| sheep_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| camel_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| donkey_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| horse_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| mule_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| cat_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| parrot_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| wolf_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| armadillo_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| bat_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| bee_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| fox_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| goat_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| llama_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| ocelot_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| panda_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| polar_bear_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| rabbit_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| axolotl_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| cod_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| dolphin_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| frog_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| glow_squid_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| nautilus_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| pufferfish_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| salmon_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| squid_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| tadpole_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| tropical_fish_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| turtle_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| allay_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| mooshroom_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| sniffer_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| sulfur_cube_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| copper_golem_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| iron_golem_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| snow_golem_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| trader_llama_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| villager_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| wandering_trader_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| bogged_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| camel_husk_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| drowned_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| husk_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| parched_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| skeleton_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| skeleton_horse_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| stray_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| wither_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| wither_skeleton_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| zombie_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| zombie_horse_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| zombie_nautilus_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| zombie_villager_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| cave_spider_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| spider_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| breeze_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| creaking_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| creeper_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| elder_guardian_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| guardian_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| phantom_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| silverfish_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| slime_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| warden_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| witch_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| evoker_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| pillager_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| ravager_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| vindicator_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| vex_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| blaze_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| ghast_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| happy_ghast_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| hoglin_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| magma_cube_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| piglin_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| piglin_brute_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| strider_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| zoglin_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| zombified_piglin_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| ender_dragon_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| enderman_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| endermite_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| shulker_spawn_egg | SpawnEggItem | `steel-core/src/behavior/items/spawn_egg.rs:32` |
| experience_bottle | ExperienceBottleItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| fire_charge | FireChargeItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wind_charge | WindChargeItem | `steel-core/src/behavior/items/wind_charge.rs:31` |
| writable_book | WritableBookItem | `steel-core/src/behavior/items/books.rs:18` |
| written_book | WrittenBookItem | `steel-core/src/behavior/items/books.rs:35` |
| breeze_rod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| mace | MaceItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| item_frame | ItemFrameItem | `steel-core/src/behavior/items/item_frame.rs:24` |
| glow_item_frame | ItemFrameItem | `steel-core/src/behavior/items/item_frame.rs:24` |
| flower_pot | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| carrot | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| potato | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| baked_potato | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| poisonous_potato | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| map | EmptyMapItem | `steel-core/src/behavior/items/empty_map.rs:20` |
| golden_carrot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| skeleton_skull | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wither_skeleton_skull | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| player_head | PlayerHeadItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| zombie_head | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| creeper_head | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_head | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| piglin_head | StandingAndWallBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_star | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pumpkin_pie | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| firework_rocket | FireworkRocketItem | `steel-core/src/behavior/items/firework_rocket.rs:28` |
| firework_star | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| enchanted_book | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_shard | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_crystals | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| rabbit | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_rabbit | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| rabbit_stew | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| rabbit_foot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| rabbit_hide | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| armor_stand | ArmorStandItem | `steel-core/src/behavior/items/armor_stand.rs:42` |
| copper_horse_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_horse_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_horse_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_horse_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_horse_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| leather_horse_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| lead | LeadItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| name_tag | NameTagItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| command_block_minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| mutton | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_mutton | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_banner | BannerItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| end_crystal | EndCrystalItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chorus_fruit | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| popped_chorus_fruit | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| torchflower_seeds | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pitcher_pod | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| beetroot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| beetroot_seeds | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| beetroot_soup | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_breath | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| splash_potion | SplashPotionItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spectral_arrow | SpectralArrowItem | `steel-core/src/behavior/items/arrow.rs:59` |
| tipped_arrow | TippedArrowItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lingering_potion | LingeringPotionItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| shield | ShieldItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wooden_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| stone_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_spear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| totem_of_undying | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| shulker_shell | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_nugget | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_nugget | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| knowledge_book | KnowledgeBookItem | `steel-core/src/behavior/items/knowledge_book.rs:22` |
| debug_stick | DebugStickItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_13 | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_cat | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_blocks | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_bounce | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_chirp | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_creator | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_creator_music_box | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_far | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_lava_chicken | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_mall | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_mellohi | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_stal | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_strad | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_ward | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_11 | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_wait | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_otherside | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_relic | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_5 | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_pigstep | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_precipice | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| music_disc_tears | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| disc_fragment_5 | DiscFragmentItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| trident | TridentItem | `steel-core/src/behavior/items/trident.rs:47` |
| nautilus_shell | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| heart_of_the_sea | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| crossbow | CrossbowItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| suspicious_stew | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| loom | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| flower_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| creeper_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| skull_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| mojang_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| globe_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| piglin_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| flow_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| guster_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| field_masoned_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| bordure_indented_banner_pattern | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| goat_horn | InstrumentItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| composter | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| barrel | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smoker | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blast_furnace | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cartography_table | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| fletching_table | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| grindstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| smithing_table | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| stonecutter | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bell | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_lantern | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sweet_berries | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| glow_berries | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| campfire | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| soul_campfire | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| shroomlight | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| honeycomb | HoneycombItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bee_nest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| beehive | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| honey_bottle | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| honeycomb_block | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lodestone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| crying_obsidian | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gilded_blackstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_polished_blackstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_slab | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_stairs | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_polished_blackstone_bricks | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| respawn_anchor | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| orange_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| magenta_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_blue_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| yellow_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| lime_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pink_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| gray_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| light_gray_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| cyan_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| purple_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| blue_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| brown_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| green_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| black_candle | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| small_amethyst_bud | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| medium_amethyst_bud | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| large_amethyst_bud | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| amethyst_cluster | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pointed_dripstone | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_spike | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ochre_froglight | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| verdant_froglight | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| pearlescent_froglight | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| frogspawn | PlaceOnWaterBlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| echo_shard | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| brush | BrushItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_upgrade_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| sentry_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| dune_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| coast_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wild_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ward_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| eye_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| vex_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| tide_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| snout_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| rib_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| spire_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| wayfinder_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| shaper_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| silence_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| raiser_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| host_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| flow_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| bolt_armor_trim_smithing_template | SmithingTemplateItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| angler_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| archer_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| arms_up_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| blade_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| brewer_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| burn_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| danger_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| explorer_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| flow_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| friend_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| guster_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| heart_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| heartbreak_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| howl_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| miner_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| mourner_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| plenty_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| prize_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| scrape_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| sheaf_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| shelter_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| skull_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| snort_pottery_sherd | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_grate | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_bulb | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_chest | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| exposed_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| weathered_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| oxidized_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper_golem_statue | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| trial_spawner | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| trial_key | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ominous_trial_key | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| vault | BlockItem | Нет точной аннотации: проверить наследование/генератор/fallback |
| ominous_bottle | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
