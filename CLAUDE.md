# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Resort Tycoon is a hotel/resort tycoon game built with Bevy ECS (version 0.15), inspired by RimWorld. It uses an ASCII rendering system (designed to be swapped for sprites later) and implements a finer-grained tile system than RimWorld for more detailed building placement.

## Build Commands

```bash
# Run in development mode (with optimizations for dependencies)
cargo run

# Run in release mode with LTO optimizations
cargo run --release
```

The project uses optimized dependencies in dev builds (`opt-level = 3` for dependencies, `opt-level = 1` for the crate) for faster iteration.

## Architecture Overview

### ECS Plugin System

The game is structured using Bevy's plugin architecture. All major systems are organized as plugins registered in `src/main.rs`:

- **GridPlugin**: Grid rendering and coordinate conversion utilities
- **ToolbarPlugin**: Bottom construction menu UI
- **SpeedControlPlugin**: Game speed controls
- **MoneyDisplayPlugin**: Economy UI
- **BuildingPlugin**: Building placement and collision detection
- **PawnPlugin**: Worker spawning and movement
- **WorkPlugin**: Job assignment and construction work
- **AsciiRendererPlugin**: ASCII character rendering for all entities
- **TimeControlPlugin**: Game time simulation
- **EconomyPlugin**: Resource and money management

### Tile System Design

**Critical**: This game uses a finer-grained tile system than RimWorld:
- Base tile size: 16 pixels (defined in `src/systems/grid.rs::TILE_SIZE`)
- Pawn footprint: **2x2 tiles** (not 1x1 like RimWorld)
- Grid size: 100x100 tiles
- All entities use `GridPosition` component for tile-based positioning
- Coordinate conversion: Use `world_to_grid()` and `grid_to_world()` from `src/systems/grid.rs`

### Blueprint and Construction System

Construction works through a multi-stage process:

1. User selects building type from toolbar → creates placement preview
2. User clicks to confirm → spawns `Blueprint` component with `ConstructionJob`
3. Idle pawns (via `CurrentJob` component) find nearest unassigned job
4. Pawn moves to blueprint location (via `MovementTarget` component)
5. Pawn works on blueprint (accumulates `work_done` at `work_speed` per second)
6. When `work_done >= work_required`, blueprint transforms into finished building

**Key files:**
- `src/components/work.rs`: Blueprint, ConstructionJob, WorkInProgress components
- `src/systems/work.rs`: assign_jobs_to_pawns, work_on_blueprints, complete_blueprints

### Building Collision System

The `BuildingMap` resource (in `src/systems/building.rs`) tracks occupied tiles:
- Prevents overlapping building placement
- Multi-tile buildings (like doors) register all occupied tiles
- Check collision before spawning blueprints

### Component Organization

- **components/building.rs**: GridPosition, Wall, Door, Window, Floor, Building markers
- **components/pawn.rs**: Pawn, MovementTarget, CurrentJob
- **components/work.rs**: Blueprint, ConstructionJob, WorkInProgress
- **components/furniture.rs**: Future furniture components

### ASCII Rendering

All entities render as ASCII characters (src/systems/ascii_renderer.rs):
- `@` = Pawn
- `#` = Wall
- `+` = Door
- `=` = Window
- `▒` = Blueprint (under construction)
- Characters and colors are component-based for easy sprite replacement

## Important Patterns

### Adding New Building Types

1. Add component in `src/components/building.rs` (or furniture.rs)
2. Add to `BlueprintType` enum in `src/components/work.rs`
3. Set work_required in `Blueprint::new()`
4. Add toolbar button in `src/ui/toolbar.rs`
5. Handle placement in `src/systems/building.rs`
6. Add ASCII character in `src/systems/ascii_renderer.rs`
7. Update completion logic in `complete_blueprints()` system

### Coordinate Conversion

Always use the grid helper functions for consistency:
```rust
// World → Grid (returns Option<IVec2>)
world_to_grid(world_pos, tile_size, grid_width, grid_height)

// Grid → World (returns Vec2, always valid)
grid_to_world(grid_pos, tile_size, grid_width, grid_height)
```

### Multi-tile Buildings

Buildings that occupy multiple tiles (doors, furniture):
1. Implement a method returning occupied tiles (see `Door::tiles_occupied()`)
2. Register ALL tiles in BuildingMap
3. Check ALL tiles during placement collision detection

## Current State

Based on git status, recent work includes:
- Time control and speed systems
- Economy/money display
- Furniture component system
- Building system enhancements

The project is in active development with working pawn AI, construction system, and UI foundation.
