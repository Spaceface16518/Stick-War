pub use crate::config::{GameConfig, load_game_config};
use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, Default, Eq, PartialEq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    Battle,
    Sandbox,
    Results,
}

/// States that share the battlefield lifecycle and simulation systems.
///
/// Add new playable modes here so setup, cleanup, UI, and system scheduling
/// stay centralized instead of duplicating state registrations.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum GameplayState {
    Interface,
    Active,
}

impl ComputedStates for GameplayState {
    type SourceStates = AppState;

    fn compute(state: AppState) -> Option<Self> {
        match state {
            AppState::Battle | AppState::Sandbox => Some(Self::Active),
            AppState::MainMenu | AppState::Results => Some(Self::Interface),
        }
    }
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
#[derive(Component, Debug, Clone, Copy)]
pub struct MotionEstimate {
    pub previous_position: Vec2,
    pub velocity: Vec2,
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
pub struct ArmyOrders {
    pub player: ArmyOrder,
    pub enemy: ArmyOrder,
}
impl ArmyOrders {
    pub fn get(&self, team: Team) -> ArmyOrder {
        match team {
            Team::Player => self.player,
            Team::Enemy => self.enemy,
        }
    }

    pub fn set(&mut self, team: Team, order: ArmyOrder) {
        match team {
            Team::Player => self.player = order,
            Team::Enemy => self.enemy = order,
        }
    }
}
#[derive(Resource)]
pub struct PassiveIncome(pub Timer);

#[derive(Resource)]
pub struct BattleClock {
    pub elapsed_seconds: f32,
}

#[derive(Resource)]
pub struct SandboxSettings {
    pub is_sandbox: bool,
    pub paused: bool,
    pub charge_costs: bool,
    pub training_time_enabled: bool,
}

#[derive(Debug)]
pub struct ActiveTraining {
    pub team: Team,
    pub kind: UnitKind,
    pub timer: Timer,
}

#[derive(Resource, Default)]
pub struct TrainingQueue(pub Vec<ActiveTraining>);

impl TrainingQueue {
    pub fn contains(&self, team: Team, kind: UnitKind) -> bool {
        self.0
            .iter()
            .any(|training| training.team == team && training.kind == kind)
    }

    pub fn remaining(&self, team: Team, kind: UnitKind) -> Option<f32> {
        self.0
            .iter()
            .find(|training| training.team == team && training.kind == kind)
            .map(|training| training.timer.remaining_secs())
    }
}

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
#[derive(Resource)]
pub struct CombatRandom(pub u64);

pub fn training_seconds(kind: UnitKind, c: &GameConfig) -> f32 {
    match kind {
        UnitKind::Miner => c.units.miner.training_seconds,
        UnitKind::Swordsman => c.units.swordsman.training_seconds,
        UnitKind::Archer => c.units.archer.training_seconds,
    }
}

pub fn statue_x(team: Team, c: &GameConfig) -> f32 {
    if team == Team::Player {
        c.battlefield.player.statue_x
    } else {
        c.battlefield.enemy.statue_x
    }
}
pub fn mine_x(team: Team, c: &GameConfig) -> f32 {
    if team == Team::Player {
        c.battlefield.player.mine_x
    } else {
        c.battlefield.enemy.mine_x
    }
}
pub fn defense_x(team: Team, c: &GameConfig) -> f32 {
    mine_x(team, c) + team.direction() * c.ai.defense_line_offset
}
pub fn retreat_x(team: Team, c: &GameConfig) -> f32 {
    statue_x(team, c) - team.direction() * c.formation.retreat_offset
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
        return retreat_x(team, c) - team.direction() * combat_slot as f32 * c.formation.spacing;
    }
    if order == ArmyOrder::Attack {
        return statue_x(team.opponent(), c);
    }
    let front_offset = match kind {
        UnitKind::Swordsman => {
            c.formation.swordsman_defense_offset + role_slot as f32 * c.formation.spacing
        }
        UnitKind::Archer => {
            c.formation.archer_defense_offset
                + role_slot as f32 * c.formation.spacing * c.formation.archer_spacing_factor
        }
        UnitKind::Miner => 0.0,
    };
    mine_x(team, c) + team.direction() * front_offset
}
pub fn target_distance(kind: UnitKind, attack_range: f32, c: &GameConfig) -> f32 {
    if kind == UnitKind::Archer {
        attack_range * c.units.archer.preferred_range_factor
    } else {
        attack_range
    }
}
pub fn activation_range(kind: UnitKind, c: &GameConfig) -> f32 {
    match kind {
        UnitKind::Miner => 0.0,
        UnitKind::Swordsman => c.units.swordsman.activation_range,
        UnitKind::Archer => c.units.archer.activation_range,
    }
}
pub fn projectile_can_hit(projectile_team: Team, object_team: Team) -> bool {
    projectile_team != object_team
}
pub fn ballistic_launch_velocity(
    origin: Vec2,
    target: Vec2,
    target_velocity: Vec2,
    horizontal_speed: f32,
    gravity: f32,
    max_flight_time: f32,
    variation: Vec2,
    speed_variation: f32,
    vertical_variation: f32,
) -> Vec2 {
    let base_direction = (target.x - origin.x).signum();
    let mut flight_time =
        ((target.x - origin.x).abs() / horizontal_speed.max(1.0)).clamp(0.05, max_flight_time);
    for _ in 0..2 {
        let predicted_x = target.x + target_velocity.x * flight_time;
        flight_time = ((predicted_x - origin.x).abs() / horizontal_speed.max(1.0))
            .clamp(0.05, max_flight_time);
    }
    let predicted = target + target_velocity * flight_time;
    let direction = (predicted.x - origin.x).signum();
    let direction = if direction == 0.0 {
        base_direction
    } else {
        direction
    };
    let varied_speed = horizontal_speed * (1.0 + variation.x * speed_variation);
    let velocity_x = direction * varied_speed;
    let relative_x_speed = (velocity_x - target_velocity.x).abs().max(1.0);
    let intercept_time =
        ((predicted.x - origin.x).abs() / relative_x_speed).clamp(0.05, max_flight_time);
    let intercept_y = target.y + target_velocity.y * intercept_time;
    let velocity_y = (intercept_y - origin.y + gravity * intercept_time * intercept_time * 0.5)
        / intercept_time
        + variation.y * vertical_variation;
    Vec2::new(velocity_x, velocity_y)
}

pub fn next_combat_variation(state: &mut u64) -> f32 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    let normalized = ((*state >> 40) as u32) as f32 / ((1_u32 << 24) - 1) as f32;
    normalized * 2.0 - 1.0
}
pub fn varied_attack_cooldown_seconds(
    base_seconds: f32,
    standard_milliseconds: f32,
    variation_milliseconds: f32,
    random_sample: f32,
) -> f32 {
    let delay_milliseconds =
        standard_milliseconds + random_sample.clamp(-1.0, 1.0) * variation_milliseconds;
    base_seconds + delay_milliseconds.max(0.0) / 1000.0
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
pub fn try_reserve_unit(
    team: Team,
    kind: UnitKind,
    economy: &mut Economy,
    c: &GameConfig,
    charge_cost: bool,
) -> bool {
    let cost = match kind {
        UnitKind::Miner => c.units.miner.cost,
        UnitKind::Swordsman => c.units.swordsman.cost,
        UnitKind::Archer => c.units.archer.cost,
    };
    if (charge_cost && economy.gold(team) < cost)
        || economy.population(team) >= economy.population_limit
    {
        return false;
    }
    if charge_cost {
        *economy.gold_mut(team) -= cost;
    }
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
        assert!(try_reserve_unit(
            Team::Player,
            UnitKind::Swordsman,
            &mut e,
            &GameConfig::default(),
            true,
        ));
        assert_eq!(e.player_gold, 0);
    }
    #[test]
    fn purchase_fails_without_gold() {
        let mut e = economy(49, 0);
        assert!(!try_reserve_unit(
            Team::Player,
            UnitKind::Miner,
            &mut e,
            &GameConfig::default(),
            true,
        ));
    }
    #[test]
    fn purchase_fails_at_limit() {
        let mut e = economy(500, 10);
        assert!(!try_reserve_unit(
            Team::Player,
            UnitKind::Miner,
            &mut e,
            &GameConfig::default(),
            true,
        ));
    }
    #[test]
    fn archer_purchase_uses_125_gold_and_population() {
        let c = GameConfig::default();
        let mut exact = economy(125, 2);
        assert!(try_reserve_unit(
            Team::Player,
            UnitKind::Archer,
            &mut exact,
            &c,
            true,
        ));
        assert_eq!(exact.player_gold, 0);
        assert_eq!(exact.player_population, 3);

        let mut short = economy(124, 0);
        assert!(!try_reserve_unit(
            Team::Player,
            UnitKind::Archer,
            &mut short,
            &c,
            true,
        ));
    }
    #[test]
    fn free_training_reserves_population_without_spending_gold() {
        let mut e = economy(0, 2);
        assert!(try_reserve_unit(
            Team::Enemy,
            UnitKind::Archer,
            &mut e,
            &GameConfig::default(),
            false,
        ));
        assert_eq!(e.enemy_gold, 0);
        assert_eq!(e.enemy_population, 3);
    }
    #[test]
    fn training_slots_are_unique_per_team_and_kind() {
        let mut queue = TrainingQueue::default();
        queue.0.push(ActiveTraining {
            team: Team::Player,
            kind: UnitKind::Miner,
            timer: Timer::from_seconds(3.0, TimerMode::Once),
        });
        assert!(queue.contains(Team::Player, UnitKind::Miner));
        assert!(!queue.contains(Team::Enemy, UnitKind::Miner));
        assert!(!queue.contains(Team::Player, UnitKind::Archer));
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
    fn gameplay_state_groups_all_playable_modes() {
        assert_eq!(
            GameplayState::compute(AppState::Battle),
            Some(GameplayState::Active)
        );
        assert_eq!(
            GameplayState::compute(AppState::Sandbox),
            Some(GameplayState::Active)
        );
        assert_eq!(
            GameplayState::compute(AppState::MainMenu),
            Some(GameplayState::Interface)
        );
        assert_eq!(
            GameplayState::compute(AppState::Results),
            Some(GameplayState::Interface)
        );
    }
    #[test]
    fn damage_is_clamped() {
        assert_eq!(apply_damage_value(5.0, 10.0), 0.0);
    }
    #[test]
    fn defensive_positions_are_team_sided() {
        let c = GameConfig::default();
        assert!(defense_x(Team::Player, &c) > c.battlefield.player.mine_x);
        assert!(retreat_x(Team::Player, &c) < c.battlefield.player.statue_x);
        assert!(defense_x(Team::Enemy, &c) < c.battlefield.enemy.mine_x);
    }

    #[test]
    fn miners_are_slower_than_swordsmen() {
        let c = GameConfig::default();
        assert!(c.units.miner.speed < c.units.swordsman.speed);
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
        assert!(archers.iter().all(|x| *x > c.battlefield.player.mine_x));
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
        assert_eq!(retreat_first - retreat_second, c.formation.spacing);
        assert!(retreat_first < c.battlefield.player.statue_x);
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
        assert_eq!(player_destination, c.battlefield.enemy.statue_x);
        assert_eq!(enemy_destination, c.battlefield.player.statue_x);
        assert!(player_destination > c.battlefield.player.statue_x);
        assert!(enemy_destination < c.battlefield.enemy.statue_x);
    }

    #[test]
    fn archer_prefers_standoff_distance() {
        let c = GameConfig::default();
        assert!(
            (target_distance(UnitKind::Archer, c.units.archer.weapon_range, &c) - 546.0).abs()
                < 0.001
        );
        assert_eq!(
            target_distance(UnitKind::Swordsman, c.units.swordsman.weapon_range, &c),
            c.units.swordsman.weapon_range
        );
    }

    #[test]
    fn archers_activate_before_swordsmen() {
        let c = load_game_config();
        assert!(activation_range(UnitKind::Archer, &c) > activation_range(UnitKind::Swordsman, &c));
        assert!(c.units.archer.activation_range > c.units.archer.weapon_range);
    }

    #[test]
    fn arrow_trajectory_rises_and_falls_under_gravity() {
        let c = load_game_config();
        let origin = Vec2::ZERO;
        let velocity = ballistic_launch_velocity(
            origin,
            Vec2::new(600.0, 0.0),
            Vec2::ZERO,
            c.units.archer.arrow.horizontal_speed,
            c.units.archer.arrow.gravity,
            c.units.archer.arrow.lifetime_seconds,
            Vec2::ZERO,
            c.units.archer.arrow.speed_variation,
            c.units.archer.arrow.vertical_variation,
        );
        let (near_apex, _) = projectile_step(origin, velocity, c.units.archer.arrow.gravity, 0.4);
        let (after_one_second, _) =
            projectile_step(origin, velocity, c.units.archer.arrow.gravity, 1.0);
        assert!(near_apex.y > origin.y);
        assert!(after_one_second.y < near_apex.y);
        assert_eq!(after_one_second.x, c.units.archer.arrow.horizontal_speed);
    }

    #[test]
    fn close_targets_receive_a_flatter_shot_than_distant_targets() {
        let c = load_game_config();
        let arrow = &c.units.archer.arrow;
        let close = ballistic_launch_velocity(
            Vec2::ZERO,
            Vec2::new(120.0, 0.0),
            Vec2::ZERO,
            arrow.horizontal_speed,
            arrow.gravity,
            arrow.lifetime_seconds,
            Vec2::ZERO,
            arrow.speed_variation,
            arrow.vertical_variation,
        );
        let far = ballistic_launch_velocity(
            Vec2::ZERO,
            Vec2::new(650.0, 0.0),
            Vec2::ZERO,
            arrow.horizontal_speed,
            arrow.gravity,
            arrow.lifetime_seconds,
            Vec2::ZERO,
            arrow.speed_variation,
            arrow.vertical_variation,
        );
        assert!(close.y < far.y);
    }

    #[test]
    fn ballistic_aim_leads_a_moving_target() {
        let c = load_game_config();
        let arrow = &c.units.archer.arrow;
        let stationary = ballistic_launch_velocity(
            Vec2::ZERO,
            Vec2::new(500.0, 0.0),
            Vec2::ZERO,
            arrow.horizontal_speed,
            arrow.gravity,
            arrow.lifetime_seconds,
            Vec2::ZERO,
            arrow.speed_variation,
            arrow.vertical_variation,
        );
        let retreating = ballistic_launch_velocity(
            Vec2::ZERO,
            Vec2::new(500.0, 0.0),
            Vec2::new(100.0, 0.0),
            arrow.horizontal_speed,
            arrow.gravity,
            arrow.lifetime_seconds,
            Vec2::ZERO,
            arrow.speed_variation,
            arrow.vertical_variation,
        );
        assert!(retreating.y > stationary.y);
    }

    #[test]
    fn combat_variation_is_small_deterministic_and_bounded() {
        let mut first_seed = 42;
        let mut second_seed = 42;
        for _ in 0..32 {
            let first = next_combat_variation(&mut first_seed);
            let second = next_combat_variation(&mut second_seed);
            assert_eq!(first, second);
            assert!((-1.0..=1.0).contains(&first));
        }
    }

    #[test]
    fn sword_attack_delay_stays_within_configured_millisecond_window() {
        let c = load_game_config();
        let sword = &c.units.swordsman;
        let timing = &sword.attack_delay;
        let fastest = varied_attack_cooldown_seconds(
            sword.attack_cooldown_seconds,
            timing.standard_milliseconds,
            timing.variation_milliseconds,
            -1.0,
        );
        let midpoint = varied_attack_cooldown_seconds(
            sword.attack_cooldown_seconds,
            timing.standard_milliseconds,
            timing.variation_milliseconds,
            0.0,
        );
        let slowest = varied_attack_cooldown_seconds(
            sword.attack_cooldown_seconds,
            timing.standard_milliseconds,
            timing.variation_milliseconds,
            1.0,
        );
        assert!((fastest - 0.805).abs() < 0.0001);
        assert!((midpoint - 0.810).abs() < 0.0001);
        assert!((slowest - 0.815).abs() < 0.0001);
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
        assert_eq!(c.units.archer.activation_range, 1200.0);
        assert_eq!(c.units.archer.weapon_range, 700.0);
        assert_eq!(c.units.archer.arrow.lifetime_seconds, 1.0);
        assert_eq!(c.units.archer.arrow.gravity, 650.0);
        assert_eq!(c.ai.enemy_swordsmen_per_archer, 2);
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
