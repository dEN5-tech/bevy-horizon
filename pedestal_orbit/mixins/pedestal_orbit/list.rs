fn pedestal_orbit_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::pedestal_orbit_runtime as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: vec![
            [
                m::KEY_PLACE.into(),
                "Place Pedestal Block".into(),
                m::DEFAULT_KEY_PLACE.into(),
            ],
            [
                m::KEY_CYCLE_BLOCK.into(),
                "Cycle Block Type".into(),
                m::DEFAULT_KEY_CYCLE_BLOCK.into(),
            ],
            [
                m::KEY_RELEASE.into(),
                "Release Orbiting Mob".into(),
                m::DEFAULT_KEY_RELEASE.into(),
            ],
        ],
    });
}
