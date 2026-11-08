use bevy_ecs::prelude::*;


/// Fired when the player interacts with (right-clicks) an entity.
#[derive(Event, Clone, Debug, PartialEq)]
pub struct EntityInteractEvent {
    pub entity_id: u32,
    pub kind: String,
    pub stack_name: Option<String>,
    pub claimed: bool,
}

impl EntityInteractEvent {
    pub fn new(entity_id: u32, kind: impl Into<String>, stack_name: Option<String>) -> Self {
        Self {
            entity_id,
            kind: kind.into(),
            stack_name,
            claimed: false,
        }
    }

    pub fn claim(&mut self) {
        self.claimed = true;
    }
}

/// Fired on keyboard input actions.
#[derive(Event, Clone, Debug, PartialEq, Eq)]
pub struct HorizonKeyEvent {
    pub key: String,
    pub pressed: bool,
}

impl HorizonKeyEvent {
    pub fn new(key: impl Into<String>, pressed: bool) -> Self {
        Self {
            key: key.into(),
            pressed,
        }
    }
}

/// Fired every frame during the HUD rendering pass.
#[derive(Event, Clone, Copy, Debug, PartialEq)]
pub struct HudDrawEvent {
    pub delta_secs: f64,
}

impl HudDrawEvent {
    pub fn new(delta_secs: f64) -> Self {
        Self { delta_secs }
    }
}

/// Fired on every game simulation tick (nominally 20 Hz).
#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerTickEvent {
    pub tick: u64,
}

impl PlayerTickEvent {
    pub fn new(tick: u64) -> Self {
        Self { tick }
    }
}

/// Fired when an anchor or pedestal block type is selected by configuration or GUI.
#[derive(Event, Clone, Debug, PartialEq, Eq)]
pub struct BlockSelectEvent {
    pub block_type: String,
}

impl BlockSelectEvent {
    pub fn new(block_type: impl Into<String>) -> Self {
        Self {
            block_type: block_type.into(),
        }
    }
}

/// Fired when a block is placed into the world grid.
#[derive(Event, Clone, Debug, PartialEq, Eq)]
pub struct BlockPlaceEvent {
    pub dimension: i32,
    pub pos: [i32; 3],
    pub block_type: String,
}

impl BlockPlaceEvent {
    pub fn new(dimension: i32, pos: [i32; 3], block_type: impl Into<String>) -> Self {
        Self {
            dimension,
            pos,
            block_type: block_type.into(),
        }
    }
}
