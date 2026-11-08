fn pedestal_orbit_player_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::pedestal_orbit_runtime as m;
    if !game.native_on(m::ID) {
        return;
    }
    let settings = m::settings(|k| game.native_setting(m::ID, k));
    if !settings.enabled {
        return;
    }
    let Some(player) = game.native_player() else {
        return;
    };

    let live_mob = m::host_bridge::bound_mob().and_then(|b| {
        game.native_mob(player.dimension, b.entity_id)
            .map(|mob| (mob.pos, mob.health, mob.max_health))
    });

    let commands = m::host_bridge::step_simulation(
        0,
        [player.x, player.y, player.z],
        [player.yaw, player.pitch],
        20.0,
        20.0,
        live_mob,
    );

    for cmd in commands {
        match cmd {
            m::OutboxCommand::TeleportMob { entity_id, pos, .. } => {
                let _ = game.native_teleport_mob(player.dimension, entity_id, pos);
            }
            m::OutboxCommand::PlaceBlock { pos, block_id, .. } => {
                game.native_flame_particle([pos[0] as f64 + 0.5, pos[1] as f64 + 1.2, pos[2] as f64 + 0.5], true);
                let uuid = game.native_acting_id();
                let _ = game.native_action_bar(uuid, horizon_client::text::Text::literal(&format!("[PedestalOrbit] Anchor placed at ({}, {}, {}) [{}]", pos[0], pos[1], pos[2], block_id)));
            }
            m::OutboxCommand::PushMob { entity_id, velocity, .. } => {
                let _ = game.native_push_mob(player.dimension, entity_id, velocity);
            }
            m::OutboxCommand::ActionBar { message } => {
                let uuid = game.native_acting_id();
                let _ = game.native_action_bar(uuid, horizon_client::text::Text::literal(&message));
            }
            m::OutboxCommand::Chat { message } => {
                game.native_chat(&message);
            }
        }
    }
}
