fn pedestal_orbit_hud_draw(
    game: &mut crate::game::Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    corners: &mut horizon_client::extras_hud::Corners,
) {
    use crate::game::pedestal_orbit_runtime as m;
    use horizon_client::extras::Corner;

    if !game.native_on(m::ID) {
        return;
    }
    let settings = m::settings(|k| game.native_setting(m::ID, k));
    if !settings.enabled || !settings.hud_on {
        return;
    }

    let corner = [
        Corner::TopLeft,
        Corner::TopRight,
        Corner::BottomLeft,
        Corner::BottomRight,
    ][settings.corner % 4];

    if !game.native_corner_shown(corner) {
        return;
    }

    let Some(label) = m::host_bridge::latest_hud_label() else {
        return;
    };

    let text_w = game.native_text_width(&label);
    let box_w = text_w + 8.0;
    let box_h = 13.0;
    let origin = corners.place(corner, box_w, box_h);

    game.native_fill(batch, [origin[0], origin[1], origin[0] + box_w, origin[1] + box_h], 0x88000000);
    game.native_text(batch, &label, [origin[0] + 4.0, origin[1] + 2.0], 0x00FFCC, true);
}
