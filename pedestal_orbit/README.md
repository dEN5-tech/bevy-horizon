# Pedestal Orbit (Horizon Native Mod)

Reference native Horizon mod implemented with **Bevy ECS** (`bevy_horizon`).

## Features
- **Pedestal Anchoring**: Place an anchor block (Diamond Block, Gold Block, Obsidian, Beacon, Emerald Block).
- **Smooth Orbiting & Bobbing**: Orbiting entities follow harmonic trajectory:
  $$x = x_0 + R \cos(\theta),\quad y = y_0 + A \sin(\omega t),\quad z = z_0 + R \sin(\theta)$$
- **Entity Binding**: Right-click any living Minecraft mob to tether it to the pedestal orbit ring.
- **HUD Telemetry Overlay**: Real-time display of orbit phase, bound target, and pedestal position.
