use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{self, Stylize},
    terminal::{self, ClearType},
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{stdout, Read, Write},
    path::Path,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use std::io::Result as IoResult;

// Game structures
#[derive(Clone, Debug)]
struct Building {
    name: String,
    description: String,
    base_cost: u64,
    base_production: f64,
    count: u64,
    cost_multiplier: f64,
}

impl Building {
    fn new(name: &str, description: &str, base_cost: u64, base_production: f64, cost_multiplier: f64) -> Self {
        Building {
            name: name.to_string(),
            description: description.to_string(),
            base_cost,
            base_production,
            count: 0,
            cost_multiplier,
        }
    }

    fn current_cost(&self) -> u64 {
        if self.count == 0 {
            return self.base_cost;
        }
        (self.base_cost as f64 * self.cost_multiplier.powf(self.count as f64)) as u64
    }

    fn total_production(&self) -> f64 {
        self.base_production * self.count as f64
    }

    fn buy(&mut self) -> u64 {
        let cost = self.current_cost();
        self.count += 1;
        cost
    }
}

#[derive(Clone, Debug)]
struct Upgrade {
    name: String,
    description: String,
    cost: u64,
    purchased: bool,
    building_multiplier: Option<(String, f64)>,
    click_multiplier: Option<f64>,
}

impl Upgrade {
    fn new(
        name: &str,
        description: &str,
        cost: u64,
        building_multiplier: Option<(String, f64)>,
        click_multiplier: Option<f64>,
    ) -> Self {
        Upgrade {
            name: name.to_string(),
            description: description.to_string(),
            cost,
            purchased: false,
            building_multiplier,
            click_multiplier,
        }
    }
}

#[derive(Clone, Debug)]
struct GameState {
    points: u64,
    lifetime_points: u64,
    click_power: u64,
    buildings: HashMap<String, Building>,
    upgrades: Vec<Upgrade>,
    current_menu: Menu,
    selected_index: usize,
    production_remainder: f64, // Track fractional production
}

#[derive(Clone, Debug, PartialEq)]
enum Menu {
    Main,
    Buildings,
    Upgrades,
}

impl GameState {
    fn new() -> Self {
        let mut buildings = HashMap::new();
        let production_remainder = 0.0;
        
        // Add Cthulhu-themed buildings
        buildings.insert(
            "cursor".to_string(),
            Building::new("Cultist", "Whispers eldritch secrets", 15, 0.1, 1.15),
        );
        buildings.insert(
            "grandma".to_string(),
            Building::new("Elder One", "Ancient being from beyond", 100, 1.0, 1.15),
        );
        buildings.insert(
            "farm".to_string(),
            Building::new("Ritual Site", "Conducts forbidden ceremonies", 1100, 8.0, 1.15),
        );
        buildings.insert(
            "mine".to_string(),
            Building::new("Deep One Colony", "Underwater servants of Cthulhu", 12000, 47.0, 1.15),
        );
        buildings.insert(
            "temple".to_string(),
            Building::new("Temple of Dagon", "Ancient place of worship", 130000, 260.0, 1.15),
        );
        buildings.insert(
            "portal".to_string(),
            Building::new("Dimensional Portal", "Gateway to R'lyeh", 1400000, 1400.0, 1.15),
        );
        
        // Create Cthulhu-themed upgrades
        let upgrades = vec![
            Upgrade::new(
                "Necronomicon Pages",
                "Cultists are twice as efficient",
                100,
                Some(("cursor".to_string(), 2.0)),
                None,
            ),
            Upgrade::new(
                "Eldritch Incantation",
                "Your influence is twice as powerful",
                500,
                None,
                Some(2.0),
            ),
            Upgrade::new(
                "Ancient Artifacts",
                "Elder Ones are twice as efficient",
                1000,
                Some(("grandma".to_string(), 2.0)),
                None,
            ),
            Upgrade::new(
                "Blood Sacrifice",
                "Ritual Sites are twice as efficient",
                11000,
                Some(("farm".to_string(), 2.0)),
                None,
            ),
            Upgrade::new(
                "Esoteric Geometry",
                "Deep One Colonies are twice as efficient",
                120000,
                Some(("mine".to_string(), 2.0)),
                None,
            ),
            Upgrade::new(
                "Non-Euclidean Architecture",
                "Temples of Dagon are twice as efficient",
                1300000,
                Some(("temple".to_string(), 2.0)),
                None,
            ),
            Upgrade::new(
                "The Stars Are Right",
                "All minions are twice as efficient",
                10000000,
                Some(("all".to_string(), 2.0)),
                Some(5.0),
            ),
        ];
        
        GameState {
            points: 0,
            lifetime_points: 0,
            click_power: 1,
            buildings,
            upgrades,
            current_menu: Menu::Main,
            selected_index: 0,
            production_remainder: 0.0,
        }
    }
    
    fn calculate_production_per_second(&self) -> f64 {
        let mut total = 0.0;
        let mut all_buildings_multiplier = 1.0;
        
        // First check for "all" building multipliers
        for upgrade in &self.upgrades {
            if upgrade.purchased {
                if let Some((building_key, building_mult)) = &upgrade.building_multiplier {
                    if building_key == "all" {
                        all_buildings_multiplier *= building_mult;
                    }
                }
            }
        }
        
        for (key, building) in &self.buildings {
            let mut multiplier = all_buildings_multiplier;
            
            // Apply specific building upgrades
            for upgrade in &self.upgrades {
                if upgrade.purchased {
                    if let Some((building_key, building_mult)) = &upgrade.building_multiplier {
                        if building_key == key {
                            multiplier *= building_mult;
                        }
                    }
                }
            }
            
            total += building.total_production() * multiplier;
        }
        
        total
    }
    
    fn click(&mut self) {
        let mut click_multiplier = 1.0;
        
        // Apply click upgrades
        for upgrade in &self.upgrades {
            if upgrade.purchased {
                if let Some(mult) = upgrade.click_multiplier {
                    click_multiplier *= mult;
                }
            }
        }
        
        let points_to_add = (self.click_power as f64 * click_multiplier) as u64;
        self.points += points_to_add;
        self.lifetime_points += points_to_add;
        
        // Check if we should increase click power based on lifetime points
        self.check_click_power_upgrade();
    }
    
    fn check_click_power_upgrade(&mut self) {
        // Increase click power based on lifetime points milestones
        let new_click_power = match self.lifetime_points {
            0..=999 => 1,
            1000..=9999 => 2,
            10000..=99999 => 5,
            100000..=999999 => 10,
            1000000..=9999999 => 25,
            10000000..=99999999 => 50,
            _ => 100,
        };
        
        if new_click_power > self.click_power {
            self.click_power = new_click_power;
        }
    }
    
    fn buy_building(&mut self, key: &str) -> bool {
        if let Some(building) = self.buildings.get_mut(key) {
            let cost = building.current_cost();
            if self.points >= cost {
                self.points -= cost;
                building.buy();
                return true;
            }
        }
        false
    }
    
    fn buy_upgrade(&mut self, index: usize) -> bool {
        if index < self.upgrades.len() {
            let cost = self.upgrades[index].cost;
            if !self.upgrades[index].purchased && self.points >= cost {
                self.points -= cost;
                self.upgrades[index].purchased = true;
                return true;
            }
        }
        false
    }
    
    fn save_game(&self) -> IoResult<()> {
        // Simple save format - just save the key stats for now
        let save_dir = "saves";
        if !Path::new(save_dir).exists() {
            fs::create_dir(save_dir)?;
        }
        
        let mut file = File::create("saves/game.save")?;
        
        // Write points
        writeln!(file, "points:{}", self.points)?;
        writeln!(file, "lifetime:{}", self.lifetime_points)?;
        writeln!(file, "click_power:{}", self.click_power)?;
        
        // Write buildings
        for (key, building) in &self.buildings {
            writeln!(file, "building:{}:{}:{}", key, building.count, building.base_production)?;
        }
        
        // Write upgrades
        for (i, upgrade) in self.upgrades.iter().enumerate() {
            writeln!(file, "upgrade:{}:{}", i, upgrade.purchased)?;
        }
        
        Ok(())
    }
    
    fn load_game(&mut self) -> IoResult<()> {
        let path = Path::new("saves/game.save");
        if !path.exists() {
            return Ok(());
        }
        
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Reset production remainder when loading a game
        self.production_remainder = 0.0;
        
        for line in contents.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() < 2 {
                continue;
            }
            
            match parts[0] {
                "points" => {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        self.points = val;
                    }
                },
                "lifetime" => {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        self.lifetime_points = val;
                    }
                },
                "click_power" => {
                    if let Ok(val) = parts[1].parse::<u64>() {
                        self.click_power = val;
                    }
                },
                "building" => {
                    if parts.len() >= 4 {
                        let key = parts[1];
                        if let (Ok(count), Ok(_)) = (parts[2].parse::<u64>(), parts[3].parse::<f64>()) {
                            if let Some(building) = self.buildings.get_mut(key) {
                                building.count = count;
                            }
                        }
                    }
                },
                "upgrade" => {
                    if parts.len() >= 3 {
                        if let (Ok(index), Ok(purchased)) = (parts[1].parse::<usize>(), parts[2].parse::<bool>()) {
                            if index < self.upgrades.len() {
                                self.upgrades[index].purchased = purchased;
                            }
                        }
                    }
                },
                _ => {}
            }
        }
        
        // Check if click power should be upgraded based on lifetime points
        self.check_click_power_upgrade();
        
        Ok(())
    }
}

fn main() -> IoResult<()> {
    let mut stdout = stdout();

    // Setup terminal
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    // Initialize game state
    let game_state = Arc::new(Mutex::new(GameState::new()));
    
    // Try to load saved game
    {
        let mut state = game_state.lock().unwrap();
        let _ = state.load_game();
    }
    
    let running = Arc::new(Mutex::new(true));

    // Tick thread (production)
    {
        let game_state = Arc::clone(&game_state);
        let running = Arc::clone(&running);
        thread::spawn(move || {
            let mut last_time = std::time::Instant::now();
            
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_millis(100));
                
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(last_time).as_secs_f64();
                last_time = now;
                
                let mut state = game_state.lock().unwrap();
                let production = state.calculate_production_per_second() * elapsed;
                
                // Add the current production to any remainder from previous ticks
                state.production_remainder += production;
                
                // Extract the whole number part
                let points_to_add = state.production_remainder.floor() as u64;
                
                if points_to_add > 0 {
                    // Update the remainder to keep only the fractional part
                    state.production_remainder -= points_to_add as f64;
                    
                    // Add the points
                    state.points += points_to_add;
                    state.lifetime_points += points_to_add;
                }
            }
        });
    }

    // Auto-save thread
    {
        let game_state = Arc::clone(&game_state);
        let running = Arc::clone(&running);
        thread::spawn(move || {
            while *running.lock().unwrap() {
                thread::sleep(Duration::from_secs(30));
                
                let state = game_state.lock().unwrap();
                let _ = state.save_game();
            }
        });
    }

    // Input + draw loop
    loop {
        // Get current state
        let state = game_state.lock().unwrap();
        
        // Draw UI based on current menu
        match state.current_menu {
            Menu::Main => draw_main_menu(&mut stdout, &state)?,
            Menu::Buildings => draw_buildings_menu(&mut stdout, &state)?,
            Menu::Upgrades => draw_upgrades_menu(&mut stdout, &state)?,
        }
        
        // Release lock while waiting for input
        drop(state);
        
        // Poll for input with 100ms timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                let mut state = game_state.lock().unwrap();
                
                match key_event.code {
                    // Global keys
                    KeyCode::Char('.') => {
                        state.click();
                    },
                    KeyCode::Char('s') => {
                        let _ = state.save_game();
                    },
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        drop(state);
                        break;
                    },
                    
                    // Menu navigation
                    KeyCode::Char('1') => {
                        state.current_menu = Menu::Main;
                        state.selected_index = 0;
                    },
                    KeyCode::Char('2') => {
                        state.current_menu = Menu::Buildings;
                        state.selected_index = 0;
                    },
                    KeyCode::Char('3') => {
                        state.current_menu = Menu::Upgrades;
                        state.selected_index = 0;
                    },
                    
                    // Selection navigation
                    KeyCode::Up => {
                        if state.selected_index > 0 {
                            state.selected_index -= 1;
                        }
                    },
                    KeyCode::Down => {
                        match state.current_menu {
                            Menu::Buildings => {
                                if state.selected_index < state.buildings.len() - 1 {
                                    state.selected_index += 1;
                                }
                            },
                            Menu::Upgrades => {
                                if state.selected_index < state.upgrades.len() - 1 {
                                    state.selected_index += 1;
                                }
                            },
                            _ => {}
                        }
                    },
                    
                    // Selection action
                    KeyCode::Enter => {
                        match state.current_menu {
                            Menu::Buildings => {
                                // First collect all building keys in sorted order by cost
                                let mut building_entries: Vec<(String, u64)> = Vec::new();
                                for (key, building) in &state.buildings {
                                    building_entries.push((key.clone(), building.current_cost()));
                                }
                                
                                // Sort by cost
                                building_entries.sort_by(|a, b| a.1.cmp(&b.1));
                                
                                // Now we can safely use the sorted keys
                                if state.selected_index < building_entries.len() {
                                    let key = &building_entries[state.selected_index].0;
                                    state.buy_building(key);
                                }
                            },
                            Menu::Upgrades => {
                                let index = state.selected_index;
                                state.buy_upgrade(index);
                            },
                            _ => {}
                        }
                    },
                    
                    _ => {}
                }
            }
        }
    }

    // Cleanup terminal
    *running.lock().unwrap() = false;
    
    // Save game before exit
    {
        let state = game_state.lock().unwrap();
        let _ = state.save_game();
    }
    
    execute!(
        stdout,
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;

    Ok(())
}

fn draw_main_menu(stdout: &mut std::io::Stdout, state: &GameState) -> IoResult<()> {
    let (_width, height) = terminal::size()?;
    let production_per_second = state.calculate_production_per_second();
    
    // Determine next influence power milestone
    let next_milestone = match state.lifetime_points {
        0..=999 => "1,000",
        1000..=9999 => "10,000",
        10000..=99999 => "100,000",
        100000..=999999 => "1,000,000",
        1000000..=9999999 => "10,000,000",
        10000000..=99999999 => "100,000,000",
        _ => "Maximum",
    };
    
    let next_power = match state.lifetime_points {
        0..=999 => 2,
        1000..=9999 => 5,
        10000..=99999 => 10,
        100000..=999999 => 25,
        1000000..=9999999 => 50,
        10000000..=99999999 => 100,
        _ => state.click_power,
    };
    
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        style::PrintStyledContent("Cthulhu's Dominion".blue().bold()),
        cursor::MoveTo(0, 2),
        style::PrintStyledContent(format!("Followers: {}", state.points).green()),
        cursor::MoveTo(0, 3),
        style::Print(format!("Total Converts: {}", state.lifetime_points)),
        cursor::MoveTo(0, 4),
        style::Print(format!("Conversion Rate: {:.1} followers/sec", production_per_second)),
        cursor::MoveTo(0, 5),
        style::Print(format!("Influence Power: {}", state.click_power)),
        cursor::MoveTo(0, 6),
        style::Print(format!("Next Power ({}) at {} total converts",
            if next_power > state.click_power { next_power.to_string() } else { "Max".to_string() },
            next_milestone)),
        cursor::MoveTo(0, 7),
        style::Print(format!("Domination Progress: {}", get_domination_status(state.lifetime_points))),
        
        cursor::MoveTo(0, 9),
        style::PrintStyledContent("Rituals:".yellow()),
        cursor::MoveTo(0, 10),
        style::Print("Press '.' to spread influence and gain followers"),
        cursor::MoveTo(0, 11),
        style::Print("Press '1' for Sanctum, '2' for Minions, '3' for Artifacts"),
        cursor::MoveTo(0, 12),
        style::Print("Press 's' to record in the Necronomicon"),
        cursor::MoveTo(0, 13),
        style::Print("Press Ctrl+C to return to mortal realm"),
        
        cursor::MoveTo(0, height - 1),
        style::PrintStyledContent("The Sanctum".cyan())
    )?;
    
    Ok(())
}

fn draw_buildings_menu(stdout: &mut std::io::Stdout, state: &GameState) -> IoResult<()> {
    let (_width, height) = terminal::size()?;
    
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        style::PrintStyledContent("Minions of Cthulhu".blue().bold()),
        cursor::MoveTo(0, 1),
        style::PrintStyledContent(format!("Followers: {}", state.points).green()),
        cursor::MoveTo(0, 2),
        style::Print(format!("Conversion Rate: {:.1} followers/sec", state.calculate_production_per_second()))
    )?;
    
    // Sort buildings by cost
    let mut buildings: Vec<(&String, &Building)> = state.buildings.iter().collect();
    buildings.sort_by(|a, b| a.1.current_cost().cmp(&b.1.current_cost()));
    
    for (i, (_key, building)) in buildings.iter().enumerate() {
        let y_pos = i as u16 + 4;
        let can_afford = state.points >= building.current_cost();
        let is_selected = i == state.selected_index;
        
        let prefix = if is_selected { "> " } else { "  " };
        let name_style = if is_selected {
            building.name.clone().yellow().bold()
        } else if can_afford {
            building.name.clone().white()
        } else {
            building.name.clone().dark_grey()
        };
        
        execute!(
            stdout,
            cursor::MoveTo(0, y_pos),
            style::Print(prefix),
            style::PrintStyledContent(name_style),
            cursor::MoveTo(20, y_pos),
            style::Print(format!("x{}", building.count)),
            cursor::MoveTo(30, y_pos),
            style::Print(format!("Souls Required: {}", building.current_cost())),
            cursor::MoveTo(50, y_pos),
            style::Print(format!("Converts: {:.1}/sec", building.total_production()))
        )?;
    }
    
    execute!(
        stdout,
        cursor::MoveTo(0, height - 2),
        style::Print("Use Up/Down to select, Enter to summon"),
        cursor::MoveTo(0, height - 1),
        style::PrintStyledContent("Minions Menu".cyan())
    )?;
    
    Ok(())
}

fn get_domination_status(lifetime_points: u64) -> String {
    match lifetime_points {
        0..=999 => "Local Cult (Town)".to_string(),
        1000..=9999 => "Regional Influence (County)".to_string(),
        10000..=99999 => "National Presence (Country)".to_string(),
        100000..=999999 => "Continental Power (Continent)".to_string(),
        1000000..=9999999 => "Global Reach (Earth)".to_string(),
        10000000..=99999999 => "Cosmic Influence (Solar System)".to_string(),
        100000000..=999999999 => "Galactic Dominion (Galaxy)".to_string(),
        _ => "Universal Awakening (Cthulhu Rises!)".to_string(),
    }
}

fn draw_upgrades_menu(stdout: &mut std::io::Stdout, state: &GameState) -> IoResult<()> {
    let (_width, height) = terminal::size()?;
    
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        style::PrintStyledContent("Eldritch Artifacts".blue().bold()),
        cursor::MoveTo(0, 1),
        style::PrintStyledContent(format!("Followers: {}", state.points).green())
    )?;
    
    for (i, upgrade) in state.upgrades.iter().enumerate() {
        // Use 3 lines per upgrade instead of 2 for better spacing
        let y_pos = i as u16 * 3 + 3;
        let can_afford = state.points >= upgrade.cost && !upgrade.purchased;
        let is_selected = i == state.selected_index;
        
        let prefix = if is_selected { "> " } else { "  " };
        let name_style = if upgrade.purchased {
            upgrade.name.clone().green()
        } else if is_selected {
            upgrade.name.clone().yellow().bold()
        } else if can_afford {
            upgrade.name.clone().white()
        } else {
            upgrade.name.clone().dark_grey()
        };
        
        let status = if upgrade.purchased { "[PURCHASED]" } else { "" };
        
        execute!(
            stdout,
            cursor::MoveTo(0, y_pos),
            style::Print(prefix),
            style::PrintStyledContent(name_style),
            cursor::MoveTo(40, y_pos),
            style::Print(format!("Souls Required: {}", upgrade.cost)),
            cursor::MoveTo(65, y_pos),
            style::Print(status),
            cursor::MoveTo(4, y_pos + 1),
            style::Print(format!("{}", upgrade.description))
        )?;
    }
    
    execute!(
        stdout,
        cursor::MoveTo(0, height - 2),
        style::Print("Use Up/Down to select, Enter to acquire"),
        cursor::MoveTo(0, height - 1),
        style::PrintStyledContent("Artifacts Menu".cyan())
    )?;
    
    Ok(())
}

