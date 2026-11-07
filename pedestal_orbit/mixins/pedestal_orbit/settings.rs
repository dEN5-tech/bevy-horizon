{
    use crate::game::pedestal_orbit_runtime as m;
    if mod_id == m::ID {
        let settings = m::settings(|k| game.native_setting(m::ID, k));
        *buttons = m::settings_labels(&settings);
    }
}
