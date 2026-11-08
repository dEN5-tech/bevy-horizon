fn pedestal_orbit_interact_entity(
    game: &mut crate::game::Game<'_>,
    entity: i32,
    kind: &str,
    stack: Option<&horizon_client::items::ItemStack>,
    claimed: &mut bool,
) {
    use crate::game::pedestal_orbit_runtime as m;
    if *claimed || entity < 0 || !game.native_on(m::ID) {
        return;
    }
    let target_id = entity as u32;
    let Some(player) = game.native_player() else { return; };
    let live_mob = game.native_mob(player.dimension, target_id);
    let mob_pos = live_mob.map(|m| m.pos).unwrap_or([player.x, player.y, player.z]);

    let msg = m::host_bridge::bind_or_toggle_mob(target_id, kind, mob_pos);
    *claimed = true;
    let uuid = game.native_acting_id();
    let _ = game.native_action_bar(uuid, horizon_client::text::Text::literal(&msg));
}
