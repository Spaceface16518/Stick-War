use bevy::prelude::*;
use serde::Deserialize;

#[derive(Resource, Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub battlefield: BattlefieldConfig,
    pub economy: EconomyConfig,
    pub units: UnitConfigs,
    pub ai: AiConfig,
    pub formation: FormationConfig,
    pub collision: CollisionConfig,
    pub camera: CameraConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BattlefieldConfig {
    pub half_width: f32,
    pub ground_y: f32,
    pub player: BasePositions,
    pub enemy: BasePositions,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BasePositions {
    pub statue_x: f32,
    pub mine_x: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EconomyConfig {
    pub player_starting_gold: u32,
    pub enemy_starting_gold: u32,
    pub starting_population: u32,
    pub population_limit: u32,
    pub passive_income_amount: u32,
    pub passive_income_seconds: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnitConfigs {
    pub miner: MinerConfig,
    pub swordsman: CombatUnitConfig,
    pub archer: ArcherConfig,
    pub statue: StatueConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MinerConfig {
    pub cost: u32,
    pub health: f32,
    pub speed: f32,
    pub capacity: u32,
    pub mining_seconds: f32,
    pub initial_spawn_offset: f32,
    pub return_offset: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CombatUnitConfig {
    pub cost: u32,
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub weapon_range: f32,
    pub attack_cooldown_seconds: f32,
    pub activation_range: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArcherConfig {
    pub cost: u32,
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub weapon_range: f32,
    pub attack_cooldown_seconds: f32,
    pub activation_range: f32,
    pub preferred_range_factor: f32,
    pub backpedal_range_factor: f32,
    pub arrow: ArrowConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArrowConfig {
    pub horizontal_speed: f32,
    pub gravity: f32,
    pub lifetime_seconds: f32,
    pub collision_radius: f32,
    pub spawn_forward: f32,
    pub spawn_height: f32,
    pub unit_target_height: f32,
    pub statue_target_height: f32,
    pub speed_variation: f32,
    pub vertical_variation: f32,
    pub velocity_smoothing: f32,
    pub random_seed: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatueConfig {
    pub health: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AiConfig {
    pub defense_radius: f32,
    pub defense_line_offset: f32,
    pub enemy_spawn_seconds: f32,
    pub enemy_desired_miners: usize,
    pub enemy_swordsmen_per_archer: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormationConfig {
    pub retreat_offset: f32,
    pub spacing: f32,
    pub swordsman_defense_offset: f32,
    pub archer_defense_offset: f32,
    pub archer_spacing_factor: f32,
    pub trained_unit_spawn_offset: f32,
    pub unit_ground_offset: f32,
    pub arrival_tolerance: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CollisionConfig {
    pub character: CollisionBounds,
    pub statue: CollisionBounds,
    pub deposit: CollisionBounds,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CollisionBounds {
    pub half_width: f32,
    pub bottom: f32,
    pub top: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CameraConfig {
    pub view_height: f32,
    pub pan_speed: f32,
    pub follow_dead_zone: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        load_game_config()
    }
}

pub fn load_game_config() -> GameConfig {
    let source = std::fs::read_to_string("config/game_config.ron")
        .unwrap_or_else(|_| include_str!("../config/game_config.ron").to_owned());
    ron::from_str(&source).expect("config/game_config.ron must contain a valid GameConfig")
}
