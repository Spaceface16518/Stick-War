use bevy::prelude::*;
use serde::Deserialize;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Battle,
    Results,
}

#[derive(Resource, Debug, Clone, Copy)]
pub enum BattleResult {
    Victory,
    Defeat,
}

#[derive(Component)]
pub struct MainMenuEntity;
#[derive(Component)]
pub struct BattleEntity;
#[derive(Component)]
pub struct ResultsEntity;

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum Team {
    Player,
    Enemy,
}
impl Team {
    pub fn direction(self) -> f32 {
        if self == Self::Player { 1.0 } else { -1.0 }
    }

    pub fn opponent(self) -> Self {
        if self == Self::Player {
            Self::Enemy
        } else {
            Self::Player
        }
    }
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnitKind {
    Miner,
    Swordsman,
    Archer,
}
#[derive(Component)]
pub struct Unit;
#[derive(Component)]
pub struct Statue;
#[derive(Component)]
pub struct GoldDeposit;
#[derive(Component)]
pub struct BattleCamera;
#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub maximum: f32,
}
#[derive(Component)]
pub struct MoveSpeed(pub f32);
#[derive(Component)]
pub struct Attack {
    pub damage: f32,
    pub range: f32,
    pub cooldown: Timer,
}
#[derive(Component, Debug, Clone, Copy)]
pub enum AttackMode {
    Melee,
    Projectile,
}
#[derive(Component, Debug, Clone, Copy)]
pub struct Projectile {
    pub owner: Entity,
    pub team: Team,
    pub damage: f32,
    pub velocity: Vec2,
    pub lifetime_remaining: f32,
}
#[derive(Component)]
pub struct CurrentTarget(pub Entity);
#[derive(Component)]
pub struct Controlled;
#[derive(Component)]
pub struct HealthBarFill {
    pub owner: Entity,
}
#[derive(Component)]
pub struct SelectionMarker;
#[derive(Component)]
pub struct WeaponVisual {
    pub kind: UnitKind,
}
#[derive(Component)]
pub struct GoldSackVisual;
#[derive(Component)]
pub struct TimedEffect(pub Timer);
#[derive(Component)]
pub struct Limb {
    pub kind: LimbKind,
    pub rest_angle: f32,
}
#[derive(Debug, Clone, Copy)]
pub enum LimbKind {
    LeftArm,
    RightArm,
    LeftLeg,
    RightLeg,
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum MinerState {
    GoingToMine,
    Mining,
    Returning,
}
#[derive(Component)]
pub struct MiningTimer(pub Timer);
#[derive(Component)]
pub struct CarriedGold(pub u32);
#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum CombatUnitState {
    Idle,
    Moving,
    Attacking,
    Retreating,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArmyOrder {
    Attack,
    Defend,
    Retreat,
}
#[derive(Resource)]
pub struct PlayerArmyOrder(pub ArmyOrder);
#[derive(Resource)]
pub struct PassiveIncome(pub Timer);

#[derive(Resource)]
pub struct Economy {
    pub player_gold: u32,
    pub enemy_gold: u32,
    pub player_population: u32,
    pub enemy_population: u32,
    pub population_limit: u32,
}
impl Economy {
    pub fn gold(&self, team: Team) -> u32 {
        if team == Team::Player {
            self.player_gold
        } else {
            self.enemy_gold
        }
    }
    pub fn gold_mut(&mut self, team: Team) -> &mut u32 {
        if team == Team::Player {
            &mut self.player_gold
        } else {
            &mut self.enemy_gold
        }
    }
    pub fn population(&self, team: Team) -> u32 {
        if team == Team::Player {
            self.player_population
        } else {
            self.enemy_population
        }
    }
    pub fn population_mut(&mut self, team: Team) -> &mut u32 {
        if team == Team::Player {
            &mut self.player_population
        } else {
            &mut self.enemy_population
        }
    }
}

#[derive(Resource, Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub battlefield_half_width: f32,
    pub ground_y: f32,
    pub player_statue_x: f32,
    pub enemy_statue_x: f32,
    pub player_mine_x: f32,
    pub enemy_mine_x: f32,
    pub player_starting_gold: u32,
    pub enemy_starting_gold: u32,
    pub starting_population: u32,
    pub population_limit: u32,
    pub miner_cost: u32,
    pub miner_health: f32,
    pub miner_speed: f32,
    pub miner_capacity: u32,
    pub mining_duration: f32,
    pub passive_income_amount: u32,
    pub passive_income_seconds: f32,
    pub swordsman_cost: u32,
    pub swordsman_health: f32,
    pub swordsman_speed: f32,
    pub swordsman_damage: f32,
    pub swordsman_range: f32,
    pub swordsman_attack_seconds: f32,
    pub swordsman_activation_range: f32,
    pub archer_cost: u32,
    pub archer_health: f32,
    pub archer_speed: f32,
    pub archer_damage: f32,
    pub archer_range: f32,
    pub archer_attack_seconds: f32,
    pub archer_activation_range: f32,
    pub archer_preferred_range_factor: f32,
    pub archer_backpedal_range_factor: f32,
    pub arrow_horizontal_speed: f32,
    pub arrow_vertical_speed: f32,
    pub arrow_gravity: f32,
    pub arrow_lifetime_seconds: f32,
    pub arrow_collision_radius: f32,
    pub arrow_spawn_forward: f32,
    pub arrow_spawn_height: f32,
    pub character_collision_half_width: f32,
    pub character_collision_bottom: f32,
    pub character_collision_height: f32,
    pub statue_collision_half_width: f32,
    pub statue_collision_bottom: f32,
    pub statue_collision_height: f32,
    pub deposit_collision_half_width: f32,
    pub deposit_collision_bottom: f32,
    pub deposit_collision_height: f32,
    pub statue_health: f32,
    pub defense_radius: f32,
    pub retreat_offset: f32,
    pub formation_spacing: f32,
    pub swordsman_defense_offset: f32,
    pub archer_defense_offset: f32,
    pub archer_formation_spacing_factor: f32,
    pub unit_spawn_offset: f32,
    pub initial_miner_spawn_offset: f32,
    pub unit_ground_offset: f32,
    pub miner_return_offset: f32,
    pub formation_arrival_tolerance: f32,
    pub defense_line_offset: f32,
    pub enemy_spawn_seconds: f32,
    pub enemy_desired_miners: usize,
    pub enemy_swordsmen_per_archer: usize,
    pub camera_view_height: f32,
    pub camera_pan_speed: f32,
    pub camera_dead_zone: f32,
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

#[derive(Message)]
pub struct TrainUnitRequest {
    pub team: Team,
    pub kind: UnitKind,
}
#[derive(Message)]
pub struct DamageMessage {
    pub target: Entity,
    pub amount: f32,
}
#[derive(Resource)]
pub struct EnemyController {
    pub spawn_timer: Timer,
    pub next_unit: UnitKind,
}

pub fn statue_x(team: Team, c: &GameConfig) -> f32 {
    if team == Team::Player {
        c.player_statue_x
    } else {
        c.enemy_statue_x
    }
}
pub fn mine_x(team: Team, c: &GameConfig) -> f32 {
    if team == Team::Player {
        c.player_mine_x
    } else {
        c.enemy_mine_x
    }
}
pub fn defense_x(team: Team, c: &GameConfig) -> f32 {
    mine_x(team, c) + team.direction() * c.defense_line_offset
}
pub fn retreat_x(team: Team, c: &GameConfig) -> f32 {
    statue_x(team, c) - team.direction() * c.retreat_offset
}
pub fn formation_x(
    team: Team,
    order: ArmyOrder,
    kind: UnitKind,
    role_slot: usize,
    combat_slot: usize,
    c: &GameConfig,
) -> f32 {
    if order == ArmyOrder::Retreat {
        return retreat_x(team, c) - team.direction() * combat_slot as f32 * c.formation_spacing;
    }
    if order == ArmyOrder::Attack {
        return statue_x(team.opponent(), c);
    }
    let front_offset = match kind {
        UnitKind::Swordsman => c.swordsman_defense_offset + role_slot as f32 * c.formation_spacing,
        UnitKind::Archer => {
            c.archer_defense_offset
                + role_slot as f32 * c.formation_spacing * c.archer_formation_spacing_factor
        }
        UnitKind::Miner => 0.0,
    };
    mine_x(team, c) + team.direction() * front_offset
}
pub fn target_distance(kind: UnitKind, attack_range: f32, c: &GameConfig) -> f32 {
    if kind == UnitKind::Archer {
        attack_range * c.archer_preferred_range_factor
    } else {
        attack_range
    }
}
pub fn activation_range(kind: UnitKind, c: &GameConfig) -> f32 {
    match kind {
        UnitKind::Miner => 0.0,
        UnitKind::Swordsman => c.swordsman_activation_range,
        UnitKind::Archer => c.archer_activation_range,
    }
}
pub fn projectile_can_hit(projectile_team: Team, object_team: Team) -> bool {
    projectile_team != object_team
}
pub fn projectile_step(position: Vec2, velocity: Vec2, gravity: f32, dt: f32) -> (Vec2, Vec2) {
    let next = Vec2::new(
        position.x + velocity.x * dt,
        position.y + velocity.y * dt - gravity * dt * dt * 0.5,
    );
    (next, Vec2::new(velocity.x, velocity.y - gravity * dt))
}
pub fn segment_hits_aabb(from: Vec2, to: Vec2, min: Vec2, max: Vec2) -> bool {
    let delta = to - from;
    let mut near: f32 = 0.0;
    let mut far: f32 = 1.0;
    for (origin, direction, lower, upper) in [
        (from.x, delta.x, min.x, max.x),
        (from.y, delta.y, min.y, max.y),
    ] {
        if direction.abs() < f32::EPSILON {
            if origin < lower || origin > upper {
                return false;
            }
        } else {
            let first = (lower - origin) / direction;
            let second = (upper - origin) / direction;
            near = near.max(first.min(second));
            far = far.min(first.max(second));
            if near > far {
                return false;
            }
        }
    }
    true
}
pub fn camera_half_width(view_height: f32, aspect_ratio: f32) -> f32 {
    view_height * aspect_ratio.max(0.01) * 0.5
}
pub fn clamp_camera_x(desired: f32, battlefield_half_width: f32, half_view_width: f32) -> f32 {
    let limit = (battlefield_half_width - half_view_width).max(0.0);
    desired.clamp(-limit, limit)
}
pub fn step_toward(current: f32, destination: f32, speed: f32, dt: f32) -> f32 {
    let difference = destination - current;
    let step = speed * dt;
    if difference.abs() <= step {
        destination
    } else {
        current + difference.signum() * step
    }
}
pub fn apply_damage_value(current: f32, damage: f32) -> f32 {
    (current - damage).max(0.0)
}
pub fn target_priority(is_attacking_us: bool, is_current_target: bool) -> u8 {
    if is_attacking_us {
        0
    } else if is_current_target {
        1
    } else {
        2
    }
}
pub fn try_purchase_unit(
    team: Team,
    kind: UnitKind,
    economy: &mut Economy,
    c: &GameConfig,
) -> bool {
    let cost = match kind {
        UnitKind::Miner => c.miner_cost,
        UnitKind::Swordsman => c.swordsman_cost,
        UnitKind::Archer => c.archer_cost,
    };
    if economy.gold(team) < cost || economy.population(team) >= economy.population_limit {
        return false;
    }
    *economy.gold_mut(team) -= cost;
    *economy.population_mut(team) += 1;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    fn economy(gold: u32, population: u32) -> Economy {
        Economy {
            player_gold: gold,
            enemy_gold: gold,
            player_population: population,
            enemy_population: population,
            population_limit: 10,
        }
    }
    #[test]
    fn purchase_succeeds() {
        let mut e = economy(100, 0);
        assert!(try_purchase_unit(
            Team::Player,
            UnitKind::Swordsman,
            &mut e,
            &GameConfig::default()
        ));
        assert_eq!(e.player_gold, 0);
    }
    #[test]
    fn purchase_fails_without_gold() {
        let mut e = economy(49, 0);
        assert!(!try_purchase_unit(
            Team::Player,
            UnitKind::Miner,
            &mut e,
            &GameConfig::default()
        ));
    }
    #[test]
    fn purchase_fails_at_limit() {
        let mut e = economy(500, 10);
        assert!(!try_purchase_unit(
            Team::Player,
            UnitKind::Miner,
            &mut e,
            &GameConfig::default()
        ));
    }
    #[test]
    fn archer_purchase_uses_125_gold_and_population() {
        let c = GameConfig::default();
        let mut exact = economy(125, 2);
        assert!(try_purchase_unit(
            Team::Player,
            UnitKind::Archer,
            &mut exact,
            &c
        ));
        assert_eq!(exact.player_gold, 0);
        assert_eq!(exact.player_population, 3);

        let mut short = economy(124, 0);
        assert!(!try_purchase_unit(
            Team::Player,
            UnitKind::Archer,
            &mut short,
            &c
        ));
    }
    #[test]
    fn movement_stops_at_destination() {
        assert_eq!(step_toward(0.0, 5.0, 10.0, 1.0), 5.0);
        assert_eq!(step_toward(0.0, 50.0, 10.0, 1.0), 10.0);
    }
    #[test]
    fn directions_are_correct() {
        assert_eq!(Team::Player.direction(), 1.0);
        assert_eq!(Team::Enemy.direction(), -1.0);
        assert_eq!(Team::Player.opponent(), Team::Enemy);
        assert_eq!(Team::Enemy.opponent(), Team::Player);
    }
    #[test]
    fn damage_is_clamped() {
        assert_eq!(apply_damage_value(5.0, 10.0), 0.0);
    }
    #[test]
    fn defensive_positions_are_team_sided() {
        let c = GameConfig::default();
        assert!(defense_x(Team::Player, &c) > c.player_mine_x);
        assert!(retreat_x(Team::Player, &c) < c.player_statue_x);
        assert!(defense_x(Team::Enemy, &c) < c.enemy_mine_x);
    }

    #[test]
    fn miners_are_slower_than_swordsmen() {
        let c = GameConfig::default();
        assert!(c.miner_speed < c.swordsman_speed);
    }

    #[test]
    fn attackers_take_target_priority() {
        assert!(target_priority(true, false) < target_priority(false, true));
        assert!(target_priority(false, true) < target_priority(false, false));
    }

    #[test]
    fn mixed_formation_orders_frontline_and_does_not_overlap() {
        let c = GameConfig::default();
        let swords = [0, 1].map(|slot| {
            formation_x(
                Team::Player,
                ArmyOrder::Defend,
                UnitKind::Swordsman,
                slot,
                slot,
                &c,
            )
        });
        let archers = [0, 1, 2].map(|slot| {
            formation_x(
                Team::Player,
                ArmyOrder::Defend,
                UnitKind::Archer,
                slot,
                slot + 2,
                &c,
            )
        });
        assert!(archers.iter().all(|x| *x > c.player_mine_x));
        assert!(swords.iter().all(|sword| *sword > archers[2]));
        let mut all = [swords[0], swords[1], archers[0], archers[1], archers[2]];
        all.sort_by(f32::total_cmp);
        assert!(all.windows(2).all(|pair| pair[1] - pair[0] >= 35.0));

        let retreat_first = formation_x(
            Team::Player,
            ArmyOrder::Retreat,
            UnitKind::Swordsman,
            0,
            0,
            &c,
        );
        let retreat_second =
            formation_x(Team::Player, ArmyOrder::Retreat, UnitKind::Archer, 0, 1, &c);
        assert_eq!(retreat_first - retreat_second, c.formation_spacing);
        assert!(retreat_first < c.player_statue_x);
    }

    #[test]
    fn attack_order_advances_each_team_toward_the_opposing_statue() {
        let c = GameConfig::default();
        let player_destination = formation_x(
            Team::Player,
            ArmyOrder::Attack,
            UnitKind::Swordsman,
            0,
            0,
            &c,
        );
        let enemy_destination =
            formation_x(Team::Enemy, ArmyOrder::Attack, UnitKind::Archer, 0, 0, &c);
        assert_eq!(player_destination, c.enemy_statue_x);
        assert_eq!(enemy_destination, c.player_statue_x);
        assert!(player_destination > c.player_statue_x);
        assert!(enemy_destination < c.enemy_statue_x);
    }

    #[test]
    fn archer_prefers_standoff_distance() {
        let c = GameConfig::default();
        assert!((target_distance(UnitKind::Archer, c.archer_range, &c) - 249.6).abs() < 0.001);
        assert_eq!(
            target_distance(UnitKind::Swordsman, c.swordsman_range, &c),
            c.swordsman_range
        );
    }

    #[test]
    fn archers_activate_before_swordsmen() {
        let c = load_game_config();
        assert!(activation_range(UnitKind::Archer, &c) > activation_range(UnitKind::Swordsman, &c));
        assert!(c.archer_activation_range > c.archer_range);
    }

    #[test]
    fn arrow_trajectory_rises_and_falls_under_gravity() {
        let c = load_game_config();
        let origin = Vec2::ZERO;
        let velocity = Vec2::new(c.arrow_horizontal_speed, c.arrow_vertical_speed);
        let (near_apex, _) = projectile_step(origin, velocity, c.arrow_gravity, 0.4);
        let (after_one_second, _) = projectile_step(origin, velocity, c.arrow_gravity, 1.0);
        assert!(near_apex.y > origin.y);
        assert!(after_one_second.y < near_apex.y);
        assert_eq!(after_one_second.x, c.arrow_horizontal_speed);
    }

    #[test]
    fn swept_arrow_collides_with_character_or_ground_bounds() {
        assert!(segment_hits_aabb(
            Vec2::new(0.0, 20.0),
            Vec2::new(30.0, -10.0),
            Vec2::new(12.0, -5.0),
            Vec2::new(18.0, 25.0),
        ));
        assert!(!segment_hits_aabb(
            Vec2::new(0.0, 40.0),
            Vec2::new(30.0, 30.0),
            Vec2::new(12.0, -5.0),
            Vec2::new(18.0, 25.0),
        ));
    }

    #[test]
    fn arrows_pass_through_friendlies_and_hit_enemies() {
        assert!(!projectile_can_hit(Team::Player, Team::Player));
        assert!(!projectile_can_hit(Team::Enemy, Team::Enemy));
        assert!(projectile_can_hit(Team::Player, Team::Enemy));
        assert!(projectile_can_hit(Team::Enemy, Team::Player));
    }

    #[test]
    fn ron_file_controls_character_and_arrow_behavior() {
        let c = load_game_config();
        assert_eq!(c.archer_activation_range, 720.0);
        assert_eq!(c.arrow_lifetime_seconds, 1.0);
        assert_eq!(c.arrow_gravity, 650.0);
        assert_eq!(c.enemy_swordsmen_per_archer, 2);
    }

    #[test]
    fn camera_clamps_to_battlefield_edges() {
        assert_eq!(clamp_camera_x(-5000.0, 1600.0, 640.0), -960.0);
        assert_eq!(clamp_camera_x(5000.0, 1600.0, 640.0), 960.0);
        assert_eq!(clamp_camera_x(120.0, 1600.0, 640.0), 120.0);
    }

    #[test]
    fn camera_bounds_follow_multiple_aspect_ratios() {
        let height = 720.0;
        let wide = camera_half_width(height, 16.0 / 9.0);
        let narrow = camera_half_width(height, 9.0 / 16.0);
        assert_eq!(wide, 640.0);
        assert_eq!(narrow, 202.5);
        assert!(clamp_camera_x(5000.0, 1600.0, narrow) > clamp_camera_x(5000.0, 1600.0, wide));
        assert_eq!(clamp_camera_x(5000.0, 1600.0, 2000.0), 0.0);
    }
}
