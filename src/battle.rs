use crate::{model::*, rendering::*, units::*};
use bevy::prelude::*;

pub struct BattlePlugin;

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum BattleSet {
    Input,
    Decisions,
    Movement,
    Combat,
    Consequences,
    Presentation,
}

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                BattleSet::Input,
                BattleSet::Decisions,
                BattleSet::Movement,
                BattleSet::Combat,
                BattleSet::Consequences,
                BattleSet::Presentation,
            )
                .chain()
                .run_if(in_state(AppState::Battle)),
        )
        .add_systems(OnEnter(AppState::Battle), setup_battle)
        .add_systems(OnExit(AppState::Battle), cleanup_battle)
        .add_systems(
            Update,
            (keyboard_orders, cycle_control).in_set(BattleSet::Input),
        )
        .add_systems(
            Update,
            (
                passive_income,
                enemy_controller,
                process_training,
                acquire_targets,
            )
                .chain()
                .in_set(BattleSet::Decisions),
        )
        .add_systems(
            Update,
            (
                move_miners,
                move_combat_units,
                direct_control_movement,
                move_projectiles,
            )
                .chain()
                .in_set(BattleSet::Movement),
        )
        .add_systems(
            Update,
            (automatic_attacks, controlled_attack).in_set(BattleSet::Combat),
        )
        .add_systems(
            Update,
            (apply_damage, process_deaths, check_victory)
                .chain()
                .in_set(BattleSet::Consequences),
        )
        .add_systems(
            Update,
            (
                update_health_bars,
                update_selection_markers,
                animate_walking,
            )
                .in_set(BattleSet::Presentation),
        );
    }
}

fn setup_battle(
    mut commands: Commands,
    c: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Economy {
        player_gold: 150,
        enemy_gold: 150,
        player_population: 1,
        enemy_population: 1,
        population_limit: 12,
    });
    commands.insert_resource(PlayerArmyOrder(ArmyOrder::Defend));
    commands.insert_resource(PassiveIncome(Timer::from_seconds(
        c.passive_income_seconds,
        TimerMode::Repeating,
    )));
    commands.insert_resource(EnemyController {
        spawn_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
        next_unit: UnitKind::Miner,
    });
    spawn_battlefield(&mut commands, &c);
    for team in [Team::Player, Team::Enemy] {
        spawn_statue(
            &mut commands,
            &mut meshes,
            &mut materials,
            team,
            Vec2::new(statue_x(team, &c), c.ground_y),
            c.statue_health,
        );
        spawn_gold_deposit(
            &mut commands,
            &mut meshes,
            &mut materials,
            team,
            Vec2::new(mine_x(team, &c), c.ground_y),
        );
        spawn_miner(
            &mut commands,
            &mut meshes,
            &mut materials,
            &c,
            team,
            Vec2::new(
                statue_x(team, &c) + team.direction() * 80.0,
                c.ground_y + 25.0,
            ),
        );
    }
}

fn cleanup_battle(mut commands: Commands, entities: Query<Entity, With<BattleEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Economy>();
    commands.remove_resource::<PlayerArmyOrder>();
    commands.remove_resource::<PassiveIncome>();
    commands.remove_resource::<EnemyController>();
}

fn passive_income(
    time: Res<Time>,
    c: Res<GameConfig>,
    mut timer: ResMut<PassiveIncome>,
    mut economy: ResMut<Economy>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        economy.player_gold += c.passive_income_amount;
    }
}

fn keyboard_orders(
    keys: Res<ButtonInput<KeyCode>>,
    mut order: ResMut<PlayerArmyOrder>,
    mut train: MessageWriter<TrainUnitRequest>,
) {
    if keys.just_pressed(KeyCode::KeyM) {
        train.write(TrainUnitRequest {
            team: Team::Player,
            kind: UnitKind::Miner,
        });
    }
    if keys.just_pressed(KeyCode::KeyS) {
        train.write(TrainUnitRequest {
            team: Team::Player,
            kind: UnitKind::Swordsman,
        });
    }
    if keys.just_pressed(KeyCode::KeyR) {
        train.write(TrainUnitRequest {
            team: Team::Player,
            kind: UnitKind::Archer,
        });
    }
    if keys.just_pressed(KeyCode::Digit1) {
        order.0 = ArmyOrder::Attack;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        order.0 = ArmyOrder::Defend;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        order.0 = ArmyOrder::Retreat;
    }
}

fn cycle_control(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    current: Query<Entity, With<Controlled>>,
    swords: Query<(Entity, &Team, &UnitKind), With<Unit>>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        for entity in &current {
            commands.entity(entity).remove::<Controlled>();
        }
        return;
    }
    if !keys.just_pressed(KeyCode::Tab) {
        return;
    }
    let mut choices: Vec<Entity> = swords
        .iter()
        .filter(|(_, t, k)| {
            **t == Team::Player && matches!(**k, UnitKind::Swordsman | UnitKind::Archer)
        })
        .map(|(e, _, _)| e)
        .collect();
    choices.sort_by_key(|e| e.index());
    let old = current.iter().next();
    if let Some(e) = old {
        commands.entity(e).remove::<Controlled>();
    }
    if !choices.is_empty() {
        let next = old
            .and_then(|e| choices.iter().position(|x| *x == e))
            .map(|i| (i + 1) % choices.len())
            .unwrap_or(0);
        commands
            .entity(choices[next])
            .insert(Controlled)
            .remove::<CurrentTarget>();
    }
}

fn enemy_controller(
    time: Res<Time>,
    mut ai: ResMut<EnemyController>,
    units: Query<(&Team, &UnitKind), With<Unit>>,
    mut train: MessageWriter<TrainUnitRequest>,
) {
    if !ai.spawn_timer.tick(time.delta()).just_finished() {
        return;
    }
    let miner_count = units
        .iter()
        .filter(|(t, k)| **t == Team::Enemy && **k == UnitKind::Miner)
        .count();
    let swords = units
        .iter()
        .filter(|(t, k)| **t == Team::Enemy && **k == UnitKind::Swordsman)
        .count();
    let archers = units
        .iter()
        .filter(|(t, k)| **t == Team::Enemy && **k == UnitKind::Archer)
        .count();
    ai.next_unit = if miner_count < 2 {
        UnitKind::Miner
    } else if swords >= (archers + 1) * 2 {
        UnitKind::Archer
    } else {
        UnitKind::Swordsman
    };
    train.write(TrainUnitRequest {
        team: Team::Enemy,
        kind: ai.next_unit,
    });
}

fn process_training(
    mut commands: Commands,
    mut requests: MessageReader<TrainUnitRequest>,
    mut economy: ResMut<Economy>,
    c: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for request in requests.read() {
        if !try_purchase_unit(request.team, request.kind, &mut economy, &c) {
            continue;
        }
        let pos = Vec2::new(
            statue_x(request.team, &c) + request.team.direction() * 85.0,
            c.ground_y + 25.0,
        );
        match request.kind {
            UnitKind::Miner => {
                spawn_miner(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &c,
                    request.team,
                    pos,
                );
            }
            UnitKind::Swordsman => {
                spawn_swordsman(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &c,
                    request.team,
                    pos,
                );
            }
            UnitKind::Archer => {
                spawn_archer(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &c,
                    request.team,
                    pos,
                );
            }
        }
    }
}

fn move_miners(
    time: Res<Time>,
    c: Res<GameConfig>,
    order: Res<PlayerArmyOrder>,
    mut economy: ResMut<Economy>,
    mut miners: Query<(
        &Team,
        &MoveSpeed,
        &mut Transform,
        &mut MinerState,
        &mut MiningTimer,
        &mut CarriedGold,
    )>,
) {
    for (team, speed, mut transform, mut state, mut timer, mut carried) in &mut miners {
        if *team == Team::Player && order.0 == ArmyOrder::Retreat {
            transform.translation.x = step_toward(
                transform.translation.x,
                retreat_x(*team, &c),
                speed.0,
                time.delta_secs(),
            );
            continue;
        }
        match *state {
            MinerState::GoingToMine => {
                let dest = mine_x(*team, &c);
                transform.translation.x =
                    step_toward(transform.translation.x, dest, speed.0, time.delta_secs());
                if transform.translation.x == dest {
                    timer.0.reset();
                    *state = MinerState::Mining;
                }
            }
            MinerState::Mining => {
                if timer.0.tick(time.delta()).just_finished() {
                    carried.0 = c.miner_capacity;
                    *state = MinerState::Returning;
                }
            }
            MinerState::Returning => {
                let dest = statue_x(*team, &c) + team.direction() * 70.0;
                transform.translation.x =
                    step_toward(transform.translation.x, dest, speed.0, time.delta_secs());
                if transform.translation.x == dest {
                    *economy.gold_mut(*team) += carried.0;
                    carried.0 = 0;
                    *state = MinerState::GoingToMine;
                }
            }
        }
    }
}

fn acquire_targets(
    mut commands: Commands,
    c: Res<GameConfig>,
    order: Res<PlayerArmyOrder>,
    attackers: Query<(Entity, &Team, Has<Controlled>, Option<&CurrentTarget>), With<Attack>>,
    candidates: Query<
        (Entity, &Team, &Transform, Has<Unit>, Option<&CurrentTarget>),
        Or<(With<Unit>, With<Statue>)>,
    >,
) {
    for (entity, team, controlled, target) in &attackers {
        if controlled {
            if target.is_some() {
                commands.entity(entity).remove::<CurrentTarget>();
            }
            continue;
        }
        let army_order = if *team == Team::Enemy {
            ArmyOrder::Attack
        } else {
            order.0
        };
        if army_order == ArmyOrder::Retreat {
            if target.is_some() {
                commands.entity(entity).remove::<CurrentTarget>();
            }
            continue;
        }
        let current = target.map(|target| target.0);
        let chosen = candidates
            .iter()
            .filter(|(_, other, other_transform, _, _)| {
                if *other == team {
                    return false;
                }
                army_order != ArmyOrder::Defend
                    || (other_transform.translation.x - defense_x(*team, &c)).abs()
                        <= c.defense_radius
            })
            .min_by(|a, b| {
                let priority =
                    |candidate: &(Entity, &Team, &Transform, bool, Option<&CurrentTarget>)| {
                        target_priority(
                            candidate
                                .4
                                .is_some_and(|their_target| their_target.0 == entity),
                            current == Some(candidate.0),
                        )
                    };
                let ap = priority(a);
                let bp = priority(b);
                ap.cmp(&bp).then_with(|| {
                    (a.2.translation.x - statue_x(*team, &c))
                        .abs()
                        .total_cmp(&(b.2.translation.x - statue_x(*team, &c)).abs())
                })
            })
            .map(|x| x.0);
        if chosen != current {
            if let Some(target) = chosen {
                commands.entity(entity).insert(CurrentTarget(target));
            } else if current.is_some() {
                commands.entity(entity).remove::<CurrentTarget>();
            }
        }
    }
}

fn move_combat_units(
    mut commands: Commands,
    time: Res<Time>,
    c: Res<GameConfig>,
    order: Res<PlayerArmyOrder>,
    mut queries: ParamSet<(
        Query<
            (
                Entity,
                &Team,
                &Transform,
                Option<&UnitKind>,
                Has<Controlled>,
            ),
            Or<(With<Unit>, With<Statue>)>,
        >,
        Query<
            (
                Entity,
                &Team,
                &MoveSpeed,
                &mut Transform,
                &Attack,
                &AttackMode,
                &UnitKind,
                &mut CombatUnitState,
                Option<&CurrentTarget>,
            ),
            (With<Unit>, Without<Controlled>),
        >,
    )>,
) {
    let snapshot: Vec<(Entity, Team, f32, UnitKind, bool)> = queries
        .p0()
        .iter()
        .filter(|(_, _, _, kind, _)| kind.is_some_and(|kind| *kind != UnitKind::Miner))
        .map(|(entity, team, transform, kind, controlled)| {
            (
                entity,
                *team,
                transform.translation.x,
                kind.copied().unwrap(),
                controlled,
            )
        })
        .collect();
    for (entity, team, speed, mut transform, attack, mode, kind, mut state, target) in
        &mut queries.p1()
    {
        let army_order = if *team == Team::Enemy {
            ArmyOrder::Attack
        } else {
            order.0
        };
        let combat_slot = snapshot
            .iter()
            .filter(|(other, other_team, _, _, controlled)| {
                *other_team == *team && !*controlled && other.index() < entity.index()
            })
            .count();
        let role_slot = snapshot
            .iter()
            .filter(|(other, other_team, _, other_kind, controlled)| {
                *other_team == *team
                    && *other_kind == *kind
                    && !*controlled
                    && other.index() < entity.index()
            })
            .count();
        let fallback = if army_order == ArmyOrder::Attack {
            transform.translation.x
        } else {
            formation_x(*team, army_order, *kind, role_slot, combat_slot, &c)
        };
        let destination = target.and_then(|target| {
            snapshot
                .iter()
                .find(|(entity, _, _, _, _)| *entity == target.0)
                .map(|(_, _, x, _, _)| *x)
        });
        if target.is_some() && destination.is_none() {
            commands.entity(entity).remove::<CurrentTarget>();
            continue;
        }
        if army_order == ArmyOrder::Defend
            && destination.is_some_and(|x| (x - defense_x(*team, &c)).abs() > c.defense_radius)
        {
            commands.entity(entity).remove::<CurrentTarget>();
            continue;
        }
        if let Some(dest) = destination {
            let distance = (dest - transform.translation.x).abs();
            let preferred = target_distance(*kind, attack.range);
            let too_close =
                matches!(mode, AttackMode::Projectile { .. }) && distance < preferred * 0.55;
            if too_close {
                *state = CombatUnitState::Moving;
                let escape =
                    transform.translation.x - (dest - transform.translation.x).signum() * preferred;
                transform.translation.x = step_toward(
                    transform.translation.x,
                    escape.clamp(-c.battlefield_half_width, c.battlefield_half_width),
                    speed.0,
                    time.delta_secs(),
                );
            } else if distance > preferred {
                *state = CombatUnitState::Moving;
                transform.translation.x =
                    step_toward(transform.translation.x, dest, speed.0, time.delta_secs());
            } else {
                *state = CombatUnitState::Attacking;
            }
        } else if (fallback - transform.translation.x).abs() > 2.0 {
            *state = if army_order == ArmyOrder::Retreat {
                CombatUnitState::Retreating
            } else {
                CombatUnitState::Moving
            };
            transform.translation.x = step_toward(
                transform.translation.x,
                fallback,
                speed.0,
                time.delta_secs(),
            );
        } else {
            *state = CombatUnitState::Idle;
        }
    }
}

fn direct_control_movement(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    c: Res<GameConfig>,
    mut units: Query<(&MoveSpeed, &mut Transform), With<Controlled>>,
) {
    let direction = (keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight)) as i8
        - (keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)) as i8;
    for (speed, mut transform) in &mut units {
        transform.translation.x = (transform.translation.x
            + direction as f32 * speed.0 * time.delta_secs())
        .clamp(-c.battlefield_half_width, c.battlefield_half_width);
    }
}

fn automatic_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut messages: MessageWriter<DamageMessage>,
    targets: Query<&Transform>,
    order: Res<PlayerArmyOrder>,
    mut units: Query<
        (&Team, &Transform, &CurrentTarget, &AttackMode, &mut Attack),
        Without<Controlled>,
    >,
) {
    for (team, transform, target, mode, mut attack) in &mut units {
        if *team == Team::Player && order.0 == ArmyOrder::Retreat {
            continue;
        }
        let Ok(target_transform) = targets.get(target.0) else {
            continue;
        };
        if (target_transform.translation.x - transform.translation.x).abs() <= attack.range
            && attack.cooldown.tick(time.delta()).is_finished()
        {
            perform_attack(
                &mut commands,
                &mut messages,
                *team,
                transform,
                target.0,
                target_transform.translation.x,
                *mode,
                attack.damage,
            );
            attack.cooldown.reset();
        }
    }
}

fn controlled_attack(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut messages: MessageWriter<DamageMessage>,
    mut units: Query<(&Team, &Transform, &AttackMode, &mut Attack), With<Controlled>>,
    targets: Query<(Entity, &Team, &Transform), Or<(With<Unit>, With<Statue>)>>,
) {
    for (team, transform, mode, mut attack) in &mut units {
        attack.cooldown.tick(time.delta());
        if !keys.just_pressed(KeyCode::Space) || !attack.cooldown.is_finished() {
            continue;
        }
        if let Some((entity, _, target_transform)) = targets
            .iter()
            .filter(|(_, t, p)| {
                *t != team && (p.translation.x - transform.translation.x).abs() <= attack.range
            })
            .min_by(|a, b| {
                (a.2.translation.x - transform.translation.x)
                    .abs()
                    .total_cmp(&(b.2.translation.x - transform.translation.x).abs())
            })
        {
            perform_attack(
                &mut commands,
                &mut messages,
                *team,
                transform,
                entity,
                target_transform.translation.x,
                *mode,
                attack.damage,
            );
            attack.cooldown.reset();
        }
    }
}

fn perform_attack(
    commands: &mut Commands,
    messages: &mut MessageWriter<DamageMessage>,
    team: Team,
    transform: &Transform,
    target: Entity,
    target_x: f32,
    mode: AttackMode,
    damage: f32,
) {
    match mode {
        AttackMode::Melee => {
            messages.write(DamageMessage {
                target,
                amount: damage,
            });
        }
        AttackMode::Projectile { speed } => {
            let direction = (target_x - transform.translation.x).signum();
            commands.spawn((
                BattleEntity,
                Projectile {
                    team,
                    damage,
                    velocity_x: direction * speed,
                    previous_x: transform.translation.x,
                },
                Sprite::from_color(Color::srgb(0.24, 0.13, 0.06), Vec2::new(34.0, 3.0)),
                Transform::from_xyz(
                    transform.translation.x + direction * 24.0,
                    transform.translation.y + 52.0,
                    8.0,
                ),
            ));
        }
    }
}

fn move_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    c: Res<GameConfig>,
    mut damage: MessageWriter<DamageMessage>,
    mut projectiles: Query<(Entity, &mut Projectile, &mut Transform)>,
    targets: Query<(Entity, &Team, &Transform), Or<(With<Unit>, With<Statue>)>>,
) {
    for (entity, mut projectile, mut transform) in &mut projectiles {
        projectile.previous_x = transform.translation.x;
        transform.translation.x += projectile.velocity_x * time.delta_secs();
        let hit = targets
            .iter()
            .filter(|(_, team, target)| {
                **team != projectile.team
                    && segment_crosses_point(
                        projectile.previous_x,
                        transform.translation.x,
                        target.translation.x,
                        18.0,
                    )
            })
            .min_by(|a, b| {
                (a.2.translation.x - projectile.previous_x)
                    .abs()
                    .total_cmp(&(b.2.translation.x - projectile.previous_x).abs())
            });
        if let Some((target, _, _)) = hit {
            damage.write(DamageMessage {
                target,
                amount: projectile.damage,
            });
            commands.entity(entity).despawn();
        } else if transform.translation.x.abs() > c.battlefield_half_width + 100.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn apply_damage(mut messages: MessageReader<DamageMessage>, mut health: Query<&mut Health>) {
    for message in messages.read() {
        if let Ok(mut h) = health.get_mut(message.target) {
            h.current = apply_damage_value(h.current, message.amount);
        }
    }
}

fn process_deaths(
    mut commands: Commands,
    mut economy: ResMut<Economy>,
    dead: Query<(Entity, &Team, &Health), With<Unit>>,
) {
    for (entity, team, health) in &dead {
        if health.current <= 0.0 {
            *economy.population_mut(*team) = economy.population(*team).saturating_sub(1);
            commands.entity(entity).despawn();
        }
    }
}

fn check_victory(
    mut commands: Commands,
    statues: Query<(&Team, &Health), With<Statue>>,
    mut next: ResMut<NextState<AppState>>,
) {
    for (team, health) in &statues {
        if health.current <= 0.0 {
            commands.insert_resource(if *team == Team::Enemy {
                BattleResult::Victory
            } else {
                BattleResult::Defeat
            });
            next.set(AppState::Results);
            return;
        }
    }
}
