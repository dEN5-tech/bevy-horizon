//! Horizon Native Mod Runtime: Pedestal Orbit
//! Powered by Bevy ECS via `pedestal_orbit_ecs` and `bevy_horizon`.

pub use pedestal_orbit_ecs::*;

pub const ID: &str = "pedestal_orbit";
pub const NAME: &str = "Pedestal Orbit";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str = "Anchor pedestal with smooth sinusoidal orbiting mob (Bevy ECS).";

pub const KEY_PLACE: &str = "place_pedestal";
pub const DEFAULT_KEY_PLACE: &str = "key.keyboard.p";

pub const KEY_CYCLE_BLOCK: &str = "cycle_block";
pub const DEFAULT_KEY_CYCLE_BLOCK: &str = "key.keyboard.b";

pub const KEY_RELEASE: &str = "release_orbit";
pub const DEFAULT_KEY_RELEASE: &str = "key.keyboard.g";

use std::cell::RefCell;
use std::sync::Mutex;

static CACHED_SETTINGS: Mutex<Option<PedestalSettings>> = Mutex::new(None);

pub fn settings(get: impl Fn(&str) -> Option<String>) -> PedestalSettings {
    let mut cached = CACHED_SETTINGS.lock().unwrap();
    if let Some(s) = *cached {
        return s;
    }

    let get = |k: &str| get(k).map(|v| v.trim().to_owned());
    let find_idx = |key: &str, values: &[&str], default: usize| {
        get(key)
            .and_then(|v| values.iter().position(|&s| s == v))
            .unwrap_or(default)
    };

    let block_ids: Vec<&str> = BLOCKS.iter().map(|b| b.0).collect();
    let corner_ids: Vec<&str> = CORNERS.iter().map(|c| c.0).collect();

    let s = PedestalSettings {
        enabled: get("enabled").as_deref().unwrap_or("true") == "true",
        block_idx: find_idx("block", &block_ids, 0),
        radius_idx: get("radius")
            .and_then(|v| RADII.iter().position(|&r| format!("{r:.1}") == v))
            .unwrap_or(1),
        speed_idx: get("speed")
            .and_then(|v| SPEEDS.iter().position(|&sp| format!("{sp:.1}") == v))
            .unwrap_or(1),
        bob_idx: get("bob")
            .and_then(|v| BOBS.iter().position(|&b| format!("{b:.1}") == v))
            .unwrap_or(1),
        hud_on: get("hud_on").as_deref().unwrap_or("true") == "true",
        corner: find_idx("corner", &corner_ids, 0),
    };

    *cached = Some(s);
    s
}

pub fn set_settings(s: PedestalSettings) {
    let mut cached = CACHED_SETTINGS.lock().unwrap();
    *cached = Some(s);
}

pub fn settings_labels(s: &PedestalSettings) -> Vec<String> {
    vec![
        format!("Pedestal Orbit: {}", if s.enabled { "ON" } else { "OFF" }),
        format!("Block Anchor: {}", s.block_label()),
        format!("Orbit Radius: {:.1} blocks", s.radius()),
        format!("Orbit Speed: {:.1}x", s.speed()),
        format!("Bob Amplitude: {:.1}m", s.bob_amplitude()),
        format!("HUD Telemetry: {}", if s.hud_on { "Shown" } else { "Hidden" }),
        format!("HUD Corner: {}", CORNERS[s.corner % CORNERS.len()].1),
    ]
}

pub mod host_bridge {
    use super::*;

    thread_local! {
        static KERNEL: RefCell<Option<PedestalKernel>> = const { RefCell::new(None) };
    }

    pub fn with_kernel<R>(f: impl FnOnce(&mut PedestalKernel) -> R) -> R {
        KERNEL.with(|cell| {
            let mut borrow = cell.borrow_mut();
            let kernel = borrow.get_or_insert_with(PedestalKernel::new);
            f(kernel)
        })
    }

    pub fn place_pedestal(pos: [i32; 3], block_type: &str, radius: f64, speed: f64, bob: f64) {
        with_kernel(|kernel| {
            kernel.place_pedestal(pos, block_type, radius, speed, bob);
        });
    }

    pub fn bind_or_toggle_mob(entity_id: u32, kind: &str, pos: [f64; 3]) -> String {
        with_kernel(|kernel| {
            if let Some(bound) = kernel.bound_mob() {
                if bound.entity_id == entity_id {
                    kernel.release_mob();
                    return format!("Released {}", bound.kind);
                }
            }

            if kernel.active_pedestal().is_none() {
                let default_pos = [pos[0].floor() as i32, pos[1].floor() as i32 - 1, pos[2].floor() as i32];
                kernel.place_pedestal(default_pos, BLOCKS[0].0, RADII[1], SPEEDS[1], BOBS[1]);
            }

            kernel.bind_mob(entity_id, kind, pos);
            format!("Bound {} #{entity_id} to Pedestal Orbit", kind)
        })
    }

    pub fn release_mob() -> Option<u32> {
        with_kernel(|kernel| kernel.release_mob())
    }

    pub fn active_pedestal() -> Option<[i32; 3]> {
        with_kernel(|kernel| kernel.active_pedestal())
    }

    pub fn bound_mob() -> Option<BoundMobState> {
        with_kernel(|kernel| kernel.bound_mob())
    }

    pub fn step_simulation(
        player_id: u32,
        player_pos: [f64; 3],
        player_rot: [f32; 2],
        player_hp: f32,
        player_max_hp: f32,
        live_mob: Option<([f64; 3], f32, f32)>,
    ) -> Vec<bevy_horizon::prelude::OutboxCommand> {
        with_kernel(|kernel| {
            kernel.bridge.ingest_player(
                player_id,
                player_pos,
                player_rot,
                player_hp,
                player_max_hp,
            );

            if let Some(bound) = kernel.bound_mob() {
                if let Some((pos, hp, max_hp)) = live_mob {
                    kernel.bridge.ingest_mob(bound.entity_id, &bound.kind, pos, hp, max_hp);
                }
            }

            kernel.step_frame(0.05);
            kernel.bridge.drain_outbox()
        })
    }

    pub fn latest_hud_label() -> Option<String> {
        with_kernel(|kernel| {
            let state = kernel.bridge.world.resource::<PedestalStateResource>();
            let config = kernel.bridge.world.resource::<OrbitConfigResource>();
            if !config.enabled {
                return None;
            }

            Some(if let Some(pedestal_pos) = state.active_pedestal_pos {
                if let Some(ref mob) = state.bound_mob {
                    format!(
                        "Pedestal: ({}, {}, {}) | Mob: {} #{} | R: {:.1}m",
                        pedestal_pos[0], pedestal_pos[1], pedestal_pos[2], mob.kind, mob.entity_id, config.radius
                    )
                } else {
                    format!(
                        "Pedestal: ({}, {}, {}) | Ready to Bind",
                        pedestal_pos[0], pedestal_pos[1], pedestal_pos[2]
                    )
                }
            } else {
                "Pedestal: None (Press 'P' to Anchor)".to_string()
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedestal_kernel_lifecycle() {
        let mut kernel = PedestalKernel::new();
        assert_eq!(kernel.active_pedestal(), None);
        assert_eq!(kernel.bound_mob(), None);

        // Place pedestal
        kernel.place_pedestal([10, 64, -20], "minecraft:diamond_block", 4.0, 1.5, 0.6);
        assert_eq!(kernel.active_pedestal(), Some([10, 64, -20]));

        // Bind mob
        kernel.bind_mob(55, "minecraft:zombie", [10.0, 65.0, -20.0]);
        assert_eq!(kernel.bound_mob().unwrap().entity_id, 55);

        // Step frames and verify outbox emission
        kernel.step_frame(0.05);
        let outbox = kernel.bridge.drain_outbox();
        assert!(!outbox.is_empty());

        let has_teleport = outbox.iter().any(|cmd| match cmd {
            bevy_horizon::prelude::OutboxCommand::TeleportMob { entity_id, .. } => *entity_id == 55,
            _ => false,
        });
        assert!(has_teleport);

        // Release mob
        let released = kernel.release_mob();
        assert_eq!(released, Some(55));
        assert_eq!(kernel.bound_mob(), None);
    }

    #[test]
    fn test_settings_cycle() {
        let mut s = PedestalSettings::default();
        assert_eq!(s.block_label(), "Diamond Block");
        assert_eq!(s.radius(), 3.5);

        s.click(1); // Cycle block
        assert_eq!(s.block_label(), "Gold Block");

        s.click(2); // Cycle radius
        assert_eq!(s.radius(), 5.0);

        let labels = settings_labels(&s);
        assert!(labels.iter().any(|l| l.contains("Gold Block")));
        assert!(labels.iter().any(|l| l.contains("5.0 blocks")));
    }
}
