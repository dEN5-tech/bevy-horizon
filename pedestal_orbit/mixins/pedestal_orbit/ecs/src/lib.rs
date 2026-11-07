use bevy_horizon::prelude::*;
use serde::{Deserialize, Serialize};

pub const BLOCKS: [(&str, &str); 5] = [
    ("minecraft:diamond_block", "Diamond Block"),
    ("minecraft:gold_block", "Gold Block"),
    ("minecraft:obsidian", "Obsidian"),
    ("minecraft:beacon", "Beacon"),
    ("minecraft:emerald_block", "Emerald Block"),
];

pub const RADII: [f64; 4] = [2.0, 3.5, 5.0, 7.0];
pub const SPEEDS: [f64; 4] = [0.5, 1.0, 1.5, 2.5];
pub const BOBS: [f64; 4] = [0.2, 0.5, 0.8, 1.2];

pub const CORNERS: [(&str, &str); 4] = [
    ("top_left", "Top left"),
    ("top_right", "Top right"),
    ("bottom_left", "Bottom left"),
    ("bottom_right", "Bottom right"),
];

/// Configuration settings for the Pedestal Orbit mod.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct PedestalSettings {
    pub enabled: bool,
    pub block_idx: usize,
    pub radius_idx: usize,
    pub speed_idx: usize,
    pub bob_idx: usize,
    pub hud_on: bool,
    pub corner: usize,
}

impl Default for PedestalSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            block_idx: 0,   // Diamond Block
            radius_idx: 1,  // 3.5 blocks
            speed_idx: 1,   // 1.0x
            bob_idx: 1,     // 0.5 amplitude
            hud_on: true,
            corner: 0,      // top_left
        }
    }
}

impl PedestalSettings {
    pub fn block_id(&self) -> &'static str {
        BLOCKS[self.block_idx % BLOCKS.len()].0
    }

    pub fn block_label(&self) -> &'static str {
        BLOCKS[self.block_idx % BLOCKS.len()].1
    }

    pub fn radius(&self) -> f64 {
        RADII[self.radius_idx % RADII.len()]
    }

    pub fn speed(&self) -> f64 {
        SPEEDS[self.speed_idx % SPEEDS.len()]
    }

    pub fn bob_amplitude(&self) -> f64 {
        BOBS[self.bob_idx % BOBS.len()]
    }

    pub fn click(&mut self, button: usize) -> Option<(&'static str, String)> {
        match button {
            0 => {
                self.enabled = !self.enabled;
                Some(("enabled", self.enabled.to_string()))
            }
            1 => {
                self.block_idx = (self.block_idx + 1) % BLOCKS.len();
                Some(("block", self.block_id().to_string()))
            }
            2 => {
                self.radius_idx = (self.radius_idx + 1) % RADII.len();
                Some(("radius", format!("{:.1}", self.radius())))
            }
            3 => {
                self.speed_idx = (self.speed_idx + 1) % SPEEDS.len();
                Some(("speed", format!("{:.1}", self.speed())))
            }
            4 => {
                self.bob_idx = (self.bob_idx + 1) % BOBS.len();
                Some(("bob", format!("{:.1}", self.bob_amplitude())))
            }
            5 => {
                self.hud_on = !self.hud_on;
                Some(("hud_on", self.hud_on.to_string()))
            }
            6 => {
                self.corner = (self.corner + 1) % CORNERS.len();
                Some(("corner", CORNERS[self.corner].0.to_string()))
            }
            _ => None,
        }
    }
}

/// Metadata describing an active living mob bound to orbit the pedestal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoundMobState {
    pub entity_id: u32,
    pub kind: String,
    pub last_pos: [f64; 3],
}

/// Marker component attached to the active pedestal anchor entity.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActivePedestalAnchor;

/// Component attached to the orbiting entity tracking its bound Minecraft mob ID.
#[derive(Component, Clone, Debug, PartialEq, Eq)]
pub struct BoundOrbitTarget {
    pub minecraft_id: u32,
    pub kind: String,
}

/// Resource tracking the global state of the pedestal and bound mob.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct PedestalStateResource {
    pub active_pedestal_pos: Option<[i32; 3]>,
    pub bound_mob: Option<BoundMobState>,
    pub current_block_type: String,
}

/// Resource managing runtime orbit parameters.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct OrbitConfigResource {
    pub enabled: bool,
    pub radius: f64,
    pub speed: f64,
    pub bob_amplitude: f64,
    pub bob_frequency: f64,
}

impl Default for OrbitConfigResource {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 3.5,
            speed: 1.0,
            bob_amplitude: 0.5,
            bob_frequency: 2.0,
        }
    }
}

/// System synchronizing the computed orbit position to the bound Minecraft mob via outbox.
pub fn pedestal_teleport_sync_system(
    config: Res<OrbitConfigResource>,
    mut state: ResMut<PedestalStateResource>,
    query: Query<(&Position, &BoundOrbitTarget)>,
    mut outbox: ResMut<HorizonOutboxCommands>,
) {
    if !config.enabled {
        return;
    }

    for (pos, target) in &query {
        let current_pos = pos.as_array();
        outbox.teleport_mob(0, target.minecraft_id, current_pos);

        if let Some(ref mut bound) = state.bound_mob {
            if bound.entity_id == target.minecraft_id {
                bound.last_pos = current_pos;
            }
        }
    }
}

/// System generating HUD telemetry summary into the HUD draw buffer.
pub fn pedestal_hud_telemetry_system(
    config: Res<OrbitConfigResource>,
    state: Res<PedestalStateResource>,
    mut hud: ResMut<HudDrawBuffer>,
) {
    if !config.enabled {
        return;
    }

    let summary = if let Some(pedestal_pos) = state.active_pedestal_pos {
        if let Some(ref mob) = state.bound_mob {
            format!(
                "[PedestalOrbit] Pedestal: ({}, {}, {}) | Mob: {} #{} | R: {:.1}m",
                pedestal_pos[0], pedestal_pos[1], pedestal_pos[2], mob.kind, mob.entity_id, config.radius
            )
        } else {
            format!(
                "[PedestalOrbit] Pedestal: ({}, {}, {}) | Ready to Bind (Right-click mob)",
                pedestal_pos[0], pedestal_pos[1], pedestal_pos[2]
            )
        }
    } else {
        "[PedestalOrbit] Ready: Press 'P' to place pedestal anchor".to_string()
    };

    hud.draw_text(summary, 6.0, 6.0, 0x00FFCC, true);
}

/// Headless ECS Kernel for Pedestal Orbit.
pub struct PedestalKernel {
    pub bridge: HorizonBridge,
    orbit_entity: Option<Entity>,
}

impl Default for PedestalKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl PedestalKernel {
    pub fn new() -> Self {
        let mut bridge = HorizonBridge::new();

        bridge.world.insert_resource(PedestalStateResource {
            active_pedestal_pos: None,
            bound_mob: None,
            current_block_type: BLOCKS[0].0.to_string(),
        });
        bridge.world.insert_resource(OrbitConfigResource::default());

        // Register custom systems
        bridge.schedule.add_systems((
            pedestal_teleport_sync_system,
            pedestal_hud_telemetry_system,
        ));

        Self {
            bridge,
            orbit_entity: None,
        }
    }

    /// Sets or updates the active pedestal anchor block and centers the orbit.
    pub fn place_pedestal(&mut self, pos: [i32; 3], block_type: &str, radius: f64, speed: f64, bob: f64) {
        let mut state = self.bridge.world.resource_mut::<PedestalStateResource>();
        state.active_pedestal_pos = Some(pos);
        state.current_block_type = block_type.to_string();

        let mut config = self.bridge.world.resource_mut::<OrbitConfigResource>();
        config.radius = radius;
        config.speed = speed;
        config.bob_amplitude = bob;

        // Place block in Minecraft world via outbox
        self.bridge
            .world
            .resource_mut::<HorizonOutboxCommands>()
            .place_block(0, pos, block_type);

        let pedestal = PedestalBlock::new(block_type, pos);
        let center = pedestal.anchor_center();

        if let Some(orbit_ent) = self.orbit_entity {
            if let Ok(mut entity_mut) = self.bridge.world.get_entity_mut(orbit_ent) {
                if let Some(mut orb) = entity_mut.get_mut::<OrbitingEntity>() {
                    orb.center_x = center[0];
                    orb.center_y = center[1];
                    orb.center_z = center[2];
                    orb.radius = radius;
                    orb.speed = speed;
                    orb.bob_amplitude = bob;
                }
            }
        } else {
            let mut orb = OrbitingEntity::new(center, radius, speed);
            orb.bob_amplitude = bob;
            let initial_pos = Position::from_array(orb.calculate_position(0.0));
            let ent = self
                .bridge
                .world
                .spawn((
                    ActivePedestalAnchor,
                    PedestalBlock::new(block_type, pos),
                    orb,
                    initial_pos,
                ))
                .id();
            self.orbit_entity = Some(ent);
        }
    }

    /// Binds an entity to the pedestal orbit ring.
    pub fn bind_mob(&mut self, entity_id: u32, kind: &str, pos: [f64; 3]) {
        let mut state = self.bridge.world.resource_mut::<PedestalStateResource>();
        state.bound_mob = Some(BoundMobState {
            entity_id,
            kind: kind.to_string(),
            last_pos: pos,
        });

        if let Some(orbit_ent) = self.orbit_entity {
            if let Ok(mut entity_mut) = self.bridge.world.get_entity_mut(orbit_ent) {
                entity_mut.insert(BoundOrbitTarget {
                    minecraft_id: entity_id,
                    kind: kind.to_string(),
                });
            }
        }
    }

    /// Releases any bound mob from the orbit.
    pub fn release_mob(&mut self) -> Option<u32> {
        let mut state = self.bridge.world.resource_mut::<PedestalStateResource>();
        let prev = state.bound_mob.take().map(|m| m.entity_id);

        if let Some(orbit_ent) = self.orbit_entity {
            if let Ok(mut entity_mut) = self.bridge.world.get_entity_mut(orbit_ent) {
                entity_mut.remove::<BoundOrbitTarget>();
            }
        }
        prev
    }

    /// Advances simulation frame and updates orbit kinematics.
    pub fn step_frame(&mut self, delta_secs: f64) {
        self.bridge.step_tick(delta_secs);
    }

    pub fn active_pedestal(&self) -> Option<[i32; 3]> {
        self.bridge.world.resource::<PedestalStateResource>().active_pedestal_pos
    }

    pub fn bound_mob(&self) -> Option<BoundMobState> {
        self.bridge.world.resource::<PedestalStateResource>().bound_mob.clone()
    }
}
