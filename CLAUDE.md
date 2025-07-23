# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Bevy-based 3D craps dice game simulation using Rust. The project uses Bevy 0.16.1 for the game engine and bevy_rapier3d 0.30.0 for physics simulation.

## Development Commands

### Build and Run
```bash
# Build the project
cargo build

# Run the game
cargo run

# Build optimized release version
cargo build --release

# Run release version
cargo run --release
```

### Development and Testing
```bash
# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Run with debug logging
RUST_LOG=debug cargo run
```

## Important Version Note

**This project uses Bevy 0.16.1** - ensure all code follows Bevy 0.16 syntax and APIs. Do not use deprecated patterns from older Bevy versions such as:
- Old bundle syntax (use components directly)
- Deprecated color APIs (use `bevy::color::prelude::*` and `Srgba`)
- Old camera/projection syntax (use `Camera3d` and `Projection::from()`)

## Architecture

The game is a single-file application (`src/main.rs`) that implements:

- **Camera System**: First-person camera with mouse-look controls (right mouse button)
- **Physics**: Uses Rapier3D for realistic dice physics including collisions and restitution
- **Throw System**: Space bar charges power meter, release throws two dice with physics impulses
- **Game Table**: Black table surface with orange walls, physics boundaries to contain dice

### Key Components:
- `PlayerCamera`: Tracks camera yaw/pitch for mouse-look controls
- `ThrowPower`: Resource managing the power charging system
- `Dice` and `DiceId`: Component markers for spawned dice entities
- `PowerMeterFill`: UI component for the throw power visualization

### Physics Configuration:
- Table has high restitution (0.8) for bouncing
- Walls have low restitution (0.08) for absorption
- Dice use CCD (Continuous Collision Detection) to prevent tunneling
- Linear and angular damping applied to dice for realistic settling