{
    use crate::game::pedestal_orbit_runtime as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        name: m::NAME.into(),
        version: m::VERSION.into(),
        description: m::DESCRIPTION.into(),
        authors: vec!["Horizon Community".into()],
    });
}
