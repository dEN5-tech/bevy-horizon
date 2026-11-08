fn pedestal_orbit_key_event(
    game: &mut crate::game::Game<'_>,
    key: &str,
    pressed: bool,
) {
    use crate::game::pedestal_orbit_runtime as m;
    if !pressed || !game.native_playing() {
        return;
    }

    let place_key = game.native_mod_key(m::ID, m::KEY_PLACE, m::DEFAULT_KEY_PLACE);
    let cycle_key = game.native_mod_key(m::ID, m::KEY_CYCLE_BLOCK, m::DEFAULT_KEY_CYCLE_BLOCK);
    let release_key = game.native_mod_key(m::ID, m::KEY_RELEASE, m::DEFAULT_KEY_RELEASE);

    let is_place = key == place_key || key == "key.keyboard.p" || key == "p" || key == "P";
    let is_cycle = key == cycle_key || key == "key.keyboard.b" || key == "b" || key == "B";
    let is_release = key == release_key || key == "key.keyboard.g" || key == "g" || key == "G";

    if is_place {
        if let Some(player) = game.native_player() {
            // Raycast look ray from camera to detect aimed block
            let eye = [player.x, player.y + 1.62, player.z];
            let yaw_rad = (player.yaw as f64).to_radians();
            let pitch_rad = (player.pitch as f64).to_radians();
            let dir = [
                -yaw_rad.sin() * pitch_rad.cos(),
                -pitch_rad.sin(),
                yaw_rad.cos() * pitch_rad.cos(),
            ];

            let mut block_pos = [
                player.x.floor() as i32,
                player.y.floor() as i32 - 1,
                player.z.floor() as i32,
            ];

            for step in 1..=24 {
                let dist = (step as f64) * 0.25;
                let check = [
                    (eye[0] + dir[0] * dist).floor() as i32,
                    (eye[1] + dir[1] * dist).floor() as i32,
                    (eye[2] + dir[2] * dist).floor() as i32,
                ];
                let blk = game.native_block(check);
                if blk != "minecraft:air" && blk != "minecraft:cave_air" && blk != "minecraft:void_air" && !blk.is_empty() {
                    block_pos = check;
                    break;
                }
            }

            let detected_block = game.native_block(block_pos);
            let settings = m::settings(|k| game.native_setting(m::ID, k));

            m::host_bridge::place_pedestal(
                block_pos,
                &detected_block,
                settings.radius(),
                settings.speed(),
                settings.bob_amplitude(),
            );

            // Emit anchor beacon burst particles & sound
            let anchor_top = [
                block_pos[0] as f64 + 0.5,
                block_pos[1] as f64 + 1.2,
                block_pos[2] as f64 + 0.5,
            ];
            game.native_flame_particle(anchor_top, true);
            let _ = game.native_level_play_sound("minecraft:block.beacon.activate", anchor_top, 0, 1.0, 1.2);

            // Auto-bind nearest mob within 30 blocks
            let live_mobs = game.native_mobs(player.dimension);
            let nearest = live_mobs.into_iter().min_by(|a, b| {
                let da = (a.pos[0] - block_pos[0] as f64).powi(2)
                    + (a.pos[1] - block_pos[1] as f64).powi(2)
                    + (a.pos[2] - block_pos[2] as f64).powi(2);
                let db = (b.pos[0] - block_pos[0] as f64).powi(2)
                    + (b.pos[1] - block_pos[1] as f64).powi(2)
                    + (b.pos[2] - block_pos[2] as f64).powi(2);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });

            let mut bound_info = None;
            if let Some(mob) = nearest {
                let dist = ((mob.pos[0] - block_pos[0] as f64).powi(2)
                    + (mob.pos[1] - block_pos[1] as f64).powi(2)
                    + (mob.pos[2] - block_pos[2] as f64).powi(2)).sqrt();
                if dist < 30.0 {
                    let msg = m::host_bridge::bind_or_toggle_mob(mob.id, &mob.kind, mob.pos);
                    bound_info = Some((mob.kind.clone(), mob.id, msg));
                }
            }

            // If no mob nearby, spawn companion mob (Allay or Pig) directly above pedestal
            if bound_info.is_none() {
                let summon_pos = [
                    block_pos[0] as f64 + 0.5,
                    block_pos[1] as f64 + 2.0,
                    block_pos[2] as f64 + 0.5,
                ];
                let spawn_cfg = Default::default();
                let spawned_id = game.native_spawn_mob(
                    player.dimension,
                    "minecraft:allay",
                    summon_pos,
                    &spawn_cfg,
                ).or_else(|| {
                    game.native_spawn_mob(
                        player.dimension,
                        "minecraft:pig",
                        summon_pos,
                        &spawn_cfg,
                    )
                });

                if let Some(id) = spawned_id {
                    let msg = m::host_bridge::bind_or_toggle_mob(id, "minecraft:allay", summon_pos);
                    bound_info = Some(("minecraft:allay".into(), id, msg));
                }
            }

            let uuid = game.native_acting_id();
            if let Some((kind, id, _)) = bound_info {
                let _ = game.native_action_bar(
                    uuid,
                    horizon_client::text::Text::literal(&format!("[PedestalOrbit] Bound {} #{} orbiting {}!", kind, id, detected_block)),
                );
                game.native_chat(&format!(
                    "[PedestalOrbit] Anchor placed at ({}, {}, {}) [{}] -> Orbiting: {} #{}",
                    block_pos[0], block_pos[1], block_pos[2], detected_block, kind, id
                ));
            } else {
                let _ = game.native_action_bar(
                    uuid,
                    horizon_client::text::Text::literal(&format!("[PedestalOrbit] Anchor placed at ({}, {}, {}) [{}]. Right-click mob to bind!", block_pos[0], block_pos[1], block_pos[2], detected_block)),
                );
                game.native_chat(&format!(
                    "[PedestalOrbit] Anchor placed at ({}, {}, {}) [{}]. Right-click any mob to bind into orbit!",
                    block_pos[0], block_pos[1], block_pos[2], detected_block
                ));
            }
        }
    } else if is_cycle {
        let mut s = m::settings(|k| game.native_setting(m::ID, k));
        if let Some((k, v)) = s.click(1) {
            m::set_settings(s);
            game.native_set_setting(m::ID, k, &v);
            let uuid = game.native_acting_id();
            let _ = game.native_action_bar(
                uuid,
                horizon_client::text::Text::literal(&format!("[PedestalOrbit] Block -> {}", s.block_label())),
            );
            game.native_chat(&format!("[PedestalOrbit] Block -> {}", s.block_label()));
        }
    } else if is_release {
        if let Some(id) = m::host_bridge::release_mob() {
            let uuid = game.native_acting_id();
            let _ = game.native_action_bar(
                uuid,
                horizon_client::text::Text::literal(&format!("[PedestalOrbit] Released entity #{id} from orbit")),
            );
            game.native_chat(&format!("[PedestalOrbit] Released entity #{id} from orbit"));
        } else {
            game.native_chat("[PedestalOrbit] No mob currently bound");
        }
    }
}
