# ClickerCurse Game

A Cthulhu-themed incremental/clicker game that runs in your terminal. Summon eldritch horrors, build your cult, and bring about the end of the world... one click at a time!

## Features

- Terminal-based UI using crossterm
- Multiple Cthulhu-themed buildings to purchase
- Upgrades to increase production
- Automatic saving and loading
- Different progression tiers with increasing click power
- Multiple menu screens for buildings and upgrades

## How to Play

### Controls

- `.` - Click to gain points
- `1` - Switch to Main Menu
- `2` - Switch to Buildings Menu
- `3` - Switch to Upgrades Menu
- `↑/↓` - Navigate menus
- `Enter` - Select/buy the highlighted item
- `s` - Manually save the game
- `Ctrl+C` - Quit the game

### Game Mechanics

#### Buildings

- **Cultist** - Whispers eldritch secrets
- **Elder One** - Ancient being from beyond
- **Ritual Site** - Conducts forbidden ceremonies
- **Deep One Colony** - Underwater servants of Cthulhu
- **Temple of Dagon** - Ancient place of worship
- **Dimensional Portal** - Gateway to R'lyeh

Each building produces points automatically over time. The more buildings you have, the more points you generate.

#### Upgrades

Upgrades can increase the efficiency of specific buildings or improve your click power. Some notable upgrades include:

- **Necronomicon Pages** - Cultists are twice as efficient
- **Eldritch Incantation** - Your influence is twice as powerful
- **The Stars Are Right** - All minions are twice as efficient

#### Progression

As you accumulate lifetime points, your click power will automatically increase:

- 1,000 points: 2x click power
- 10,000 points: 5x click power
- 100,000 points: 10x click power
- 1,000,000 points: 25x click power
- 10,000,000 points: 50x click power
- 100,000,000+ points: 100x click power

## Installation

### Prerequisites

- Rust and Cargo installed on your system

### Building from Source

1. Clone the repository:
   ```
   git clone https://github.com/yourusername/clickercurse-game.git
   cd clickercurse-game
   ```

2. Build and run the game:
   ```
   cargo run --release
   ```

## Dependencies

- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation library
- [tokio](https://github.com/tokio-rs/tokio) - Asynchronous runtime

## Save Files

Game progress is automatically saved every 30 seconds to `saves/game.save`. The save file contains your current points, lifetime points, buildings, and upgrades.

The save directory is now included in `.gitignore` to prevent save files from being tracked by git.

## License

This project is open source and available under the [MIT License](LICENSE).