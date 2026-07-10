use bevy::prelude::*;

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
}

#[derive(Component, Debug, Clone, Copy, Eq, PartialEq)]
pub enum UnitKind {
    Miner,
    Swordsman,
}
#[derive(Component)]
pub struct Unit;
#[derive(Component)]
pub struct Statue;
#[derive(Component)]
pub struct GoldDeposit;
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
pub enum SwordsmanState {
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

#[derive(Resource)]
pub struct GameConfig {
    pub battlefield_half_width: f32,
    pub ground_y: f32,
    pub player_statue_x: f32,
    pub enemy_statue_x: f32,
    pub player_mine_x: f32,
    pub enemy_mine_x: f32,
    pub miner_cost: u32,
    pub miner_health: f32,
    pub miner_speed: f32,
    pub miner_capacity: u32,
    pub mining_duration: f32,
    pub swordsman_cost: u32,
    pub swordsman_health: f32,
    pub swordsman_speed: f32,
    pub swordsman_damage: f32,
    pub swordsman_range: f32,
    pub swordsman_attack_seconds: f32,
    pub statue_health: f32,
    pub defense_radius: f32,
    pub retreat_offset: f32,
}
impl Default for GameConfig {
    fn default() -> Self {
        Self {
            battlefield_half_width: 1600.0,
            ground_y: -180.0,
            player_statue_x: -1400.0,
            enemy_statue_x: 1400.0,
            player_mine_x: -1080.0,
            enemy_mine_x: 1080.0,
            miner_cost: 50,
            miner_health: 40.0,
            miner_speed: 90.0,
            miner_capacity: 25,
            mining_duration: 1.5,
            swordsman_cost: 100,
            swordsman_health: 100.0,
            swordsman_speed: 120.0,
            swordsman_damage: 20.0,
            swordsman_range: 55.0,
            swordsman_attack_seconds: 0.8,
            statue_health: 500.0,
            defense_radius: 280.0,
            retreat_offset: 100.0,
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
    statue_x(team, c) + team.direction() * 140.0
}
pub fn retreat_x(team: Team, c: &GameConfig) -> f32 {
    statue_x(team, c) - team.direction() * c.retreat_offset
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
pub fn try_purchase_unit(
    team: Team,
    kind: UnitKind,
    economy: &mut Economy,
    c: &GameConfig,
) -> bool {
    let cost = if kind == UnitKind::Miner {
        c.miner_cost
    } else {
        c.swordsman_cost
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
    fn movement_stops_at_destination() {
        assert_eq!(step_toward(0.0, 5.0, 10.0, 1.0), 5.0);
        assert_eq!(step_toward(0.0, 50.0, 10.0, 1.0), 10.0);
    }
    #[test]
    fn directions_are_correct() {
        assert_eq!(Team::Player.direction(), 1.0);
        assert_eq!(Team::Enemy.direction(), -1.0);
    }
    #[test]
    fn damage_is_clamped() {
        assert_eq!(apply_damage_value(5.0, 10.0), 0.0);
    }
    #[test]
    fn defensive_positions_are_team_sided() {
        let c = GameConfig::default();
        assert!(defense_x(Team::Player, &c) > c.player_statue_x);
        assert!(retreat_x(Team::Player, &c) < c.player_statue_x);
        assert!(defense_x(Team::Enemy, &c) < c.enemy_statue_x);
    }
}
