fn pedestal_orbit_settings_click(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    button: usize,
) {
    use crate::game::pedestal_orbit_runtime as m;
    if mod_id == m::ID {
        let mut settings = m::settings(|k| game.native_setting(m::ID, k));
        if let Some((k, v)) = settings.click(button) {
            m::set_settings(settings);
            game.native_set_setting(m::ID, k, &v);
        }
    }
}
