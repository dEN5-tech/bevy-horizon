use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

/// High precision game clock tracking ticks and elapsed simulation seconds.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HorizonClock {
    pub tick: u64,
    pub time_secs: f64,
    pub delta_secs: f64,
}

impl HorizonClock {
    pub const fn new(tick: u64, time_secs: f64, delta_secs: f64) -> Self {
        Self {
            tick,
            time_secs,
            delta_secs,
        }
    }

    /// Advances the clock forward by `delta_secs` and increments the tick count.
    #[inline]
    pub fn advance(&mut self, delta_secs: f64) {
        self.tick = self.tick.saturating_add(1);
        self.delta_secs = delta_secs;
        self.time_secs += delta_secs;
    }
}

/// Screen display parameters and active GUI screen metadata.
#[derive(Resource, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ScreenState {
    pub is_in_game: bool,
    pub screen_width: f32,
    pub screen_height: f32,
    pub active_screen: Option<String>,
}

impl ScreenState {
    pub fn new(width: f32, height: f32, is_in_game: bool) -> Self {
        Self {
            is_in_game,
            screen_width: width,
            screen_height: height,
            active_screen: None,
        }
    }
}

/// Low-level graphics primitives emitted during the HUD draw phase.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HudDrawCommand {
    Text {
        text: String,
        x: f32,
        y: f32,
        color_rgb: u32,
        shadow: bool,
    },
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color_argb: u32,
    },
}

/// Buffer accumulating HUD draw calls to be dispatched into Horizon's native HUD pipeline.
#[derive(Resource, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HudDrawBuffer {
    pub commands: Vec<HudDrawCommand>,
}

impl HudDrawBuffer {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn draw_text(&mut self, text: impl Into<String>, x: f32, y: f32, color_rgb: u32, shadow: bool) {
        self.commands.push(HudDrawCommand::Text {
            text: text.into(),
            x,
            y,
            color_rgb,
            shadow,
        });
    }

    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color_argb: u32) {
        self.commands.push(HudDrawCommand::Rect {
            x,
            y,
            w,
            h,
            color_argb,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn drain_commands(&mut self) -> Vec<HudDrawCommand> {
        std::mem::take(&mut self.commands)
    }
}

/// Outgoing command intent targeting Minecraft engine hooks.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum OutboxCommand {
    TeleportMob {
        dimension: i32,
        entity_id: u32,
        pos: [f64; 3],
    },
    PlaceBlock {
        dimension: i32,
        pos: [i32; 3],
        block_id: String,
    },
    PushMob {
        dimension: i32,
        entity_id: u32,
        velocity: [f64; 3],
    },
    ActionBar {
        message: String,
    },
    Chat {
        message: String,
    },
}

/// Outbox command queue drained after schedule execution to mutate the Minecraft host.
#[derive(Resource, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HorizonOutboxCommands {
    commands: Vec<OutboxCommand>,
}

impl HorizonOutboxCommands {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn teleport_mob(&mut self, dimension: i32, entity_id: u32, pos: [f64; 3]) {
        self.commands.push(OutboxCommand::TeleportMob {
            dimension,
            entity_id,
            pos,
        });
    }

    pub fn place_block(&mut self, dimension: i32, pos: [i32; 3], block_id: impl Into<String>) {
        self.commands.push(OutboxCommand::PlaceBlock {
            dimension,
            pos,
            block_id: block_id.into(),
        });
    }

    pub fn push_mob(&mut self, dimension: i32, entity_id: u32, velocity: [f64; 3]) {
        self.commands.push(OutboxCommand::PushMob {
            dimension,
            entity_id,
            velocity,
        });
    }

    pub fn action_bar(&mut self, message: impl Into<String>) {
        self.commands.push(OutboxCommand::ActionBar {
            message: message.into(),
        });
    }

    pub fn chat(&mut self, message: impl Into<String>) {
        self.commands.push(OutboxCommand::Chat {
            message: message.into(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn drain_commands(&mut self) -> Vec<OutboxCommand> {
        std::mem::take(&mut self.commands)
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}
