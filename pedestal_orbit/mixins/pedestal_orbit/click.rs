{
    use crate::game::pedestal_orbit_runtime as m;
    if mod_id == m::ID {
        let mut settings = m::settings(|k| game.native_setting(m::ID, k));
        if let Some((k, v)) = settings.click(button) {
            m::set_settings(settings);
            game.native_set_setting(m::ID, k, &v);
            *buttons = m::settings_labels(&settings);
        }
    }
}
