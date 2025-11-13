# Resort Tycoon

A hotel/resort tycoon game inspired by RimWorld, built with Bevy.

## Concept

Focus on building and managing a resort/hotel rather than survival. The game uses a finer-grained tile system compared to RimWorld for more detailed placement of smaller objects and installations.

## Tile System

- **Base tile size**: 1/4th the size of a pawn's standing area (16 pixels)
- **Pawn footprint**: 2x2 tiles (compared to RimWorld's 1x1)
- This allows for more granular building placement and smaller decorative objects

## Features Implemented

### UI System
- **Bottom Toolbar**: RimWorld-style construction menu at the bottom of the screen
- **Tab Categories**: Structure, Furniture, Decoration, Floors
- **Building Selection**: Click tabs to open construction options, click buttons to select buildings

### Grid System
- Visual grid overlay (100x100 tiles)
- Grid coordinate conversion utilities
- Tile-based positioning for all entities

### Pawn System
- Worker pawns that move around the map
- Automatic job assignment to idle pawns
- Pathfinding to construction sites
- Currently spawns 3 initial workers

### Work & Construction System
- **Blueprint System**: Buildings start as blueprints that must be constructed
- **Work Assignment**: Idle pawns automatically find and take construction jobs
- **Construction Progress**: Pawns work on nearby blueprints, progress shown visually
- **Job Completion**: Blueprints transform into finished buildings when complete

### Building Types
- **Walls**: Basic stone walls (character: `#`)
- **Doors**: Doorways for access (character: `+`)
- **Windows**: Window openings (character: `=`)

### ASCII Rendering
- All entities rendered with ASCII characters
- Easy to swap for sprite-based graphics later
- Characters:
  - `@` = Pawn/Worker
  - `#` = Wall
  - `+` = Door
  - `=` = Window
  - `▒` = Blueprint (under construction)

## Controls

- **Mouse**: Navigate construction menus and place buildings
- **Left Click**: Select tabs/buttons, place blueprints

## Development

```bash
# Run the game
cargo run

# Build for release
cargo run --release
```

## Project Structure

```
src/
├── main.rs              # App entry point
├── components/          # ECS components
│   ├── building.rs      # Building-related components
│   ├── pawn.rs          # Pawn components
│   └── work.rs          # Work/job components
├── systems/             # ECS systems
│   ├── grid.rs          # Grid rendering & utilities
│   ├── building.rs      # Building placement
│   ├── pawn.rs          # Pawn movement & spawning
│   ├── work.rs          # Job assignment & construction
│   └── ascii_renderer.rs # ASCII character rendering
└── ui/                  # UI systems
    └── toolbar.rs       # Bottom construction toolbar
```

## Roadmap

- [ ] Camera pan and zoom controls
- [ ] Guest AI system
- [ ] Room quality/ratings system
- [ ] Resource management
- [ ] More building types (furniture, decorations, floors)
- [ ] Save/load system
- [ ] Replace ASCII with sprite graphics
