fn pedestal_orbit_key_event(
    game: &mut crate::game::Game<'_>,
    key: &str,
    pressed: bool,
) {
    use crate::game::pedestal_orbit_runtime as m;
    if !pressed || !game.native_on(m::ID) || !game.native_playing() {
        return;
    }

    let place_key = game.native_mod_key(m::ID, m::KEY_PLACE, m::DEFAULT_KEY_PLACE);
    let cycle_key = game.native_mod_key(m::ID, m::KEY_CYCLE_BLOCK, m::DEFAULT_KEY_CYCLE_BLOCK);
    let release_key = game.native_mod_key(m::ID, m::KEY_RELEASE, m::DEFAULT_KEY_RELEASE);

    if key == place_key {
        if let Some(player) = game.native_player() {
            let block_pos = [
                player.x.floor() as i32,
                player.y.floor() as i32 - 1,
                player.z.floor() as i32,
            ];
            let settings = m::settings(|k| game.native_setting(m::ID, k));
            m::host_bridge::place_pedestal(
                block_pos,
                settings.block_id(),
                settings.radius(),
                settings.speed(),
                settings.bob_amplitude(),
            );
            game.native_chat(&format!(
                "[PedestalOrbit] Anchor placed at ({}, {}, {}) with {}",
                block_pos[0], block_pos[1], block_pos[2], settings.block_label()
            ));
        }
    } else if key == cycle_key {
        let mut s = m::settings(|k| game.native_setting(m::ID, k));
        if let Some((k, v)) = s.click(1) {
            m::set_settings(s);
            game.native_set_setting(m::ID, k, &v);
            game.native_chat(&format!("[PedestalOrbit] Block -> {}", s.block_label()));
        }
    } else if key == release_key {
        if let Some(id) = m::host_bridge::release_mob() {
            game.native_chat(&format!("[PedestalOrbit] Released entity #{id} from orbit"));
        } else {
            game.native_chat("[PedestalOrbit] No mob currently bound");
        }
    }
}
