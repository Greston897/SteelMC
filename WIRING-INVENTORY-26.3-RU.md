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
| FloatGoal | `steel-core/src/entity/ai/goal/float_goal.rs:6` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:97`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:94`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:77`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:78`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:111`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:86`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:86`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:126`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:142`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:120`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:82`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:114`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:86`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:80`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:78`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:87`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:86`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:82`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:80`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:85`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:117`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:105`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:166`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:87`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:82`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:99`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:80`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:88`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:86`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:105`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:101`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:82`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:86`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:104`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:78`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:87`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:90`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:80`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:82`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:86`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:82`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:117`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:87`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:110`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:105`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:104`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:98`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:113`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:98`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:116`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:98`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:94`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:98`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:98`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:98`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:92`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:98`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:107`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:106`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:104`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:184`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:114`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:109`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:145`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:118`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:163`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:114`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:98`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:100`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:98`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:113`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:118`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:130` |
| FollowMobGoal | `steel-core/src/entity/ai/goal/follow_mob.rs:10` | `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:122`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:126` |
| FollowParentGoal | `steel-core/src/entity/ai/goal/follow_parent.rs:11` | `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:113`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:107`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:101`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:115`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:100`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:101`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:97`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:101`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:101`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:101`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:95`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:101`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:120`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:107`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:199`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:129`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:132`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:160`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:137`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:166`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:116`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:100`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:102`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:100`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:116`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:121` |
| HurtByTargetGoal | `steel-core/src/entity/ai/goal/hurt_by_target.rs:12` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:104`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:102`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:84`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:85`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:136`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:93`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:93`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:133`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:149`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:143`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:89`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:121`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:93`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:87`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:85`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:94`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:93`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:89`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:87`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:93`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:123`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:130`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:173`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:94`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:89`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:106`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:87`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:95`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:94`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:130`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:107`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:89`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:93`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:129`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:85`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:96`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:97`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:87`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:89`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:93`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:89`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:124`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:94` |
| InteractGoal | `steel-core/src/entity/ai/goal/interact.rs:6` | `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:121` |
| LeapAtTargetGoal | `steel-core/src/entity/ai/goal/leap_at_target.rs:14` | `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:60`, `steel-core/src/entity/entities/mobs/passive/frog.rs:119` |
| LookAtPlayerGoal | `steel-core/src/entity/ai/goal/look_at_player.rs:19` | `steel-core/src/entity/ai/goal/interact.rs:1`, `steel-core/src/entity/ai/goal/interact.rs:7`, `steel-core/src/entity/ai/goal/interact.rs:14`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:100`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:98`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:80`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:81`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:36`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:65`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:132`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:89`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:89`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:129`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:145`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:72`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:135`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:85`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:117`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:89`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:83`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:81`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:90`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:89`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:85`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:83`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:89`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:119`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:31`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:59`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:126`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:169`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:90`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:85`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:102`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:83`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:91`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:90`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:31`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:59`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:126`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:103`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:85`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:89`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:30`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:58`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:125`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:81`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:92`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:93`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:83`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:85`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:89`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:85`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:120`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:90`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:120`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:107`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:109`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:103`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:117`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:102`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:123`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:147`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:103`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:99`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:103`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:103`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:103`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:97`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:103`, `steel-core/src/entity/entities/mobs/passive/villager.rs:26`, `steel-core/src/entity/entities/mobs/passive/villager.rs:125`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:122`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:109`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:201`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:131`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:134`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:162`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:136`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:139`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:168`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:107`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:118`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:107`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:107`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:102`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:102`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:128`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:131`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:144` |
| MeleeAttackGoal | `steel-core/src/entity/ai/goal/melee_attack.rs:22` | `steel-core/src/entity/entities/mobs/hostile/endermite.rs:19`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:96`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:78`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:79`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:87`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:11`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:87`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:127`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:143`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:34`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:133`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:83`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:115`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:27`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:87`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:81`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:79`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:88`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:87`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:83`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:81`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:86`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:167`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:88`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:83`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:81`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:89`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:87`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:83`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:87`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:79`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:90`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:91`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:81`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:83`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:87`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:83`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:118`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:88`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:111`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:70`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:143`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:141` |
| SilverfishMergeWithStoneGoal | `steel-core/src/entity/ai/goal/merge_with_stone.rs:29` | `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:9`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:30`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:88` |
| MoveToBlockGoal | `steel-core/src/entity/ai/goal/move_to_block.rs:18` | `steel-core/src/entity/ai/goal/sleep_in_bed.rs:18`, `steel-core/src/entity/ai/goal/sleep_in_bed.rs:35`, `steel-core/src/entity/ai/goal/sleep_in_bed.rs:45`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:7` |
| MoveTowardsRestrictionGoal | `steel-core/src/entity/ai/goal/move_towards_restriction.rs:9` | `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:115` |
| MoveTowardsTargetGoal | `steel-core/src/entity/ai/goal/move_towards_target.rs:9` | `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:28`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:87` |
| NearestAttackableTargetGoal | `steel-core/src/entity/ai/goal/nearest_attackable_target.rs:17` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:107`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:20`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:105`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:21`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:87`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:21`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:88`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:139`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:143`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:26`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:96`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:96`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:37`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:136`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:27`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:152`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:35`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:141`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:25`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:92`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:124`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:21`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:90`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:88`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:27`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:97`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:96`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:25`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:92`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:21`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:90`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:27`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:98`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:45`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:126`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:130`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:133`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:137`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:26`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:176`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:97`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:25`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:92`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:28`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:109`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:113`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:120`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:21`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:90`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:27`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:98`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:29`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:97`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:133`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:137`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:41`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:110`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:114`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:25`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:92`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:29`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:96`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:132`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:136`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:21`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:88`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:28`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:99`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:101`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:28`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:100`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:21`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:90`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:25`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:92`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:29`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:96`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:25`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:92`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:36`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:127`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:97` |
| ObtainRaidBannerGoal | `steel-core/src/entity/ai/goal/obtain_raid_banner.rs:11` | `steel-core/src/entity/entities/mobs/hostile/evoker.rs:78`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:78`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:97`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:78`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:85`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:78` |
| OpenDoorGoal | `steel-core/src/entity/ai/goal/open_door.rs:8` | `steel-core/src/entity/ai/goal/restrict_open_door.rs:7`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:89`, `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:116` |
| PanicGoal | `steel-core/src/entity/ai/goal/panic_goal.rs:14` | `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:25`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:112`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:105`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:142`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:99`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:114`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:99`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:54`, `steel-core/src/entity/entities/mobs/passive/frog.rs:117`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:99`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:95`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:99`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:99`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:99`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:93`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:99`, `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:108`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:26`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:119`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:105`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:185`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:115`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:110`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:146`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:30`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:119`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:134`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:107`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:187`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:470`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:41`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:182`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:42`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:175`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:27`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:130`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:48`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:174`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:164`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:115`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:104`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:99`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:101`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:99`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:25`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:114`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:119`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:131`, `steel-core/src/entity/mob/pathfinder.rs:320` |
| RaidBreakDoorGoal | `steel-core/src/entity/ai/goal/raid_break_door.rs:7` | `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:88` |
| RaidCelebrationGoal | `steel-core/src/entity/ai/goal/raid_celebration.rs:6` | `steel-core/src/entity/entities/mobs/hostile/evoker.rs:79`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:79`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:98`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:79`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:86`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:79` |
| RandomLookAroundGoal | `steel-core/src/entity/ai/goal/random_look_around.rs:11` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:26`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:101`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:20`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:99`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:22`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:81`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:22`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:82`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:133`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:27`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:90`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:90`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:38`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:130`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:28`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:146`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:35`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:136`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:26`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:86`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:29`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:118`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:28`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:90`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:22`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:84`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:82`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:91`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:90`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:26`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:86`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:22`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:84`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:28`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:90`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:46`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:120`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:127`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:27`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:170`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:91`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:26`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:86`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:29`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:103`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:84`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:28`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:92`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:30`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:91`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:127`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:42`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:104`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:26`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:86`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:90`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:126`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:22`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:82`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:93`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:29`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:94`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:22`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:84`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:26`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:86`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:90`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:26`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:86`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:37`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:121`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:91`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:26`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:122`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:108`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:28`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:110`, `steel-core/src/entity/entities/mobs/passive/bee.rs:30`, `steel-core/src/entity/entities/mobs/passive/bee.rs:104`, `steel-core/src/entity/entities/mobs/passive/camel.rs:33`, `steel-core/src/entity/entities/mobs/passive/camel.rs:118`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:26`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:103`, `steel-core/src/entity/entities/mobs/passive/frog.rs:30`, `steel-core/src/entity/entities/mobs/passive/frog.rs:125`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:148`, `steel-core/src/entity/entities/mobs/passive/goat.rs:30`, `steel-core/src/entity/entities/mobs/passive/goat.rs:104`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:26`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:100`, `steel-core/src/entity/entities/mobs/passive/panda.rs:30`, `steel-core/src/entity/entities/mobs/passive/panda.rs:104`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:30`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:104`, `steel-core/src/entity/entities/mobs/passive/strider.rs:30`, `steel-core/src/entity/entities/mobs/passive/strider.rs:104`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:24`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:98`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:30`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:104`, `steel-core/src/entity/entities/mobs/passive/villager.rs:27`, `steel-core/src/entity/entities/mobs/passive/villager.rs:126`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:27`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:110`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:202`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:132`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:33`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:135`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:41`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:163`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:31`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:137`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:140`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:169`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:108`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:38`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:119`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:108`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:108`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:105`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:26`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:103`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:130`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:146` |
| RandomStrollGoal | `steel-core/src/entity/ai/goal/random_stroll.rs:10` | `steel-core/src/entity/ai/goal/random_swimming.rs:5`, `steel-core/src/entity/ai/goal/random_swimming.rs:16`, `steel-core/src/entity/ai/goal/random_swimming.rs:23`, `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:4`, `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:11`, `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:24`, `steel-core/src/entity/entities/mobs/passive/frog.rs:31`, `steel-core/src/entity/entities/mobs/passive/frog.rs:56`, `steel-core/src/entity/entities/mobs/passive/frog.rs:120` |
| RandomSwimmingGoal | `steel-core/src/entity/ai/goal/random_swimming.rs:15` | `steel-core/src/entity/entities/mobs/passive/axolotl.rs:30`, `steel-core/src/entity/entities/mobs/passive/axolotl.rs:146`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:145`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/cod.rs:110`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:25`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:188`, `steel-core/src/entity/entities/mobs/passive/fish/nautilus.rs:471`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:41`, `steel-core/src/entity/entities/mobs/passive/fish/pufferfish.rs:185`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:42`, `steel-core/src/entity/entities/mobs/passive/fish/salmon.rs:178`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:27`, `steel-core/src/entity/entities/mobs/passive/fish/tadpole.rs:133`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:48`, `steel-core/src/entity/entities/mobs/passive/fish/tropical_fish.rs:177` |
| RangedAttackGoal | `steel-core/src/entity/ai/goal/ranged_attack.rs:23` | `steel-core/src/entity/ranged_attack.rs:3`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:25`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:98`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:9`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:125`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:10`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:119`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:10`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:119`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:9`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:118`, `steel-core/src/entity/mob/pathfinder.rs:106` |
| RestrictOpenDoorGoal | `steel-core/src/entity/ai/goal/restrict_open_door.rs:20` | `steel-core/src/entity/entities/mobs/passive/villager.rs:28`, `steel-core/src/entity/entities/mobs/passive/villager.rs:117`, `steel-core/src/entity/entities/mobs/passive/villager.rs:119` |
| RestrictSunGoal | `steel-core/src/entity/ai/goal/restrict_sun.rs:5` | `steel-core/src/entity/entities/mobs/hostile/bogged.rs:8`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:37`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:112`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:9`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:32`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:106`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:9`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:32`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:106`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:8`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:31`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:105` |
| SleepInBedGoal | `steel-core/src/entity/ai/goal/sleep_in_bed.rs:34` | `steel-core/src/entity/entities/mobs/passive/villager.rs:28`, `steel-core/src/entity/entities/mobs/passive/villager.rs:121`, `steel-core/src/entity/entities/mobs/passive/villager.rs:123` |
| SquidFleeGoal | `steel-core/src/entity/ai/goal/squid_flee_goal.rs:16` | `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:40`, `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:301`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:38`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:205` |
| SquidRandomMovementGoal | `steel-core/src/entity/ai/goal/squid_random_movement_goal.rs:25` | `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:32`, `steel-core/src/entity/entities/mobs/passive/squid/mod.rs:300`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:30`, `steel-core/src/entity/entities/mobs/passive/squid/glow.rs:204` |
| TemptGoal | `steel-core/src/entity/ai/goal/tempt_goal.rs:17` | `steel-core/src/entity/entities/mobs/passive/frog.rs:31`, `steel-core/src/entity/entities/mobs/passive/frog.rs:58`, `steel-core/src/entity/entities/mobs/passive/frog.rs:118`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:189`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:119`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:114`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:122`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:42`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:150`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:31`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:123`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:135`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:165`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:115`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:120`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:140` |
| TryFindLiquidGoal | `steel-core/src/entity/ai/goal/try_find_liquid.rs:14` | `steel-core/src/entity/entities/mobs/passive/dolphin.rs:31`, `steel-core/src/entity/entities/mobs/passive/dolphin.rs:142` |
| WaterAvoidingRandomStrollGoal | `steel-core/src/entity/ai/goal/water_avoiding_random_stroll.rs:10` | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:26`, `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:99`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:20`, `steel-core/src/entity/entities/mobs/hostile/endermite.rs:97`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:22`, `steel-core/src/entity/entities/mobs/hostile/giant.rs:79`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:22`, `steel-core/src/entity/entities/mobs/hostile/blaze.rs:80`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:38`, `steel-core/src/entity/entities/mobs/hostile/bogged.rs:131`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:27`, `steel-core/src/entity/entities/mobs/hostile/breeze.rs:88`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/cave_spider.rs:88`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:38`, `steel-core/src/entity/entities/mobs/hostile/copper_golem.rs:128`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:28`, `steel-core/src/entity/entities/mobs/hostile/creaking.rs:144`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:35`, `steel-core/src/entity/entities/mobs/hostile/creeper.rs:134`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:26`, `steel-core/src/entity/entities/mobs/hostile/drowned.rs:84`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:29`, `steel-core/src/entity/entities/mobs/hostile/elder_guardian.rs:116`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:28`, `steel-core/src/entity/entities/mobs/hostile/enderman.rs:88`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:22`, `steel-core/src/entity/entities/mobs/hostile/evoker.rs:82`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ghast.rs:80`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:28`, `steel-core/src/entity/entities/mobs/hostile/guardian.rs:89`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/hoglin.rs:88`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:26`, `steel-core/src/entity/entities/mobs/hostile/husk.rs:84`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:22`, `steel-core/src/entity/entities/mobs/hostile/illusioner.rs:82`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:28`, `steel-core/src/entity/entities/mobs/hostile/iron_golem.rs:88`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:46`, `steel-core/src/entity/entities/mobs/hostile/magma_cube.rs:118`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:33`, `steel-core/src/entity/entities/mobs/hostile/parched.rs:125`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:27`, `steel-core/src/entity/entities/mobs/hostile/phantom.rs:168`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/piglin.rs:89`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:26`, `steel-core/src/entity/entities/mobs/hostile/piglin_brute.rs:84`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:29`, `steel-core/src/entity/entities/mobs/hostile/pillager.rs:101`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:22`, `steel-core/src/entity/entities/mobs/hostile/ravager.rs:82`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:28`, `steel-core/src/entity/entities/mobs/hostile/shulker.rs:90`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:30`, `steel-core/src/entity/entities/mobs/hostile/silverfish.rs:89`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:33`, `steel-core/src/entity/entities/mobs/hostile/skeleton.rs:125`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:42`, `steel-core/src/entity/entities/mobs/hostile/slime.rs:102`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:26`, `steel-core/src/entity/entities/mobs/hostile/snow_golem.rs:84`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:30`, `steel-core/src/entity/entities/mobs/hostile/spider.rs:88`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:32`, `steel-core/src/entity/entities/mobs/hostile/stray.rs:124`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:22`, `steel-core/src/entity/entities/mobs/hostile/vex.rs:80`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:29`, `steel-core/src/entity/entities/mobs/hostile/vindicator.rs:91`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:29`, `steel-core/src/entity/entities/mobs/hostile/warden.rs:92`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:22`, `steel-core/src/entity/entities/mobs/hostile/witch.rs:82`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:26`, `steel-core/src/entity/entities/mobs/hostile/wither_skeleton.rs:84`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:30`, `steel-core/src/entity/entities/mobs/hostile/zoglin.rs:88`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:26`, `steel-core/src/entity/entities/mobs/hostile/zombie.rs:84`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:37`, `steel-core/src/entity/entities/mobs/hostile/zombie_villager.rs:119`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:31`, `steel-core/src/entity/entities/mobs/hostile/zombified_piglin.rs:89`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:26`, `steel-core/src/entity/entities/mobs/neutral/polar_bear.rs:116`, `steel-core/src/entity/entities/mobs/passive/allay.rs:28`, `steel-core/src/entity/entities/mobs/passive/allay.rs:106`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:29`, `steel-core/src/entity/entities/mobs/passive/armadillo.rs:108`, `steel-core/src/entity/entities/mobs/passive/bee.rs:31`, `steel-core/src/entity/entities/mobs/passive/bee.rs:102`, `steel-core/src/entity/entities/mobs/passive/camel.rs:34`, `steel-core/src/entity/entities/mobs/passive/camel.rs:116`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:27`, `steel-core/src/entity/entities/mobs/passive/camel_husk.rs:101`, `steel-core/src/entity/entities/mobs/passive/goat.rs:31`, `steel-core/src/entity/entities/mobs/passive/goat.rs:102`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:27`, `steel-core/src/entity/entities/mobs/passive/happy_ghast.rs:98`, `steel-core/src/entity/entities/mobs/passive/panda.rs:31`, `steel-core/src/entity/entities/mobs/passive/panda.rs:102`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:31`, `steel-core/src/entity/entities/mobs/passive/sniffer.rs:102`, `steel-core/src/entity/entities/mobs/passive/strider.rs:31`, `steel-core/src/entity/entities/mobs/passive/strider.rs:102`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:25`, `steel-core/src/entity/entities/mobs/passive/sulfur_cube.rs:96`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:31`, `steel-core/src/entity/entities/mobs/passive/turtle.rs:102`, `steel-core/src/entity/entities/mobs/passive/villager.rs:28`, `steel-core/src/entity/entities/mobs/passive/villager.rs:124`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:27`, `steel-core/src/entity/entities/mobs/passive/wandering_trader.rs:120`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:28`, `steel-core/src/entity/entities/mobs/passive/wolf.rs:108`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/chicken/mod.rs:200`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/cow/mod.rs:130`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:34`, `steel-core/src/entity/entities/mobs/passive/pig/mod.rs:133`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:42`, `steel-core/src/entity/entities/mobs/passive/mooshroom/mod.rs:161`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:31`, `steel-core/src/entity/entities/mobs/passive/sheep/mod.rs:135`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:35`, `steel-core/src/entity/entities/mobs/passive/cat/mod.rs:138`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:28`, `steel-core/src/entity/entities/mobs/passive/fox/mod.rs:167`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:28`, `steel-core/src/entity/entities/mobs/passive/horse/donkey.rs:106`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:39`, `steel-core/src/entity/entities/mobs/passive/horse/horse.rs:117`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:28`, `steel-core/src/entity/entities/mobs/passive/horse/llama.rs:106`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:28`, `steel-core/src/entity/entities/mobs/passive/horse/mule.rs:106`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/skeleton_horse.rs:101`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/trader_llama.rs:103`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:27`, `steel-core/src/entity/entities/mobs/passive/horse/zombie_horse.rs:101`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:26`, `steel-core/src/entity/entities/mobs/passive/ocelot/mod.rs:117`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:27`, `steel-core/src/entity/entities/mobs/passive/parrot/mod.rs:128`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:29`, `steel-core/src/entity/entities/mobs/passive/rabbit/mod.rs:141` |

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
| item | ItemEntity | `steel-core/src/entity/entities/objects/items/item_entity.rs:104` |
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
| wither | WitherBoss | `steel-core/src/entity/entities/mobs/bosses/wither_boss.rs:50` |
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
| air | AirBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:29` |
| stone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| granite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_granite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| diorite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_diorite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| andesite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_andesite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| grass_block | GrassBlock | `steel-core/src/behavior/blocks/vegetation/grass_block.rs:17` |
| dirt | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| coarse_dirt | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| podzol | SnowyBlock | `steel-core/src/behavior/blocks/vegetation/snowy_block.rs:45` |
| cobblestone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| birch_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| jungle_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| acacia_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cherry_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_oak_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pale_oak_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| pale_oak_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| mangrove_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| poplar_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bamboo_mosaic | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| oak_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| spruce_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| birch_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| jungle_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| acacia_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| cherry_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| dark_oak_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| pale_oak_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| mangrove_propagule | MangrovePropaguleBlock | `steel-core/src/behavior/blocks/vegetation/mangrove_propagule_block.rs:27` |
| poplar_sapling | SaplingBlock | `steel-core/src/behavior/blocks/vegetation/sapling_block.rs:35` |
| bedrock | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| water | LiquidBlock | `steel-core/src/behavior/blocks/fluid/liquid_block.rs:37` |
| lava | LiquidBlock | `steel-core/src/behavior/blocks/fluid/liquid_block.rs:37` |
| sand | SandBlock | `steel-core/src/behavior/blocks/falling/sand_block.rs:16` |
| suspicious_sand | BrushableBlock | `steel-core/src/behavior/blocks/building/brushable_block.rs:26` |
| red_sand | SandBlock | `steel-core/src/behavior/blocks/falling/sand_block.rs:16` |
| gravel | ColoredFallingBlock | `steel-core/src/behavior/blocks/falling/falling_block.rs:19` |
| suspicious_gravel | BrushableBlock | `steel-core/src/behavior/blocks/building/brushable_block.rs:26` |
| gold_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_gold_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| iron_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_iron_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| coal_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_coal_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| nether_gold_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| oak_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| spruce_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| birch_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| jungle_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| acacia_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| cherry_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| dark_oak_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| pale_oak_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| mangrove_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| poplar_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| mangrove_roots | MangroveRootsBlock | `steel-core/src/behavior/blocks/vegetation/mangrove_roots_block.rs:13` |
| muddy_mangrove_roots | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| bamboo_block | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_spruce_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_birch_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_jungle_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_acacia_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_cherry_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_dark_oak_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_pale_oak_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_oak_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_mangrove_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_poplar_log | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_bamboo_block | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| oak_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| spruce_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| birch_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| jungle_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| acacia_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| cherry_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| dark_oak_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| mangrove_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| poplar_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_oak_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_spruce_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_birch_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_jungle_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_acacia_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_cherry_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_dark_oak_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_pale_oak_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_mangrove_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_poplar_wood | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| oak_leaves | TintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:175` |
| spruce_leaves | LeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:33` |
| birch_leaves | TintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:175` |
| jungle_leaves | TintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:175` |
| acacia_leaves | TintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:175` |
| cherry_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| dark_oak_leaves | TintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:175` |
| pale_oak_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| mangrove_leaves | MangroveLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:218` |
| red_poplar_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| orange_poplar_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| yellow_poplar_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| azalea_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| flowering_azalea_leaves | UntintedParticleLeavesBlock | `steel-core/src/behavior/blocks/vegetation/leaves_block.rs:133` |
| sponge | SpongeBlock | `steel-core/src/behavior/blocks/building/sponge_block.rs:23` |
| wet_sponge | WetSpongeBlock | `steel-core/src/behavior/blocks/building/wet_sponge_block.rs:12` |
| glass | TransparentBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:50` |
| lapis_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_lapis_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| lapis_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dispenser | DispenserBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:256` |
| sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| note_block | NoteBlock | `steel-core/src/behavior/blocks/redstone/note_block.rs:29` |
| white_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| orange_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| magenta_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| light_blue_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| yellow_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| lime_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| pink_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| gray_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| light_gray_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| cyan_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| purple_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| blue_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| brown_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| green_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| red_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| black_bed | BedBlock | `steel-core/src/behavior/blocks/building/bed_block.rs:29` |
| straw_bed | StrawBedBlock | `steel-core/src/behavior/blocks/building/straw_bed_block.rs:33` |
| powered_rail | PoweredRailBlock | `steel-core/src/behavior/blocks/redstone/rail/powered_rail_block.rs:18` |
| detector_rail | DetectorRailBlock | `steel-core/src/behavior/blocks/redstone/rail/detector_rail_block.rs:19` |
| sticky_piston | PistonBaseBlock | `steel-core/src/behavior/blocks/redstone/piston/base.rs:39` |
| cobweb | WebBlock | `steel-core/src/behavior/blocks/building/web_block.rs:12` |
| short_grass | TallGrassBlock | `steel-core/src/behavior/blocks/vegetation/tall_grass_block.rs:31` |
| fern | TallGrassBlock | `steel-core/src/behavior/blocks/vegetation/tall_grass_block.rs:31` |
| dead_bush | DryVegetationBlock | `steel-core/src/behavior/blocks/vegetation/dry_vegetation_block.rs:14` |
| bush | BushBlock | `steel-core/src/behavior/blocks/vegetation/bush_block.rs:22` |
| red_shrub | BushBlock | `steel-core/src/behavior/blocks/vegetation/bush_block.rs:22` |
| short_dry_grass | ShortDryGrassBlock | `steel-core/src/behavior/blocks/vegetation/short_dry_grass_block.rs:19` |
| tall_dry_grass | TallDryGrassBlock | `steel-core/src/behavior/blocks/vegetation/tall_dry_grass_block.rs:22` |
| seagrass | SeagrassBlock | `steel-core/src/behavior/blocks/vegetation/seagrass_block.rs:21` |
| tall_seagrass | TallSeagrassBlock | `steel-core/src/behavior/blocks/vegetation/tall_seagrass_block.rs:23` |
| piston | PistonBaseBlock | `steel-core/src/behavior/blocks/redstone/piston/base.rs:39` |
| piston_head | PistonHeadBlock | `steel-core/src/behavior/blocks/redstone/piston/head.rs:21` |
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
| white_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| orange_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| magenta_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| light_blue_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| yellow_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| lime_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| pink_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| gray_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| light_gray_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cyan_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| purple_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| blue_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| brown_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| green_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| red_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| black_wool_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| white_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| orange_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| magenta_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| light_blue_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| yellow_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| lime_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| pink_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| gray_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| light_gray_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cyan_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| purple_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| blue_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| brown_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| green_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| red_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| black_wool_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| moving_piston | MovingPistonBlock | `steel-core/src/behavior/blocks/redstone/piston/moving.rs:27` |
| dandelion | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| golden_dandelion | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| torchflower | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| poppy | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| blue_orchid | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| allium | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| azure_bluet | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| red_tulip | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| orange_tulip | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| white_tulip | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| pink_tulip | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| oxeye_daisy | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| cornflower | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| wither_rose | WitherRoseBlock | `steel-core/src/behavior/blocks/vegetation/wither_rose_block.rs:18` |
| lily_of_the_valley | FlowerBlock | `steel-core/src/behavior/blocks/vegetation/flower_block.rs:17` |
| brown_mushroom | MushroomBlock | `steel-core/src/behavior/blocks/vegetation/mushroom_block.rs:31` |
| red_mushroom | MushroomBlock | `steel-core/src/behavior/blocks/vegetation/mushroom_block.rs:31` |
| gold_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tnt | TntBlock | `steel-core/src/behavior/blocks/redstone/tnt_block.rs:31` |
| bookshelf | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_bookshelf | ChiseledBookShelfBlock | `steel-core/src/behavior/blocks/container/chiseled_bookshelf_block.rs:37` |
| acacia_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| bamboo_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| birch_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| cherry_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| crimson_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| dark_oak_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| jungle_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| mangrove_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| oak_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| pale_oak_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| poplar_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| spruce_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| warped_shelf | ShelfBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1200` |
| mossy_cobblestone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| obsidian | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| torch | TorchBlock | `steel-core/src/behavior/blocks/decoration/torch_block.rs:26` |
| wall_torch | WallTorchBlock | `steel-core/src/behavior/blocks/decoration/torch_block.rs:81` |
| fire | FireBlock | `steel-core/src/behavior/blocks/portal/fire.rs:30` |
| soul_fire | SoulFireBlock | `steel-core/src/behavior/blocks/portal/fire.rs:478` |
| spawner | SpawnerBlock | `steel-core/src/behavior/blocks/building/spawner_block.rs:17` |
| creaking_heart | CreakingHeartBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:507` |
| oak_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| chest | ChestBlock | `steel-core/src/behavior/blocks/container/chest_block.rs:34` |
| redstone_wire | RedstoneWireBlock | `steel-core/src/behavior/blocks/redstone/wire/block.rs:31` |
| diamond_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_diamond_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| diamond_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| crafting_table | CraftingTableBlock | `steel-core/src/behavior/blocks/container/crafting_table_block.rs:23` |
| wheat | CropBlock | `steel-core/src/behavior/blocks/vegetation/crop_block.rs:39` |
| farmland | FarmlandBlock | `steel-core/src/behavior/blocks/vegetation/farmland_block.rs:28` |
| furnace | FurnaceBlock | `steel-core/src/behavior/blocks/container/furnace_block.rs:27` |
| oak_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| spruce_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| birch_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| acacia_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| cherry_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| jungle_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| dark_oak_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| pale_oak_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| mangrove_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| poplar_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| bamboo_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| oak_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| ladder | LadderBlock | `steel-core/src/behavior/blocks/building/ladder_block.rs:17` |
| rail | RailBlock | `steel-core/src/behavior/blocks/redstone/rail/rail_block.rs:15` |
| cobblestone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| oak_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| spruce_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| birch_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| acacia_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| cherry_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| jungle_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| dark_oak_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| pale_oak_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| mangrove_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| poplar_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| bamboo_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| oak_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| spruce_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| birch_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| acacia_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| cherry_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| jungle_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| dark_oak_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| pale_oak_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| crimson_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| warped_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| mangrove_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| poplar_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| bamboo_hanging_sign | CeilingHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:569` |
| oak_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| spruce_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| birch_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| acacia_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| cherry_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| jungle_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| dark_oak_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| pale_oak_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| mangrove_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| poplar_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| crimson_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| warped_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| bamboo_wall_hanging_sign | WallHangingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:737` |
| lever | LeverBlock | `steel-core/src/behavior/blocks/redstone/lever_block.rs:23` |
| stone_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| iron_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| oak_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| spruce_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| birch_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| jungle_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| acacia_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| cherry_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| dark_oak_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| pale_oak_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| mangrove_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| poplar_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| bamboo_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| redstone_ore | RedStoneOreBlock | `steel-core/src/behavior/blocks/redstone/redstone_ore_block.rs:24` |
| deepslate_redstone_ore | RedStoneOreBlock | `steel-core/src/behavior/blocks/redstone/redstone_ore_block.rs:24` |
| redstone_torch | RedstoneTorchBlock | `steel-core/src/behavior/blocks/redstone/redstone_torch_block.rs:89` |
| redstone_wall_torch | RedstoneWallTorchBlock | `steel-core/src/behavior/blocks/redstone/redstone_torch_block.rs:242` |
| stone_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| snow | SnowLayerBlock | `steel-core/src/behavior/blocks/building/snow_layer_block.rs:26` |
| ice | IceBlock | `steel-core/src/behavior/blocks/building/ice_block.rs:19` |
| snow_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cactus | CactusBlock | `steel-core/src/behavior/blocks/vegetation/cactus_block.rs:44` |
| cactus_flower | CactusFlowerBlock | `steel-core/src/behavior/blocks/vegetation/cactus_flower_block.rs:20` |
| clay | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sugar_cane | SugarCaneBlock | `steel-core/src/behavior/blocks/vegetation/sugar_cane.rs:25` |
| jukebox | JukeboxBlock | `steel-core/src/behavior/blocks/decoration/jukebox_block.rs:28` |
| oak_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| netherrack | NetherrackBlock | `steel-core/src/behavior/blocks/building/netherrack_block.rs:21` |
| soul_sand | SoulSandBlock | `steel-core/src/behavior/blocks/building/soul_sand_block.rs:13` |
| soul_soil | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| basalt | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| polished_basalt | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| soul_torch | TorchBlock | `steel-core/src/behavior/blocks/decoration/torch_block.rs:26` |
| soul_wall_torch | WallTorchBlock | `steel-core/src/behavior/blocks/decoration/torch_block.rs:81` |
| copper_torch | TorchBlock | `steel-core/src/behavior/blocks/decoration/torch_block.rs:26` |
| copper_wall_torch | WallTorchBlock | `steel-core/src/behavior/blocks/decoration/torch_block.rs:81` |
| glowstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_portal | NetherPortalBlock | `steel-core/src/behavior/blocks/portal/nether_portal_block.rs:20` |
| carved_pumpkin | CarvedPumpkinBlock | `steel-core/src/behavior/blocks/vegetation/carved_pumpkin_block.rs:19` |
| jack_o_lantern | CarvedPumpkinBlock | `steel-core/src/behavior/blocks/vegetation/carved_pumpkin_block.rs:19` |
| cake | CakeBlock | `steel-core/src/behavior/blocks/decoration/cake_block.rs:29` |
| repeater | RepeaterBlock | `steel-core/src/behavior/blocks/redstone/diode/repeater.rs:25` |
| white_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| orange_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| magenta_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| light_blue_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| yellow_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| lime_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| pink_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| gray_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| light_gray_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| cyan_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| purple_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| blue_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| brown_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| green_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| red_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| black_stained_glass | StainedGlassBlock | `steel-core/src/behavior/blocks/colored/stained_glass_block.rs:17` |
| oak_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| spruce_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| birch_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| jungle_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| acacia_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| cherry_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| dark_oak_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| pale_oak_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| mangrove_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| poplar_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| bamboo_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| mossy_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| packed_mud | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| mud_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_stone | InfestedBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:36` |
| infested_cobblestone | InfestedBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:36` |
| infested_stone_bricks | InfestedBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:36` |
| infested_mossy_stone_bricks | InfestedBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:36` |
| infested_cracked_stone_bricks | InfestedBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:36` |
| infested_chiseled_stone_bricks | InfestedBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:36` |
| brown_mushroom_block | HugeMushroomBlock | `steel-core/src/behavior/blocks/vegetation/huge_mushroom_block.rs:14` |
| red_mushroom_block | HugeMushroomBlock | `steel-core/src/behavior/blocks/vegetation/huge_mushroom_block.rs:14` |
| mushroom_stem | HugeMushroomBlock | `steel-core/src/behavior/blocks/vegetation/huge_mushroom_block.rs:14` |
| iron_bars | IronBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:28` |
| copper_bars | WeatheringCopperBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:89` |
| exposed_copper_bars | WeatheringCopperBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:89` |
| weathered_copper_bars | WeatheringCopperBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:89` |
| oxidized_copper_bars | WeatheringCopperBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:89` |
| waxed_copper_bars | IronBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:28` |
| waxed_exposed_copper_bars | IronBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:28` |
| waxed_weathered_copper_bars | IronBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:28` |
| waxed_oxidized_copper_bars | IronBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:28` |
| iron_chain | ChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:24` |
| copper_chain | WeatheringCopperChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:77` |
| exposed_copper_chain | WeatheringCopperChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:77` |
| weathered_copper_chain | WeatheringCopperChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:77` |
| oxidized_copper_chain | WeatheringCopperChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:77` |
| waxed_copper_chain | ChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:24` |
| waxed_exposed_copper_chain | ChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:24` |
| waxed_weathered_copper_chain | ChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:24` |
| waxed_oxidized_copper_chain | ChainBlock | `steel-core/src/behavior/blocks/decoration/chain_block.rs:24` |
| glass_pane | IronBarsBlock | `steel-core/src/behavior/blocks/building/bar_block.rs:28` |
| pumpkin | PumpkinBlock | `steel-core/src/behavior/blocks/vegetation/pumpkin_block.rs:25` |
| melon | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| attached_pumpkin_stem | AttachedStemBlock | `steel-core/src/behavior/blocks/vegetation/attached_stem_block.rs:30` |
| attached_melon_stem | AttachedStemBlock | `steel-core/src/behavior/blocks/vegetation/attached_stem_block.rs:30` |
| pumpkin_stem | StemBlock | `steel-core/src/behavior/blocks/vegetation/stem_block.rs:35` |
| melon_stem | StemBlock | `steel-core/src/behavior/blocks/vegetation/stem_block.rs:35` |
| vine | VineBlock | `steel-core/src/behavior/blocks/vegetation/vine_block.rs:18` |
| glow_lichen | GlowLichenBlock | `steel-core/src/behavior/blocks/vegetation/glow_lichen_block.rs:17` |
| resin_clump | MultifaceBlock | `steel-core/src/behavior/blocks/vegetation/multiface_block.rs:20` |
| oak_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| stone_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| mud_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| mycelium | MyceliumBlock | `steel-core/src/behavior/blocks/vegetation/mycelium_block.rs:14` |
| lily_pad | LilyPadBlock | `steel-core/src/behavior/blocks/vegetation/lily_pad_block.rs:19` |
| resin_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| resin_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| resin_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| resin_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_resin_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_brick_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| nether_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| nether_wart | NetherWartBlock | `steel-core/src/behavior/blocks/vegetation/nether_wart.rs:31` |
| enchanting_table | EnchantingTableBlock | `steel-core/src/behavior/blocks/container/enchanting_table_block.rs:25` |
| brewing_stand | BrewingStandBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:856` |
| cauldron | CauldronBlock | `steel-core/src/behavior/blocks/building/cauldron_block.rs:23` |
| water_cauldron | LayeredCauldronBlock | `steel-core/src/behavior/blocks/building/cauldron_block.rs:107` |
| lava_cauldron | LavaCauldronBlock | `steel-core/src/behavior/blocks/building/lava_cauldron_block.rs:31` |
| powder_snow_cauldron | LayeredCauldronBlock | `steel-core/src/behavior/blocks/building/cauldron_block.rs:107` |
| end_portal | EndPortalBlock | `steel-core/src/behavior/blocks/portal/end_portal_block.rs:23` |
| end_portal_frame | EndPortalFrameBlock | `steel-core/src/behavior/blocks/portal/end_portal_frame_block.rs:17` |
| end_stone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_egg | DragonEggBlock | `steel-core/src/behavior/blocks/falling/dragon_egg_block.rs:26` |
| redstone_lamp | RedstoneLampBlock | `steel-core/src/behavior/blocks/redstone/redstone_lamp_block.rs:18` |
| cocoa | CocoaBlock | `steel-core/src/behavior/blocks/vegetation/cocoa_block.rs:31` |
| shelf_mushroom | ShelfMushroomBlock | `steel-core/src/behavior/blocks/vegetation/shelf_mushroom_block.rs:29` |
| sandstone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| emerald_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_emerald_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| ender_chest | EnderChestBlock | `steel-core/src/behavior/blocks/container/ender_chest_block.rs:22` |
| tripwire_hook | TripWireHookBlock | `steel-core/src/behavior/blocks/redstone/tripwire/hook.rs:25` |
| tripwire | TripWireBlock | `steel-core/src/behavior/blocks/redstone/tripwire/block.rs:28` |
| emerald_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| birch_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| jungle_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| command_block | CommandBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:55` |
| beacon | BeaconBlock | `steel-core/src/behavior/blocks/container/beacon_block.rs:24` |
| cobblestone_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| mossy_cobblestone_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| flower_pot | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_torchflower | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_oak_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_spruce_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_birch_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_jungle_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_acacia_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_cherry_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_dark_oak_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_pale_oak_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_poplar_sapling | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_mangrove_propagule | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_fern | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_dandelion | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_golden_dandelion | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_poppy | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_blue_orchid | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_allium | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_azure_bluet | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_red_tulip | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_orange_tulip | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_white_tulip | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_pink_tulip | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_oxeye_daisy | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_cornflower | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_lily_of_the_valley | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_wither_rose | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_red_mushroom | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_brown_mushroom | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_dead_bush | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_cactus | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| carrots | CarrotBlock | `steel-core/src/behavior/blocks/vegetation/carrot.rs:24` |
| potatoes | PotatoBlock | `steel-core/src/behavior/blocks/vegetation/potato.rs:24` |
| oak_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| spruce_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| birch_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| jungle_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| acacia_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| cherry_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| dark_oak_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| pale_oak_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| mangrove_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| poplar_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| bamboo_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| skeleton_skull | SkullBlock | `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs:18` |
| skeleton_wall_skull | WallSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/wall_skull_block.rs:19` |
| wither_skeleton_skull | WitherSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/wither_skull_block.rs:19` |
| wither_skeleton_wall_skull | WitherWallSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/wither_wall_skull_block.rs:14` |
| zombie_head | SkullBlock | `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs:18` |
| zombie_wall_head | WallSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/wall_skull_block.rs:19` |
| player_head | PlayerHeadBlock | `steel-core/src/behavior/blocks/decoration/skull/player_head_block.rs:14` |
| player_wall_head | PlayerWallHeadBlock | `steel-core/src/behavior/blocks/decoration/skull/player_wall_head_block.rs:14` |
| creeper_head | SkullBlock | `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs:18` |
| creeper_wall_head | WallSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/wall_skull_block.rs:19` |
| dragon_head | SkullBlock | `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs:18` |
| dragon_wall_head | WallSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/wall_skull_block.rs:19` |
| piglin_head | SkullBlock | `steel-core/src/behavior/blocks/decoration/skull/skull_block.rs:18` |
| piglin_wall_head | PiglinWallSkullBlock | `steel-core/src/behavior/blocks/decoration/skull/piglin_wall_skull_block.rs:14` |
| anvil | AnvilBlock | `steel-core/src/behavior/blocks/container/anvil_block.rs:24` |
| chipped_anvil | AnvilBlock | `steel-core/src/behavior/blocks/container/anvil_block.rs:24` |
| damaged_anvil | AnvilBlock | `steel-core/src/behavior/blocks/container/anvil_block.rs:24` |
| trapped_chest | TrappedChestBlock | `steel-core/src/behavior/blocks/container/chest_block.rs:237` |
| light_weighted_pressure_plate | WeightedPressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/weighted.rs:21` |
| heavy_weighted_pressure_plate | WeightedPressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/weighted.rs:21` |
| comparator | ComparatorBlock | `steel-core/src/behavior/blocks/redstone/diode/comparator.rs:32` |
| daylight_detector | DaylightDetectorBlock | `steel-core/src/behavior/blocks/redstone/daylight_detector_block.rs:27` |
| redstone_block | PoweredBlock | `steel-core/src/behavior/blocks/redstone/powered_block.rs:13` |
| nether_quartz_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| hopper | HopperBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:132` |
| quartz_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_quartz_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_pillar | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| quartz_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| activator_rail | PoweredRailBlock | `steel-core/src/behavior/blocks/redstone/rail/powered_rail_block.rs:18` |
| dropper | DropperBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:270` |
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
| white_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| orange_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| magenta_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| light_blue_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| yellow_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| lime_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| pink_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| gray_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| light_gray_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| cyan_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| purple_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| blue_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| brown_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| green_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| red_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| black_stained_glass_pane | StainedGlassPaneBlock | `steel-core/src/behavior/blocks/colored/stained_glass_pane_block.rs:19` |
| acacia_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cherry_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| dark_oak_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| pale_oak_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| mangrove_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| poplar_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| bamboo_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| bamboo_mosaic_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| slime_block | SlimeBlock | `steel-core/src/behavior/blocks/building/slime_block.rs:15` |
| barrier | BarrierBlock | `steel-core/src/behavior/blocks/building/barrier_block.rs:16` |
| light | LightBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:157` |
| iron_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| prismarine | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dark_prismarine | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| prismarine_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| prismarine_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| dark_prismarine_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| prismarine_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| prismarine_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| dark_prismarine_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| sea_lantern | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| hay_block | HayBlock | `steel-core/src/behavior/blocks/building/hay_block.rs:19` |
| white_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| orange_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| magenta_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| light_blue_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| yellow_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| lime_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| pink_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| gray_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| light_gray_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| cyan_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| purple_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| blue_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| brown_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| green_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| red_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| black_carpet | WoolCarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:51` |
| terracotta | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| coal_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| packed_ice | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sunflower | TallFlowerBlock | `steel-core/src/behavior/blocks/vegetation/tall_flower_block.rs:17` |
| lilac | TallFlowerBlock | `steel-core/src/behavior/blocks/vegetation/tall_flower_block.rs:17` |
| rose_bush | TallFlowerBlock | `steel-core/src/behavior/blocks/vegetation/tall_flower_block.rs:17` |
| peony | TallFlowerBlock | `steel-core/src/behavior/blocks/vegetation/tall_flower_block.rs:17` |
| tall_grass | DoublePlantBlock | `steel-core/src/behavior/blocks/vegetation/double_plant_block.rs:22` |
| large_fern | DoublePlantBlock | `steel-core/src/behavior/blocks/vegetation/double_plant_block.rs:22` |
| white_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| orange_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| magenta_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| light_blue_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| yellow_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| lime_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| pink_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| gray_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| light_gray_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| cyan_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| purple_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| blue_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| brown_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| green_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| red_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| black_banner | BannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:21` |
| white_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| orange_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| magenta_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| light_blue_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| yellow_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| lime_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| pink_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| gray_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| light_gray_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| cyan_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| purple_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| blue_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| brown_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| green_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| red_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| black_wall_banner | WallBannerBlock | `steel-core/src/behavior/blocks/decoration/banner_block.rs:81` |
| red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_sandstone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| oak_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| spruce_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| birch_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| jungle_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| acacia_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cherry_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| dark_oak_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| pale_oak_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| mangrove_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| poplar_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| bamboo_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| bamboo_mosaic_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| stone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| sandstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cut_sandstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| petrified_oak_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cobblestone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| stone_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| mud_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| nether_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| quartz_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| red_sandstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cut_red_sandstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| smooth_stone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_stone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| smooth_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_quartz | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| smooth_red_sandstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| spruce_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| birch_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| jungle_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| acacia_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| cherry_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| dark_oak_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| pale_oak_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| mangrove_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| poplar_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| bamboo_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| spruce_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| birch_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| jungle_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| acacia_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| cherry_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| dark_oak_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| pale_oak_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| mangrove_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| poplar_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| bamboo_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| spruce_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| birch_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| jungle_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| acacia_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| cherry_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| dark_oak_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| pale_oak_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| mangrove_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| poplar_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| bamboo_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| end_rod | EndRodBlock | `steel-core/src/behavior/blocks/decoration/end_rod_block.rs:14` |
| chorus_plant | ChorusPlantBlock | `steel-core/src/behavior/blocks/vegetation/chorus_plant_block.rs:25` |
| chorus_flower | ChorusFlowerBlock | `steel-core/src/behavior/blocks/vegetation/chorus_flower_block.rs:26` |
| purpur_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| purpur_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| purpur_pillar | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| purpur_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| end_stone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| torchflower_crop | TorchflowerCropBlock | `steel-core/src/behavior/blocks/vegetation/torchflower.rs:24` |
| pitcher_crop | PitcherCropBlock | `steel-core/src/behavior/blocks/vegetation/pitcher_crop.rs:39` |
| pitcher_plant | DoublePlantBlock | `steel-core/src/behavior/blocks/vegetation/double_plant_block.rs:22` |
| beetroots | BeetrootBlock | `steel-core/src/behavior/blocks/vegetation/beetroots.rs:24` |
| dirt_path | PathBlock | `steel-core/src/behavior/blocks/vegetation/dirt_path_block.rs:18` |
| end_gateway | EndGatewayBlock | `steel-core/src/behavior/blocks/portal/end_portal_block.rs:116` |
| repeating_command_block | CommandBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:55` |
| chain_command_block | CommandBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:55` |
| frosted_ice | FrostedIceBlock | `steel-core/src/behavior/blocks/building/frosted_ice_block.rs:30` |
| magma_block | MagmaBlock | `steel-core/src/behavior/blocks/building/magma_block.rs:16` |
| nether_wart_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| red_nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| bone_block | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| structure_void | StructureVoidBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:111` |
| observer | ObserverBlock | `steel-core/src/behavior/blocks/redstone/observer_block.rs:23` |
| shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| white_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| orange_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| magenta_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| light_blue_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| yellow_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| lime_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| pink_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| gray_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| light_gray_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| cyan_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| purple_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| blue_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| brown_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| green_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| red_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| black_shulker_box | ShulkerBoxBlock | `steel-core/src/behavior/blocks/container/shulker_box_block.rs:38` |
| white_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| orange_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| magenta_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| light_blue_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| yellow_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| lime_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| pink_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| gray_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| light_gray_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| cyan_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| purple_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| blue_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| brown_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| green_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| red_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
| black_glazed_terracotta | GlazedTerracottaBlock | `steel-core/src/behavior/blocks/building/glazed_terracotta_block.rs:13` |
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
| white_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| orange_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| magenta_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| light_blue_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| yellow_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| lime_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| pink_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| gray_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| light_gray_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cyan_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| purple_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| blue_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| brown_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| green_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| red_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| black_concrete_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| white_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| orange_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| magenta_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| light_blue_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| yellow_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| lime_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| pink_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| gray_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| light_gray_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cyan_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| purple_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| blue_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| brown_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| green_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| red_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| black_concrete_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| white_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| orange_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| magenta_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| light_blue_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| yellow_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| lime_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| pink_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| gray_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| light_gray_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| cyan_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| purple_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| blue_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| brown_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| green_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| red_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| black_concrete_powder | ConcretePowderBlock | `steel-core/src/behavior/blocks/falling/concrete_powder_block.rs:16` |
| kelp | KelpBlock | `steel-core/src/behavior/blocks/vegetation/kelp_block.rs:26` |
| kelp_plant | KelpPlantBlock | `steel-core/src/behavior/blocks/vegetation/kelp_plant_block.rs:23` |
| dried_kelp_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| turtle_egg | TurtleEggBlock | `steel-core/src/behavior/blocks/vegetation/turtle_egg_block.rs:65` |
| sniffer_egg | SnifferEggBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:348` |
| dried_ghast | DriedGhastBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:398` |
| dead_tube_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_brain_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_bubble_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_fire_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| dead_horn_coral_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tube_coral_block | CoralBlock | `steel-core/src/behavior/blocks/vegetation/coral_block.rs:16` |
| brain_coral_block | CoralBlock | `steel-core/src/behavior/blocks/vegetation/coral_block.rs:16` |
| bubble_coral_block | CoralBlock | `steel-core/src/behavior/blocks/vegetation/coral_block.rs:16` |
| fire_coral_block | CoralBlock | `steel-core/src/behavior/blocks/vegetation/coral_block.rs:16` |
| horn_coral_block | CoralBlock | `steel-core/src/behavior/blocks/vegetation/coral_block.rs:16` |
| dead_tube_coral | BaseCoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs:18` |
| dead_brain_coral | BaseCoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs:18` |
| dead_bubble_coral | BaseCoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs:18` |
| dead_fire_coral | BaseCoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs:18` |
| dead_horn_coral | BaseCoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_plant_block.rs:18` |
| tube_coral | CoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs:19` |
| brain_coral | CoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs:19` |
| bubble_coral | CoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs:19` |
| fire_coral | CoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs:19` |
| horn_coral | CoralPlantBlock | `steel-core/src/behavior/blocks/vegetation/coral_plant_block.rs:19` |
| dead_tube_coral_fan | BaseCoralFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs:17` |
| dead_brain_coral_fan | BaseCoralFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs:17` |
| dead_bubble_coral_fan | BaseCoralFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs:17` |
| dead_fire_coral_fan | BaseCoralFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs:17` |
| dead_horn_coral_fan | BaseCoralFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_fan_block.rs:17` |
| tube_coral_fan | CoralFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs:19` |
| brain_coral_fan | CoralFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs:19` |
| bubble_coral_fan | CoralFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs:19` |
| fire_coral_fan | CoralFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs:19` |
| horn_coral_fan | CoralFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_fan_block.rs:19` |
| dead_tube_coral_wall_fan | BaseCoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs:16` |
| dead_brain_coral_wall_fan | BaseCoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs:16` |
| dead_bubble_coral_wall_fan | BaseCoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs:16` |
| dead_fire_coral_wall_fan | BaseCoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs:16` |
| dead_horn_coral_wall_fan | BaseCoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/base_coral_wall_fan_block.rs:16` |
| tube_coral_wall_fan | CoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs:21` |
| brain_coral_wall_fan | CoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs:21` |
| bubble_coral_wall_fan | CoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs:21` |
| fire_coral_wall_fan | CoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs:21` |
| horn_coral_wall_fan | CoralWallFanBlock | `steel-core/src/behavior/blocks/vegetation/coral_wall_fan_block.rs:21` |
| sea_pickle | SeaPickleBlock | `steel-core/src/behavior/blocks/vegetation/sea_pickle_block.rs:26` |
| blue_ice | HalfTransparentBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:71` |
| conduit | ConduitBlock | `steel-core/src/behavior/blocks/building/conduit_block.rs:24` |
| bamboo_sapling | BambooSaplingBlock | `steel-core/src/behavior/blocks/vegetation/bamboo_sapling.rs:25` |
| bamboo | BambooStalkBlock | `steel-core/src/behavior/blocks/vegetation/bamboo.rs:22` |
| potted_bamboo | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| void_air | AirBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:29` |
| cave_air | AirBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:29` |
| bubble_column | BubbleColumnBlock | `steel-core/src/behavior/blocks/fluid/bubble_column_block.rs:28` |
| polished_granite_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| smooth_red_sandstone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| mossy_stone_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_diorite_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| mossy_cobblestone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| end_stone_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| stone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| smooth_sandstone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| smooth_quartz_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| granite_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| andesite_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| red_nether_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_andesite_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| diorite_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_granite_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| smooth_red_sandstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| mossy_stone_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_diorite_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| mossy_cobblestone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| end_stone_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| smooth_sandstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| smooth_quartz_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| granite_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| andesite_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| red_nether_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_andesite_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| diorite_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| prismarine_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| red_sandstone_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| mossy_stone_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| granite_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| stone_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| mud_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| nether_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| andesite_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| red_nether_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| sandstone_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| end_stone_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| diorite_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| scaffolding | ScaffoldingBlock | `steel-core/src/behavior/blocks/building/scaffolding_block.rs:40` |
| loom | LoomBlock | `steel-core/src/behavior/blocks/container/station_blocks.rs:27` |
| barrel | BarrelBlock | `steel-core/src/behavior/blocks/container/barrel_block.rs:29` |
| smoker | SmokerBlock | `steel-core/src/behavior/blocks/container/furnace_block.rs:39` |
| blast_furnace | BlastFurnaceBlock | `steel-core/src/behavior/blocks/container/furnace_block.rs:33` |
| cartography_table | CartographyTableBlock | `steel-core/src/behavior/blocks/container/station_blocks.rs:69` |
| fletching_table | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| grindstone | GrindstoneBlock | `steel-core/src/behavior/blocks/container/grindstone_block.rs:19` |
| lectern | LecternBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:939` |
| smithing_table | SmithingTableBlock | `steel-core/src/behavior/blocks/container/station_blocks.rs:108` |
| stonecutter | StonecutterBlock | `steel-core/src/behavior/blocks/container/station_blocks.rs:147` |
| bell | BellBlock | `steel-core/src/behavior/blocks/redstone/bell_block.rs:47` |
| lantern | LanternBlock | `steel-core/src/behavior/blocks/decoration/lantern_block.rs:20` |
| soul_lantern | LanternBlock | `steel-core/src/behavior/blocks/decoration/lantern_block.rs:20` |
| copper_lantern | WeatheringLanternBlock | `steel-core/src/behavior/blocks/decoration/weathering_lantern_block.rs:17` |
| exposed_copper_lantern | WeatheringLanternBlock | `steel-core/src/behavior/blocks/decoration/weathering_lantern_block.rs:17` |
| weathered_copper_lantern | WeatheringLanternBlock | `steel-core/src/behavior/blocks/decoration/weathering_lantern_block.rs:17` |
| oxidized_copper_lantern | WeatheringLanternBlock | `steel-core/src/behavior/blocks/decoration/weathering_lantern_block.rs:17` |
| waxed_copper_lantern | LanternBlock | `steel-core/src/behavior/blocks/decoration/lantern_block.rs:20` |
| waxed_exposed_copper_lantern | LanternBlock | `steel-core/src/behavior/blocks/decoration/lantern_block.rs:20` |
| waxed_weathered_copper_lantern | LanternBlock | `steel-core/src/behavior/blocks/decoration/lantern_block.rs:20` |
| waxed_oxidized_copper_lantern | LanternBlock | `steel-core/src/behavior/blocks/decoration/lantern_block.rs:20` |
| campfire | CampfireBlock | `steel-core/src/behavior/blocks/building/campfire_block.rs:39` |
| soul_campfire | CampfireBlock | `steel-core/src/behavior/blocks/building/campfire_block.rs:39` |
| sweet_berry_bush | SweetBerryBushBlock | `steel-core/src/behavior/blocks/vegetation/sweet_berry_bush.rs:42` |
| warped_stem | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_warped_stem | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| warped_hyphae | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_warped_hyphae | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| warped_nylium | NyliumBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:129` |
| warped_fungus | NetherFungusBlock | `steel-core/src/behavior/blocks/vegetation/nether_fungus_block.rs:22` |
| warped_wart_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_roots | NetherRootsBlock | `steel-core/src/behavior/blocks/vegetation/nether_roots_block.rs:14` |
| nether_sprouts | NetherSproutsBlock | `steel-core/src/behavior/blocks/vegetation/nether_sprouts.rs:20` |
| crimson_stem | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_crimson_stem | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| crimson_hyphae | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| stripped_crimson_hyphae | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| crimson_nylium | NyliumBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:129` |
| crimson_fungus | NetherFungusBlock | `steel-core/src/behavior/blocks/vegetation/nether_fungus_block.rs:22` |
| shroomlight | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| weeping_vines | WeepingVinesBlock | `steel-core/src/behavior/blocks/vegetation/weeping_vines_block.rs:20` |
| weeping_vines_plant | WeepingVinesPlantBlock | `steel-core/src/behavior/blocks/vegetation/weeping_vines_plant_block.rs:18` |
| twisting_vines | TwistingVinesBlock | `steel-core/src/behavior/blocks/vegetation/twisting_vines_block.rs:21` |
| twisting_vines_plant | TwistingVinesPlantBlock | `steel-core/src/behavior/blocks/vegetation/twisting_vines_plant_block.rs:19` |
| crimson_roots | NetherRootsBlock | `steel-core/src/behavior/blocks/vegetation/nether_roots_block.rs:14` |
| crimson_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| warped_planks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| crimson_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| warped_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| crimson_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| warped_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| crimson_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| warped_fence | FenceBlock | `steel-core/src/behavior/blocks/building/fence_block.rs:30` |
| crimson_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| warped_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| crimson_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| warped_fence_gate | FenceGateBlock | `steel-core/src/behavior/blocks/building/fence_gate_block.rs:31` |
| crimson_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| warped_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| crimson_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| warped_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| crimson_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| warped_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| crimson_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| warped_sign | StandingSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:335` |
| crimson_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| warped_wall_sign | WallSignBlock | `steel-core/src/behavior/blocks/decoration/sign_block.rs:447` |
| structure_block | StructureBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:383` |
| jigsaw | JigsawBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:325` |
| test_block | TestBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:201` |
| test_instance_block | TestInstanceBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:363` |
| composter | ComposterBlock | `steel-core/src/behavior/blocks/building/composter_block.rs:32` |
| target | TargetBlock | `steel-core/src/behavior/blocks/redstone/target_block.rs:27` |
| bee_nest | BeehiveBlock | `steel-core/src/behavior/blocks/container/beehive_block.rs:38` |
| beehive | BeehiveBlock | `steel-core/src/behavior/blocks/container/beehive_block.rs:38` |
| honey_block | HoneyBlock | `steel-core/src/behavior/blocks/building/honey_block.rs:34` |
| honeycomb_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| ancient_debris | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| crying_obsidian | CryingObsidianBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:132` |
| respawn_anchor | RespawnAnchorBlock | `steel-core/src/behavior/blocks/portal/respawn_anchor_block.rs:98` |
| potted_crimson_fungus | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_warped_fungus | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_crimson_roots | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_warped_roots | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| lodestone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| blackstone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| blackstone_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| blackstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_polished_blackstone_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_polished_blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_blackstone_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_blackstone_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| gilded_blackstone | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_blackstone_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_blackstone_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_blackstone_pressure_plate | PressurePlateBlock | `steel-core/src/behavior/blocks/redstone/pressure_plate/standard.rs:30` |
| polished_blackstone_button | ButtonBlock | `steel-core/src/behavior/blocks/redstone/button_block.rs:34` |
| polished_blackstone_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_nether_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| quartz_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| white_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| orange_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| magenta_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| light_blue_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| yellow_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| lime_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| pink_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| gray_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| light_gray_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| cyan_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| purple_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| blue_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| brown_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| green_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| red_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| black_candle | CandleBlock | `steel-core/src/behavior/blocks/decoration/candle_block.rs:46` |
| candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| white_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| orange_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| magenta_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| light_blue_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| yellow_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| lime_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| pink_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| gray_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| light_gray_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| cyan_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| purple_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| blue_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| brown_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| green_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| red_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| black_candle_cake | CandleCakeBlock | `steel-core/src/behavior/blocks/decoration/candle_cake_block.rs:31` |
| amethyst_block | AmethystBlock | `steel-core/src/behavior/blocks/building/amethyst_block.rs:14` |
| budding_amethyst | BuddingAmethystBlock | `steel-core/src/behavior/blocks/building/budding_amethyst.rs:21` |
| amethyst_cluster | AmethystClusterBlock | `steel-core/src/behavior/blocks/building/amethyst_cluster.rs:22` |
| large_amethyst_bud | AmethystClusterBlock | `steel-core/src/behavior/blocks/building/amethyst_cluster.rs:22` |
| medium_amethyst_bud | AmethystClusterBlock | `steel-core/src/behavior/blocks/building/amethyst_cluster.rs:22` |
| small_amethyst_bud | AmethystClusterBlock | `steel-core/src/behavior/blocks/building/amethyst_cluster.rs:22` |
| tuff | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| tuff_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| tuff_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| polished_tuff | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_tuff_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_tuff_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_tuff_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_tuff | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tuff_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| tuff_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| tuff_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_tuff_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| potent_sulfur | PotentSulfurBlock | `steel-core/src/behavior/blocks/building/potent_sulfur_block.rs:23` |
| sulfur_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| sulfur_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| sulfur_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| polished_sulfur | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_sulfur_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_sulfur_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_sulfur_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| sulfur_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| sulfur_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| sulfur_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| sulfur_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_sulfur | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cinnabar_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cinnabar_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| polished_cinnabar | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_cinnabar_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_cinnabar_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_cinnabar_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| cinnabar_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cinnabar_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cinnabar_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cinnabar_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_cinnabar | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| calcite | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| tinted_glass | TintedGlassBlock | `steel-core/src/behavior/blocks/building/simple_marker_blocks.rs:91` |
| powder_snow | PowderSnowBlock | `steel-core/src/behavior/blocks/building/powder_snow_block.rs:41` |
| sculk_sensor | SculkSensorBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:791` |
| calibrated_sculk_sensor | CalibratedSculkSensorBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:881` |
| sculk | SculkBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:163` |
| sculk_vein | SculkVeinBlock | `steel-core/src/behavior/blocks/vegetation/sculk_vein_block.rs:28` |
| sculk_catalyst | SculkCatalystBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:983` |
| sculk_shrieker | SculkShriekerBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:1027` |
| copper_block | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| exposed_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| weathered_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| oxidized_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| waxed_copper_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| deepslate_copper_ore | DropExperienceBlock | `steel-core/src/behavior/blocks/building/drop_experience_block.rs:15` |
| cut_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| exposed_cut_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| weathered_cut_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| oxidized_cut_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| waxed_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_cut_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| chiseled_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| exposed_chiseled_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| weathered_chiseled_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| oxidized_chiseled_copper | WeatheringCopperFullBlock | `steel-core/src/behavior/blocks/building/weathering_block.rs:158` |
| waxed_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_exposed_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_weathered_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| waxed_oxidized_chiseled_copper | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cut_copper_stairs | WeatheringCopperStairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:145` |
| exposed_cut_copper_stairs | WeatheringCopperStairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:145` |
| weathered_cut_copper_stairs | WeatheringCopperStairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:145` |
| oxidized_cut_copper_stairs | WeatheringCopperStairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:145` |
| waxed_cut_copper_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| waxed_exposed_cut_copper_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| waxed_weathered_cut_copper_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| waxed_oxidized_cut_copper_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cut_copper_slab | WeatheringCopperSlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:107` |
| exposed_cut_copper_slab | WeatheringCopperSlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:107` |
| weathered_cut_copper_slab | WeatheringCopperSlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:107` |
| oxidized_cut_copper_slab | WeatheringCopperSlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:107` |
| waxed_cut_copper_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| waxed_exposed_cut_copper_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| waxed_weathered_cut_copper_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| waxed_oxidized_cut_copper_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| copper_door | WeatheringCopperDoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:380` |
| exposed_copper_door | WeatheringCopperDoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:380` |
| weathered_copper_door | WeatheringCopperDoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:380` |
| oxidized_copper_door | WeatheringCopperDoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:380` |
| waxed_copper_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| waxed_exposed_copper_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| waxed_weathered_copper_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| waxed_oxidized_copper_door | DoorBlock | `steel-core/src/behavior/blocks/building/door_block.rs:44` |
| copper_trapdoor | WeatheringCopperTrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:47` |
| exposed_copper_trapdoor | WeatheringCopperTrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:47` |
| weathered_copper_trapdoor | WeatheringCopperTrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:47` |
| oxidized_copper_trapdoor | WeatheringCopperTrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:47` |
| waxed_copper_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| waxed_exposed_copper_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| waxed_weathered_copper_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| waxed_oxidized_copper_trapdoor | TrapDoorBlock | `steel-core/src/behavior/blocks/building/trapdoor_block.rs:29` |
| copper_grate | WeatheringCopperGrateBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:56` |
| exposed_copper_grate | WeatheringCopperGrateBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:56` |
| weathered_copper_grate | WeatheringCopperGrateBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:56` |
| oxidized_copper_grate | WeatheringCopperGrateBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:56` |
| waxed_copper_grate | WaterloggedTransparentBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:16` |
| waxed_exposed_copper_grate | WaterloggedTransparentBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:16` |
| waxed_weathered_copper_grate | WaterloggedTransparentBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:16` |
| waxed_oxidized_copper_grate | WaterloggedTransparentBlock | `steel-core/src/behavior/blocks/building/waterlogged_transparent_block.rs:16` |
| copper_bulb | WeatheringCopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:122` |
| exposed_copper_bulb | WeatheringCopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:122` |
| weathered_copper_bulb | WeatheringCopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:122` |
| oxidized_copper_bulb | WeatheringCopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:122` |
| waxed_copper_bulb | CopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:19` |
| waxed_exposed_copper_bulb | CopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:19` |
| waxed_weathered_copper_bulb | CopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:19` |
| waxed_oxidized_copper_bulb | CopperBulbBlock | `steel-core/src/behavior/blocks/redstone/copper_bulb_block.rs:19` |
| copper_chest | WeatheringCopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:140` |
| exposed_copper_chest | WeatheringCopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:140` |
| weathered_copper_chest | WeatheringCopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:140` |
| oxidized_copper_chest | WeatheringCopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:140` |
| waxed_copper_chest | CopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:30` |
| waxed_exposed_copper_chest | CopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:30` |
| waxed_weathered_copper_chest | CopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:30` |
| waxed_oxidized_copper_chest | CopperChestBlock | `steel-core/src/behavior/blocks/container/copper_chest_block.rs:30` |
| copper_golem_statue | WeatheringCopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:775` |
| exposed_copper_golem_statue | WeatheringCopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:775` |
| weathered_copper_golem_statue | WeatheringCopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:775` |
| oxidized_copper_golem_statue | WeatheringCopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:775` |
| waxed_copper_golem_statue | CopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:550` |
| waxed_exposed_copper_golem_statue | CopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:550` |
| waxed_weathered_copper_golem_statue | CopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:550` |
| waxed_oxidized_copper_golem_statue | CopperGolemStatueBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:550` |
| lightning_rod | WeatheringLightningRodBlock | `steel-core/src/behavior/blocks/redstone/weathering_lightning_rod_block.rs:17` |
| exposed_lightning_rod | WeatheringLightningRodBlock | `steel-core/src/behavior/blocks/redstone/weathering_lightning_rod_block.rs:17` |
| weathered_lightning_rod | WeatheringLightningRodBlock | `steel-core/src/behavior/blocks/redstone/weathering_lightning_rod_block.rs:17` |
| oxidized_lightning_rod | WeatheringLightningRodBlock | `steel-core/src/behavior/blocks/redstone/weathering_lightning_rod_block.rs:17` |
| waxed_lightning_rod | LightningRodBlock | `steel-core/src/behavior/blocks/redstone/lightning_rod_block.rs:20` |
| waxed_exposed_lightning_rod | LightningRodBlock | `steel-core/src/behavior/blocks/redstone/lightning_rod_block.rs:20` |
| waxed_weathered_lightning_rod | LightningRodBlock | `steel-core/src/behavior/blocks/redstone/lightning_rod_block.rs:20` |
| waxed_oxidized_lightning_rod | LightningRodBlock | `steel-core/src/behavior/blocks/redstone/lightning_rod_block.rs:20` |
| dripstone_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| pointed_dripstone | PointedDripstoneBlock | `steel-core/src/behavior/blocks/vegetation/pointed_dripstone_block.rs:37` |
| sulfur_spike | SulfurSpikeBlock | `steel-core/src/behavior/blocks/vegetation/pointed_dripstone_block.rs:130` |
| cave_vines | CaveVinesBlock | `steel-core/src/behavior/blocks/vegetation/cave_vines_block.rs:32` |
| cave_vines_plant | CaveVinesPlantBlock | `steel-core/src/behavior/blocks/vegetation/cave_vines_plant_block.rs:25` |
| spore_blossom | SporeBlossomBlock | `steel-core/src/behavior/blocks/vegetation/spore_blossom_block.rs:14` |
| azalea | AzaleaBlock | `steel-core/src/behavior/blocks/vegetation/azalea_block.rs:20` |
| flowering_azalea | AzaleaBlock | `steel-core/src/behavior/blocks/vegetation/azalea_block.rs:20` |
| moss_carpet | CarpetBlock | `steel-core/src/behavior/blocks/vegetation/carpet_block.rs:14` |
| pink_petals | FlowerBedBlock | `steel-core/src/behavior/blocks/vegetation/flower_bed_block.rs:27` |
| wildflowers | FlowerBedBlock | `steel-core/src/behavior/blocks/vegetation/flower_bed_block.rs:27` |
| leaf_litter | LeafLitterBlock | `steel-core/src/behavior/blocks/vegetation/leaf_litter_block.rs:17` |
| moss_block | BonemealableFeaturePlacerBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:1148` |
| big_dripleaf | BigDripleafBlock | `steel-core/src/behavior/blocks/vegetation/big_dripleaf_block.rs:33` |
| big_dripleaf_stem | BigDripleafStemBlock | `steel-core/src/behavior/blocks/vegetation/big_dripleaf_stem_block.rs:28` |
| small_dripleaf | SmallDripleafBlock | `steel-core/src/behavior/blocks/vegetation/small_dripleaf_block.rs:23` |
| hanging_roots | HangingRootsBlock | `steel-core/src/behavior/blocks/vegetation/hanging_roots_block.rs:14` |
| rooted_dirt | RootedDirtBlock | `steel-core/src/behavior/blocks/vegetation/rooted_dirt_block.rs:20` |
| mud | MudBlock | `steel-core/src/behavior/blocks/building/mud.rs:11` |
| deepslate | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| cobbled_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cobbled_deepslate_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| cobbled_deepslate_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| cobbled_deepslate_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| polished_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| polished_deepslate_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| polished_deepslate_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| polished_deepslate_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| deepslate_tiles | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_tile_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| deepslate_tile_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| deepslate_tile_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| deepslate_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| deepslate_brick_stairs | StairBlock | `steel-core/src/behavior/blocks/building/stair_block.rs:24` |
| deepslate_brick_slab | SlabBlock | `steel-core/src/behavior/blocks/building/slab_block.rs:30` |
| deepslate_brick_wall | WallBlock | `steel-core/src/behavior/blocks/building/wall_block.rs:31` |
| chiseled_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_deepslate_bricks | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| cracked_deepslate_tiles | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| infested_deepslate | InfestedRotatedPillarBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:92` |
| smooth_basalt | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_iron_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_copper_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| raw_gold_block | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| potted_azalea_bush | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_flowering_azalea_bush | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| ochre_froglight | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| verdant_froglight | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| pearlescent_froglight | RotatedPillarBlock | `steel-core/src/behavior/blocks/building/rotated_pillar_block.rs:19` |
| frogspawn | FrogspawnBlock | `steel-core/src/behavior/blocks/building/infested_block.rs:189` |
| reinforced_deepslate | Block | Нет точной аннотации: проверить наследование/генератор/fallback |
| decorated_pot | DecoratedPotBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1163` |
| crafter | CrafterBlock | `steel-core/src/behavior/blocks/container/block_entity_blocks.rs:1004` |
| trial_spawner | TrialSpawnerBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:592` |
| vault | VaultBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:638` |
| heavy_core | HeavyCoreBlock | `steel-core/src/behavior/blocks/building/heavy_core.rs:15` |
| pale_moss_block | BonemealableFeaturePlacerBlock | `steel-core/src/behavior/blocks/building/technical_blocks.rs:1148` |
| pale_moss_carpet | MossyCarpetBlock | `steel-core/src/behavior/blocks/vegetation/mossy_carpet_block.rs:21` |
| pale_hanging_moss | HangingMossBlock | `steel-core/src/behavior/blocks/vegetation/hanging_moss_block.rs:21` |
| open_eyeblossom | EyeblossomBlock | `steel-core/src/behavior/blocks/vegetation/eyeblossom_block.rs:30` |
| closed_eyeblossom | EyeblossomBlock | `steel-core/src/behavior/blocks/vegetation/eyeblossom_block.rs:30` |
| potted_open_eyeblossom | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| potted_closed_eyeblossom | FlowerPotBlock | `steel-core/src/behavior/blocks/decoration/flower_pot_block.rs:35` |
| firefly_bush | FireflyBushBlock | `steel-core/src/behavior/blocks/vegetation/firefly_bush_block.rs:19` |

## items: входы генератора

| Объект | Vanilla class | Точная аннотация реализации |
|---|---|---|
| air | AirItem | `steel-core/src/behavior/items/air.rs:10` |
| stone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| granite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_granite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| diorite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_diorite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| andesite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_andesite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobbled_deepslate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_deepslate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| calcite | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_tuff | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_tuff | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_tuff_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_tuff_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_tuff_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tuff_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_tuff_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| potent_sulfur | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_sulfur | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_sulfur_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_sulfur_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_sulfur_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_sulfur | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_cinnabar | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_cinnabar_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_cinnabar_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_cinnabar_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cinnabar_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_cinnabar | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dripstone_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| grass_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dirt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| coarse_dirt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| podzol | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| rooted_dirt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mud | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_nylium | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_nylium | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobblestone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_planks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_mosaic | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_propagule | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_sapling | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bedrock | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sand | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| suspicious_sand | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| suspicious_gravel | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_sand | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gravel | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| coal_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_coal_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| iron_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_iron_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_copper_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gold_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_gold_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| redstone_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_redstone_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| emerald_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_emerald_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lapis_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_lapis_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| diamond_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_diamond_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_gold_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_quartz_ore | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ancient_debris | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| coal_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| raw_iron_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| raw_copper_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| raw_gold_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| heavy_core | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| amethyst_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| budding_amethyst | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| iron_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gold_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| diamond_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| netherite_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_chiseled_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_cut_copper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_cut_copper_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_cut_copper_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_roots | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| muddy_mangrove_roots | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_stem | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_stem | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_oak_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_spruce_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_birch_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_jungle_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_acacia_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_cherry_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_dark_oak_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_pale_oak_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_mangrove_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_poplar_log | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_crimson_stem | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_warped_stem | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_oak_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_spruce_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_birch_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_jungle_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_acacia_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_cherry_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_dark_oak_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_pale_oak_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_mangrove_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_poplar_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_crimson_hyphae | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_warped_hyphae | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stripped_bamboo_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_wood | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_hyphae | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_hyphae | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_poplar_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_poplar_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_poplar_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| azalea_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| flowering_azalea_leaves | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sponge | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| wet_sponge | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tinted_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lapis_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobweb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| short_grass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| fern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bush | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_shrub | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| azalea | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| flowering_azalea | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_bush | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| firefly_bush | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| short_dry_grass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tall_dry_grass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| seagrass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sea_pickle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_wool | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_wool_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_wool_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dandelion | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| golden_dandelion | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| open_eyeblossom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| closed_eyeblossom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poppy | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_orchid | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| allium | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| azure_bluet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_tulip | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_tulip | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_tulip | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_tulip | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxeye_daisy | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cornflower | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lily_of_the_valley | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| wither_rose | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| torchflower | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pitcher_plant | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spore_blossom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_mushroom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_mushroom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| shelf_mushroom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_fungus | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_fungus | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_roots | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_roots | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_sprouts | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weeping_vines | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| twisting_vines | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sugar_cane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| kelp | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_petals | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| wildflowers | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| leaf_litter | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| moss_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| moss_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_moss_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_hanging_moss | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_moss_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| hanging_roots | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| big_dripleaf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| small_dripleaf | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| bamboo | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_mosaic_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_stone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sandstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_sandstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| petrified_oak_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobblestone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mud_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| quartz_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_sandstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_red_sandstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purpur_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_prismarine_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_quartz | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_red_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_stone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_shelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bookshelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_bookshelf | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| decorated_pot | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_cobblestone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| obsidian | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| torch | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| end_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chorus_plant | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chorus_flower | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purpur_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purpur_pillar | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purpur_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spawner | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| creaking_heart | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crafting_table | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| farmland | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| furnace | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ladder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobblestone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| snow | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ice | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| snow_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cactus | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cactus_flower | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| clay | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jukebox | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pumpkin | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| carved_pumpkin | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jack_o_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| netherrack | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| soul_sand | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| soul_soil | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| basalt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_basalt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_basalt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| soul_torch | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| copper_torch | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| glowstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_stone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_cobblestone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_mossy_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_cracked_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_chiseled_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| infested_deepslate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cracked_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| packed_mud | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mud_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cracked_deepslate_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_tiles | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cracked_deepslate_tiles | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_deepslate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| reinforced_deepslate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_mushroom_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_mushroom_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mushroom_stem | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| iron_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_bars | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| iron_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_chain | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| melon | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| vine | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| glow_lichen | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| resin_clump | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| resin_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| resin_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| resin_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| resin_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| resin_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_resin_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mud_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mycelium | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lily_pad | PlaceOnWaterBlockItem | `steel-core/src/behavior/items/place_on_water_block_item.rs:9` |
| nether_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cracked_nether_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_nether_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_brick_fence | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sculk | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sculk_vein | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sculk_catalyst | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sculk_shrieker | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| enchanting_table | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| end_portal_frame | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| end_stone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| end_stone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dragon_egg | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sandstone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ender_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| emerald_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_mosaic_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| command_block | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| beacon | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobblestone_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_cobblestone_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_sandstone_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_stone_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| granite_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mud_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| andesite_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_nether_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sandstone_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| end_stone_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| diorite_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blackstone_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobbled_deepslate_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_deepslate_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_brick_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_tile_wall | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| anvil | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chipped_anvil | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| damaged_anvil | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_quartz_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| quartz_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| quartz_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| quartz_pillar | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| quartz_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| barrier | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| hay_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_carpet | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| packed_ice | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dirt_path | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sunflower | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| lilac | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| rose_bush | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| peony | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| tall_grass | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| large_fern | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| white_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_stained_glass | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_stained_glass_pane | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_prismarine | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| prismarine_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_prismarine_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sea_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_red_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cut_red_sandstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_sandstone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| repeating_command_block | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| chain_command_block | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| magma_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| nether_wart_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_wart_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_nether_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bone_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| structure_void | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_shulker_box | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_glazed_terracotta | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_concrete | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_concrete_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_concrete_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_concrete_powder | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| turtle_egg | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sniffer_egg | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dried_ghast | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_tube_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_brain_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_bubble_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_fire_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_horn_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tube_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brain_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bubble_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| fire_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| horn_coral_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tube_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brain_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bubble_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| fire_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| horn_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_brain_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_bubble_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_fire_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_horn_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dead_tube_coral | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tube_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| brain_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| bubble_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| fire_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| horn_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dead_tube_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dead_brain_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dead_bubble_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dead_fire_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dead_horn_coral_fan | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| blue_ice | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| conduit | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_granite_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_red_sandstone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_stone_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_diorite_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_cobblestone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| end_stone_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_sandstone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_quartz_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| granite_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| andesite_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_nether_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_andesite_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| diorite_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobbled_deepslate_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_deepslate_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_tile_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_granite_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_red_sandstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_stone_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_diorite_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mossy_cobblestone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| end_stone_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_sandstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smooth_quartz_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| granite_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| andesite_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_nether_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_andesite_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| diorite_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cobbled_deepslate_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_deepslate_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| deepslate_tile_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| scaffolding | ScaffoldingBlockItem | `steel-core/src/behavior/items/scaffolding_block_item.rs:12` |
| redstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| redstone_torch | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| redstone_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| repeater | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| comparator | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| piston | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sticky_piston | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| slime_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| honey_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| observer | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| hopper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dispenser | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dropper | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lectern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| target | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lever | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_lightning_rod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| daylight_detector | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sculk_sensor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| calibrated_sculk_sensor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tripwire_hook | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| trapped_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| tnt | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| redstone_lamp | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| note_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_button | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stone_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_weighted_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| heavy_weighted_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_pressure_plate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| iron_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| oak_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| spruce_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| birch_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| jungle_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| acacia_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| cherry_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| dark_oak_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| pale_oak_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| mangrove_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| poplar_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| bamboo_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| crimson_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| warped_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| exposed_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| weathered_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| oxidized_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| waxed_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| waxed_exposed_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| waxed_weathered_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| waxed_oxidized_copper_door | DoubleHighBlockItem | `steel-core/src/behavior/items/block_item.rs:232` |
| iron_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_trapdoor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oak_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| spruce_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| birch_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| jungle_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| acacia_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cherry_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| dark_oak_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pale_oak_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| mangrove_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| poplar_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bamboo_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crimson_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| warped_fence_gate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| powered_rail | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| detector_rail | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| rail | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| activator_rail | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
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
| carrot_on_a_stick | FoodOnAStickItem | `steel-core/src/behavior/items/food_on_a_stick.rs:14` |
| warped_fungus_on_a_stick | FoodOnAStickItem | `steel-core/src/behavior/items/food_on_a_stick.rs:14` |
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
| structure_block | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| jigsaw | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| test_block | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| test_instance_block | GameMasterBlockItem | `steel-core/src/behavior/items/game_master_block_item.rs:16` |
| turtle_helmet | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| turtle_scute | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| armadillo_scute | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wolf_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| flint_and_steel | FlintAndSteelItem | `steel-core/src/behavior/items/flint_and_steel.rs:21` |
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
| string | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| feather | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| gunpowder | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| wheat_seeds | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
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
| oak_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| spruce_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| birch_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| jungle_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| acacia_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| cherry_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dark_oak_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| pale_oak_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| mangrove_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| poplar_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| bamboo_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| crimson_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| warped_sign | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| oak_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| spruce_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| birch_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| jungle_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| acacia_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| cherry_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| dark_oak_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| pale_oak_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| mangrove_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| poplar_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| bamboo_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| crimson_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| warped_hanging_sign | HangingSignItem | `steel-core/src/behavior/items/hanging_sign_item.rs:22` |
| bucket | BucketItem | `steel-core/src/behavior/items/bucket.rs:28` |
| water_bucket | BucketItem | `steel-core/src/behavior/items/bucket.rs:28` |
| lava_bucket | BucketItem | `steel-core/src/behavior/items/bucket.rs:28` |
| powder_snow_bucket | SolidBucketItem | `steel-core/src/behavior/items/solid_bucket_item.rs:12` |
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
| dried_kelp_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| paper | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| book | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| slime_ball | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| egg | EggItem | `steel-core/src/behavior/items/egg.rs:30` |
| blue_egg | EggItem | `steel-core/src/behavior/items/egg.rs:30` |
| brown_egg | EggItem | `steel-core/src/behavior/items/egg.rs:30` |
| compass | CompassItem | `steel-core/src/behavior/items/compass.rs:24` |
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
| fishing_rod | FishingRodItem | `steel-core/src/behavior/items/fishing_rod.rs:17` |
| clock | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| spyglass | SpyglassItem | `steel-core/src/behavior/items/spyglass.rs:17` |
| glowstone_dust | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| salmon | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| tropical_fish | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pufferfish | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_cod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_salmon | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ink_sac | InkSacItem | `steel-core/src/behavior/items/ink_sac.rs:36` |
| glow_ink_sac | GlowInkSacItem | `steel-core/src/behavior/items/ink_sac.rs:59` |
| cocoa_beans | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
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
| bone_meal | BoneMealItem | `steel-core/src/behavior/items/bonemeal.rs:23` |
| bone | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| sugar | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cake | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| straw_bed | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cookie | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| crafter | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
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
| shears | ShearsItem | `steel-core/src/behavior/items/shears.rs:19` |
| melon_slice | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| dried_kelp | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| pumpkin_seeds | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| melon_seeds | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| beef | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_beef | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| chicken | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_chicken | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| rotten_flesh | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ender_pearl | EnderpearlItem | `steel-core/src/behavior/items/ender_pearl.rs:30` |
| blaze_rod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ghast_tear | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| gold_nugget | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| nether_wart | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| glass_bottle | BottleItem | `steel-core/src/behavior/items/bottle.rs:23` |
| potion | PotionItem | `steel-core/src/behavior/items/potion.rs:33` |
| spider_eye | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| fermented_spider_eye | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| blaze_powder | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| magma_cream | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| brewing_stand | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cauldron | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ender_eye | EnderEyeItem | `steel-core/src/behavior/items/ender_eye.rs:52` |
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
| experience_bottle | ExperienceBottleItem | `steel-core/src/behavior/items/experience_bottle.rs:31` |
| fire_charge | FireChargeItem | `steel-core/src/behavior/items/flint_and_steel.rs:85` |
| wind_charge | WindChargeItem | `steel-core/src/behavior/items/wind_charge.rs:31` |
| writable_book | WritableBookItem | `steel-core/src/behavior/items/books.rs:18` |
| written_book | WrittenBookItem | `steel-core/src/behavior/items/books.rs:35` |
| breeze_rod | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| mace | MaceItem | `steel-core/src/behavior/items/mace.rs:17` |
| item_frame | ItemFrameItem | `steel-core/src/behavior/items/item_frame.rs:24` |
| glow_item_frame | ItemFrameItem | `steel-core/src/behavior/items/item_frame.rs:24` |
| flower_pot | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| carrot | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| potato | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| baked_potato | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| poisonous_potato | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| map | EmptyMapItem | `steel-core/src/behavior/items/empty_map.rs:20` |
| golden_carrot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| skeleton_skull | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| wither_skeleton_skull | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| player_head | PlayerHeadItem | `steel-core/src/behavior/items/player_head.rs:20` |
| zombie_head | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| creeper_head | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| dragon_head | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
| piglin_head | StandingAndWallBlockItem | `steel-core/src/behavior/items/standing_and_wall_block_item.rs:33` |
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
| lead | LeadItem | `steel-core/src/behavior/items/lead.rs:19` |
| name_tag | NameTagItem | `steel-core/src/behavior/items/name_tag.rs:10` |
| command_block_minecart | MinecartItem | `steel-core/src/behavior/items/minecart.rs:18` |
| mutton | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| cooked_mutton | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| white_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| orange_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| magenta_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| light_blue_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| yellow_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| lime_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| pink_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| gray_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| light_gray_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| cyan_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| purple_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| blue_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| brown_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| green_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| red_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| black_banner | BannerItem | `steel-core/src/behavior/items/banner_item.rs:19` |
| end_crystal | EndCrystalItem | `steel-core/src/behavior/items/end_crystal.rs:22` |
| chorus_fruit | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| popped_chorus_fruit | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| torchflower_seeds | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pitcher_pod | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| beetroot | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| beetroot_seeds | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| beetroot_soup | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| dragon_breath | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| splash_potion | SplashPotionItem | `steel-core/src/behavior/items/throwable_potion.rs:41` |
| spectral_arrow | SpectralArrowItem | `steel-core/src/behavior/items/arrow.rs:59` |
| tipped_arrow | TippedArrowItem | `steel-core/src/behavior/items/tipped_arrow.rs:20` |
| lingering_potion | LingeringPotionItem | `steel-core/src/behavior/items/throwable_potion.rs:76` |
| shield | ShieldItem | `steel-core/src/behavior/items/shield.rs:16` |
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
| debug_stick | DebugStickItem | `steel-core/src/behavior/items/debug_stick.rs:36` |
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
| disc_fragment_5 | DiscFragmentItem | `steel-core/src/behavior/items/disc_fragment.rs:17` |
| trident | TridentItem | `steel-core/src/behavior/items/trident.rs:47` |
| nautilus_shell | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| iron_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| golden_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| diamond_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| netherite_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| copper_nautilus_armor | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| heart_of_the_sea | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| crossbow | CrossbowItem | `steel-core/src/behavior/items/crossbow.rs:56` |
| suspicious_stew | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| loom | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
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
| goat_horn | InstrumentItem | `steel-core/src/behavior/items/instrument_item.rs:25` |
| composter | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| barrel | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smoker | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blast_furnace | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cartography_table | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| fletching_table | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| grindstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| smithing_table | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| stonecutter | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| bell | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| soul_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_lantern | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sweet_berries | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| glow_berries | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| campfire | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| soul_campfire | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| shroomlight | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| honeycomb | HoneycombItem | `steel-core/src/behavior/items/honeycomb.rs:28` |
| bee_nest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| beehive | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| honey_bottle | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| honeycomb_block | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lodestone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| crying_obsidian | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blackstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blackstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blackstone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gilded_blackstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| chiseled_polished_blackstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_brick_slab | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| polished_blackstone_brick_stairs | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cracked_polished_blackstone_bricks | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| respawn_anchor | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| white_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| orange_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| magenta_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_blue_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| yellow_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| lime_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pink_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| gray_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| light_gray_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| cyan_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| purple_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| blue_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| brown_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| green_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| red_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| black_candle | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| small_amethyst_bud | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| medium_amethyst_bud | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| large_amethyst_bud | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| amethyst_cluster | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pointed_dripstone | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| sulfur_spike | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ochre_froglight | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| verdant_froglight | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| pearlescent_froglight | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| frogspawn | PlaceOnWaterBlockItem | `steel-core/src/behavior/items/place_on_water_block_item.rs:9` |
| echo_shard | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| brush | BrushItem | `steel-core/src/behavior/items/brush.rs:26` |
| netherite_upgrade_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| sentry_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| dune_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| coast_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| wild_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| ward_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| eye_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| vex_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| tide_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| snout_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| rib_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| spire_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| wayfinder_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| shaper_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| silence_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| raiser_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| host_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| flow_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
| bolt_armor_trim_smithing_template | SmithingTemplateItem | `steel-core/src/behavior/items/smithing_template.rs:57` |
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
| copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_grate | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_bulb | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_chest | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| exposed_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| weathered_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| oxidized_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_exposed_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_weathered_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| waxed_oxidized_copper_golem_statue | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| trial_spawner | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| trial_key | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| ominous_trial_key | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
| vault | BlockItem | `steel-core/src/behavior/items/block_item.rs:26` |
| ominous_bottle | Item | Нет точной аннотации: проверить наследование/генератор/fallback |
