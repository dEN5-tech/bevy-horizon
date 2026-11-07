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
        player.id,
        [player.x, player.y, player.z],
        [player.yaw, player.pitch],
        player.health,
        player.max_health,
        live_mob,
    );

    for cmd in commands {
        match cmd {
            bevy_horizon::prelude::OutboxCommand::TeleportMob { entity_id, pos, .. } => {
                let _ = game.native_teleport_mob(player.dimension, entity_id, pos);
            }
            bevy_horizon::prelude::OutboxCommand::PlaceBlock { pos, block_id, .. } => {
                let _ = game.native_place_block(player.dimension, pos, &block_id);
            }
            bevy_horizon::prelude::OutboxCommand::PushMob { entity_id, velocity, .. } => {
                let _ = game.native_push_mob(player.dimension, entity_id, velocity);
            }
            bevy_horizon::prelude::OutboxCommand::ActionBar { message } => {
                let uuid = game.native_acting_id();
                let _ = game.native_action_bar(uuid, horizon_client::text::Text::literal(&message));
            }
            bevy_horizon::prelude::OutboxCommand::Chat { message } => {
                game.native_chat(&message);
            }
        }
    }
}
