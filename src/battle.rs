use crate::{model::*, rendering::*, units::*};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
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
                .run_if(in_gameplay),
        )
        .add_systems(
            OnEnter(AppState::Battle),
            (setup_battle, reset_battle_camera).chain(),
        )
        .add_systems(
            OnEnter(AppState::Sandbox),
            (setup_battle, reset_battle_camera).chain(),
        )
        .add_systems(OnExit(AppState::Battle), cleanup_battle)
        .add_systems(OnExit(AppState::Sandbox), cleanup_battle)
        .add_systems(
            Update,
            (keyboard_orders, cycle_control, process_training_requests).in_set(BattleSet::Input),
        )
        .add_systems(
            Update,
            (
                tick_battle_clock,
                passive_income,
                enemy_controller,
                tick_training,
                acquire_targets,
            )
                .chain()
                .in_set(BattleSet::Decisions)
                .run_if(simulation_running),
        )
        .add_systems(
            Update,
            (
                move_miners,
                move_combat_units,
                direct_control_movement,
                update_motion_estimates,
                move_projectiles,
            )
                .chain()
                .in_set(BattleSet::Movement)
                .run_if(simulation_running),
        )
        .add_systems(
            Update,
            (automatic_attacks, controlled_attack)
                .in_set(BattleSet::Combat)
                .run_if(simulation_running),
        )
        .add_systems(
            Update,
            (apply_damage, process_deaths, check_victory)
                .chain()
                .in_set(BattleSet::Consequences)
                .run_if(simulation_running),
        )
        .add_systems(
            Update,
            (
                update_health_bars,
                update_selection_markers,
                animate_walking,
                animate_weapons,
                update_gold_sacks,
                tick_timed_effects,
            )
                .in_set(BattleSet::Presentation)
                .run_if(simulation_running),
        );
        app.add_systems(
            Update,
            update_battle_camera
                .in_set(BattleSet::Presentation)
                .run_if(in_gameplay),
        );
    }
}

fn in_gameplay(state: Res<State<AppState>>) -> bool {
    matches!(state.get(), AppState::Battle | AppState::Sandbox)
}

fn simulation_running(settings: Option<Res<SandboxSettings>>) -> bool {
    settings.is_none_or(|settings| !settings.paused)
}

fn reset_battle_camera(
    c: Res<GameConfig>,
    window: Single<&Window>,
    mut camera: Single<&mut Transform, With<BattleCamera>>,
) {
    let aspect = window.width() / window.height().max(1.0);
    camera.translation.x = clamp_camera_x(
        c.battlefield.player.statue_x,
        c.battlefield.half_width,
        camera_half_width(c.camera.view_height, aspect),
    );
    camera.translation.y = 0.0;
}

fn update_battle_camera(
    input: (
        Res<ButtonInput<KeyCode>>,
        Res<Touches>,
        Res<SandboxSettings>,
    ),
    mut mouse_wheel: MessageReader<MouseWheel>,
    time: Res<Time>,
    c: Res<GameConfig>,
    window: Single<&Window>,
    controlled: Query<&Transform, (With<Controlled>, Without<BattleCamera>)>,
    mut camera: Single<&mut Transform, With<BattleCamera>>,
) {
    let (keys, touches, settings) = input;
    let scroll = mouse_wheel
        .read()
        .map(|event| {
            let scale = match event.unit {
                MouseScrollUnit::Line => 1.0,
                MouseScrollUnit::Pixel => 1.0 / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR,
            };
            // Scroll deltas describe content movement. A camera must move in the
            // opposite direction for the battlefield to follow the host OS's
            // configured scrolling direction.
            -event.x * scale
        })
        .sum::<f32>();
    let half_view = camera_half_width(
        c.camera.view_height,
        window.width() / window.height().max(1.0),
    );
    let touch_pan = if touches.iter().count() == 1 {
        touches
            .iter()
            .map(|touch| -touch.delta().x * c.camera.view_height / window.height().max(1.0))
            .sum::<f32>()
    } else {
        0.0
    };
    let desired = if let Some(unit) = controlled.iter().next().filter(|_| !settings.paused) {
        let offset = unit.translation.x - camera.translation.x;
        if offset.abs() > c.camera.follow_dead_zone {
            unit.translation.x - offset.signum() * c.camera.follow_dead_zone
        } else {
            camera.translation.x
        }
    } else {
        let direction =
            keys.pressed(KeyCode::ArrowRight) as i8 - keys.pressed(KeyCode::ArrowLeft) as i8;
        camera.translation.x
            + direction as f32 * c.camera.pan_speed * time.delta_secs()
            + scroll * c.camera.scroll_speed
            + touch_pan
    };
    let target = clamp_camera_x(desired, c.battlefield.half_width, half_view);
    let follow_speed = if touch_pan != 0.0 {
        f32::INFINITY
    } else if controlled.is_empty() {
        c.camera.pan_speed
    } else {
        900.0
    };
    camera.translation.x = step_toward(
        camera.translation.x,
        target,
        follow_speed,
        time.delta_secs(),
    );
}

fn setup_battle(
    mut commands: Commands,
    state: Res<State<AppState>>,
    c: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(Economy {
        player_gold: c.economy.player_starting_gold,
        enemy_gold: c.economy.enemy_starting_gold,
        player_population: c.economy.starting_population,
        enemy_population: c.economy.starting_population,
        population_limit: c.economy.population_limit,
    });
    let sandbox = *state.get() == AppState::Sandbox;
    commands.insert_resource(ArmyOrders {
        player: ArmyOrder::Defend,
        enemy: ArmyOrder::Attack,
    });
    commands.insert_resource(SandboxSettings {
        is_sandbox: sandbox,
        paused: sandbox,
        charge_costs: !sandbox,
        training_time_enabled: !sandbox,
    });
    commands.insert_resource(TrainingQueue::default());
    commands.insert_resource(BattleClock {
        elapsed_seconds: 0.0,
    });
    commands.insert_resource(PassiveIncome(Timer::from_seconds(
        c.economy.passive_income_seconds,
        TimerMode::Repeating,
    )));
    commands.insert_resource(EnemyController {
        spawn_timer: Timer::from_seconds(c.ai.enemy_spawn_seconds, TimerMode::Repeating),
        next_unit: UnitKind::Miner,
    });
    commands.insert_resource(CombatRandom(c.combat.random_seed));
    spawn_battlefield(&mut commands, &mut meshes, &mut materials, &c);
    for team in [Team::Player, Team::Enemy] {
        spawn_statue(
            &mut commands,
            &mut meshes,
            &mut materials,
            team,
            Vec2::new(statue_x(team, &c), c.battlefield.ground_y),
            c.units.statue.health,
        );
        spawn_gold_deposit(
            &mut commands,
            &mut meshes,
            &mut materials,
            team,
            Vec2::new(mine_x(team, &c), c.battlefield.ground_y),
        );
        spawn_miner(
            &mut commands,
            &mut meshes,
            &mut materials,
            &c,
            team,
            Vec2::new(
                statue_x(team, &c) + team.direction() * c.units.miner.initial_spawn_offset,
                c.battlefield.ground_y + c.formation.unit_ground_offset,
            ),
        );
    }
}

fn cleanup_battle(mut commands: Commands, entities: Query<Entity, With<BattleEntity>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
    commands.remove_resource::<Economy>();
    commands.remove_resource::<ArmyOrders>();
    commands.remove_resource::<SandboxSettings>();
    commands.remove_resource::<TrainingQueue>();
    commands.remove_resource::<BattleClock>();
    commands.remove_resource::<PassiveIncome>();
    commands.remove_resource::<EnemyController>();
    commands.remove_resource::<CombatRandom>();
}

fn tick_battle_clock(time: Res<Time>, mut clock: ResMut<BattleClock>) {
    clock.elapsed_seconds += time.delta_secs();
}

fn passive_income(
    time: Res<Time>,
    c: Res<GameConfig>,
    mut timer: ResMut<PassiveIncome>,
    mut economy: ResMut<Economy>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        economy.player_gold += c.economy.passive_income_amount;
    }
}

fn keyboard_orders(
    keys: Res<ButtonInput<KeyCode>>,
    mut orders: ResMut<ArmyOrders>,
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
        orders.player = ArmyOrder::Attack;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        orders.player = ArmyOrder::Defend;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        orders.player = ArmyOrder::Retreat;
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
    state: Res<State<AppState>>,
    time: Res<Time>,
    c: Res<GameConfig>,
    mut ai: ResMut<EnemyController>,
    units: Query<(&Team, &UnitKind), With<Unit>>,
    mut train: MessageWriter<TrainUnitRequest>,
) {
    if *state.get() == AppState::Sandbox {
        return;
    }
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
    ai.next_unit = if miner_count < c.ai.enemy_desired_miners {
        UnitKind::Miner
    } else if swords >= (archers + 1) * c.ai.enemy_swordsmen_per_archer {
        UnitKind::Archer
    } else {
        UnitKind::Swordsman
    };
    train.write(TrainUnitRequest {
        team: Team::Enemy,
        kind: ai.next_unit,
    });
}

fn process_training_requests(
    mut requests: MessageReader<TrainUnitRequest>,
    mut economy: ResMut<Economy>,
    c: Res<GameConfig>,
    settings: Res<SandboxSettings>,
    mut queue: ResMut<TrainingQueue>,
) {
    for request in requests.read() {
        if queue.contains(request.team, request.kind)
            || !try_reserve_unit(
                request.team,
                request.kind,
                &mut economy,
                &c,
                settings.charge_costs,
            )
        {
            continue;
        }
        let duration = if settings.training_time_enabled {
            training_seconds(request.kind, &c)
        } else {
            0.0
        };
        queue.0.push(ActiveTraining {
            team: request.team,
            kind: request.kind,
            timer: Timer::from_seconds(duration.max(0.0), TimerMode::Once),
        });
    }
}

fn tick_training(
    mut commands: Commands,
    time: Res<Time>,
    mut queue: ResMut<TrainingQueue>,
    c: Res<GameConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut completed = Vec::new();
    for (index, training) in queue.0.iter_mut().enumerate() {
        if training.timer.tick(time.delta()).is_finished() {
            completed.push(index);
        }
    }
    for index in completed.into_iter().rev() {
        let training = queue.0.swap_remove(index);
        let pos = Vec2::new(
            statue_x(training.team, &c)
                + training.team.direction() * c.formation.trained_unit_spawn_offset,
            c.battlefield.ground_y + c.formation.unit_ground_offset,
        );
        match training.kind {
            UnitKind::Miner => {
                spawn_miner(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &c,
                    training.team,
                    pos,
                );
            }
            UnitKind::Swordsman => {
                spawn_swordsman(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &c,
                    training.team,
                    pos,
                );
            }
            UnitKind::Archer => {
                spawn_archer(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &c,
                    training.team,
                    pos,
                );
            }
        }
    }
}

fn move_miners(
    time: Res<Time>,
    c: Res<GameConfig>,
    orders: Res<ArmyOrders>,
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
        if orders.get(*team) == ArmyOrder::Retreat {
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
                    carried.0 = c.units.miner.capacity;
                    *state = MinerState::Returning;
                }
            }
            MinerState::Returning => {
                let dest = statue_x(*team, &c) + team.direction() * c.units.miner.return_offset;
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
    orders: Res<ArmyOrders>,
    attackers: Query<
        (
            Entity,
            &Team,
            &UnitKind,
            &Transform,
            Has<Controlled>,
            Option<&CurrentTarget>,
        ),
        With<Attack>,
    >,
    candidates: Query<
        (Entity, &Team, &Transform, Has<Unit>, Option<&CurrentTarget>),
        Or<(With<Unit>, With<Statue>)>,
    >,
) {
    for (entity, team, kind, attacker_transform, controlled, target) in &attackers {
        if controlled {
            if target.is_some() {
                commands.entity(entity).remove::<CurrentTarget>();
            }
            continue;
        }
        let army_order = orders.get(*team);
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
                if (other_transform.translation.x - attacker_transform.translation.x).abs()
                    > activation_range(*kind, &c)
                {
                    return false;
                }
                army_order != ArmyOrder::Defend
                    || (other_transform.translation.x - defense_x(*team, &c)).abs()
                        <= c.ai.defense_radius
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

#[derive(Clone, Copy)]
struct BattlefieldSnapshot {
    entity: Entity,
    team: Team,
    x: f32,
    kind: Option<UnitKind>,
    controlled: bool,
}

impl BattlefieldSnapshot {
    fn is_uncontrolled_combat_unit(self) -> bool {
        !self.controlled && matches!(self.kind, Some(UnitKind::Swordsman | UnitKind::Archer))
    }
}

fn move_combat_units(
    mut commands: Commands,
    time: Res<Time>,
    c: Res<GameConfig>,
    orders: Res<ArmyOrders>,
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
    let snapshot: Vec<BattlefieldSnapshot> = queries
        .p0()
        .iter()
        .map(
            |(entity, team, transform, kind, controlled)| BattlefieldSnapshot {
                entity,
                team: *team,
                x: transform.translation.x,
                kind: kind.copied(),
                controlled,
            },
        )
        .collect();
    for (entity, team, speed, mut transform, attack, mode, kind, mut state, target) in
        &mut queries.p1()
    {
        let army_order = orders.get(*team);
        let combat_slot = snapshot
            .iter()
            .filter(|other| {
                other.team == *team
                    && other.is_uncontrolled_combat_unit()
                    && other.entity.index() < entity.index()
            })
            .count();
        let role_slot = snapshot
            .iter()
            .filter(|other| {
                other.team == *team
                    && other.kind == Some(*kind)
                    && other.is_uncontrolled_combat_unit()
                    && other.entity.index() < entity.index()
            })
            .count();
        let order_destination = formation_x(*team, army_order, *kind, role_slot, combat_slot, &c);
        let destination = target.and_then(|target| {
            snapshot
                .iter()
                .find(|candidate| candidate.entity == target.0)
                .map(|candidate| candidate.x)
        });
        if target.is_some() && destination.is_none() {
            commands.entity(entity).remove::<CurrentTarget>();
            continue;
        }
        if army_order == ArmyOrder::Defend
            && destination.is_some_and(|x| (x - defense_x(*team, &c)).abs() > c.ai.defense_radius)
        {
            commands.entity(entity).remove::<CurrentTarget>();
            continue;
        }
        if let Some(dest) = destination {
            let distance = (dest - transform.translation.x).abs();
            let preferred = target_distance(*kind, attack.range, &c);
            let too_close = matches!(mode, AttackMode::Projectile)
                && distance < preferred * c.units.archer.backpedal_range_factor;
            if too_close {
                *state = CombatUnitState::Moving;
                let escape =
                    transform.translation.x - (dest - transform.translation.x).signum() * preferred;
                transform.translation.x = step_toward(
                    transform.translation.x,
                    escape.clamp(-c.battlefield.half_width, c.battlefield.half_width),
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
        } else if (order_destination - transform.translation.x).abs()
            > c.formation.arrival_tolerance
        {
            *state = if army_order == ArmyOrder::Retreat {
                CombatUnitState::Retreating
            } else {
                CombatUnitState::Moving
            };
            transform.translation.x = step_toward(
                transform.translation.x,
                order_destination,
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
        .clamp(-c.battlefield.half_width, c.battlefield.half_width);
    }
}

fn update_motion_estimates(
    time: Res<Time>,
    c: Res<GameConfig>,
    mut units: Query<(&Transform, &mut MotionEstimate), With<Unit>>,
) {
    let dt = time.delta_secs();
    if dt <= f32::EPSILON {
        return;
    }
    for (transform, mut motion) in &mut units {
        let position = transform.translation.xy();
        let measured_velocity = (position - motion.previous_position) / dt;
        motion.velocity = motion.velocity.lerp(
            measured_velocity,
            c.units.archer.arrow.velocity_smoothing.clamp(0.0, 1.0),
        );
        motion.previous_position = position;
    }
}

fn automatic_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut messages: MessageWriter<DamageMessage>,
    targets: Query<(&Transform, Option<&MotionEstimate>, Has<Statue>)>,
    orders: Res<ArmyOrders>,
    c: Res<GameConfig>,
    mut random: ResMut<CombatRandom>,
    mut units: Query<
        (
            Entity,
            &Team,
            &Transform,
            &CurrentTarget,
            &AttackMode,
            &UnitKind,
            &mut Attack,
        ),
        Without<Controlled>,
    >,
) {
    for (owner, team, transform, target, mode, kind, mut attack) in &mut units {
        if orders.get(*team) == ArmyOrder::Retreat {
            continue;
        }
        let Ok((target_transform, target_motion, target_is_statue)) = targets.get(target.0) else {
            continue;
        };
        if (target_transform.translation.x - transform.translation.x).abs() <= attack.range
            && attack.cooldown.tick(time.delta()).is_finished()
        {
            perform_attack(
                &mut commands,
                &mut messages,
                &c,
                &mut random,
                owner,
                *team,
                transform,
                target.0,
                target_transform,
                target_motion.map_or(Vec2::ZERO, |motion| motion.velocity),
                target_is_statue,
                *mode,
                attack.damage,
            );
            reset_attack_cooldown(*kind, &mut attack, &c, &mut random);
        }
    }
}

fn controlled_attack(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    c: Res<GameConfig>,
    mut random: ResMut<CombatRandom>,
    mut messages: MessageWriter<DamageMessage>,
    mut units: Query<
        (
            Entity,
            &Team,
            &Transform,
            &AttackMode,
            &UnitKind,
            &mut Attack,
        ),
        With<Controlled>,
    >,
    targets: Query<
        (
            Entity,
            &Team,
            &Transform,
            Option<&MotionEstimate>,
            Has<Statue>,
        ),
        (Or<(With<Unit>, With<Statue>)>, Without<Projectile>),
    >,
) {
    for (owner, team, transform, mode, kind, mut attack) in &mut units {
        attack.cooldown.tick(time.delta());
        if !keys.just_pressed(KeyCode::Space) || !attack.cooldown.is_finished() {
            continue;
        }
        if let Some((entity, _, target_transform, target_motion, target_is_statue)) = targets
            .iter()
            .filter(|(_, t, p, _, _)| {
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
                &c,
                &mut random,
                owner,
                *team,
                transform,
                entity,
                target_transform,
                target_motion.map_or(Vec2::ZERO, |motion| motion.velocity),
                target_is_statue,
                *mode,
                attack.damage,
            );
            reset_attack_cooldown(*kind, &mut attack, &c, &mut random);
        }
    }
}

fn reset_attack_cooldown(
    kind: UnitKind,
    attack: &mut Attack,
    c: &GameConfig,
    random: &mut CombatRandom,
) {
    let duration_seconds = match kind {
        UnitKind::Swordsman => {
            let timing = &c.units.swordsman.attack_delay;
            varied_attack_cooldown_seconds(
                c.units.swordsman.attack_cooldown_seconds,
                timing.standard_milliseconds,
                timing.variation_milliseconds,
                next_combat_variation(&mut random.0),
            )
        }
        UnitKind::Archer => c.units.archer.attack_cooldown_seconds,
        UnitKind::Miner => return,
    };
    attack
        .cooldown
        .set_duration(std::time::Duration::from_secs_f32(duration_seconds));
    attack.cooldown.reset();
}

fn perform_attack(
    commands: &mut Commands,
    messages: &mut MessageWriter<DamageMessage>,
    c: &GameConfig,
    random: &mut CombatRandom,
    owner: Entity,
    team: Team,
    transform: &Transform,
    target: Entity,
    target_transform: &Transform,
    target_velocity: Vec2,
    target_is_statue: bool,
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
        AttackMode::Projectile => {
            let direction = (target_transform.translation.x - transform.translation.x).signum();
            let origin = Vec2::new(
                transform.translation.x + direction * c.units.archer.arrow.spawn_forward,
                transform.translation.y + c.units.archer.arrow.spawn_height,
            );
            let aim_height = if target_is_statue {
                c.units.archer.arrow.statue_target_height
            } else {
                c.units.archer.arrow.unit_target_height
            };
            let target_position = target_transform.translation.xy() + Vec2::Y * aim_height;
            let variation = Vec2::new(
                next_combat_variation(&mut random.0),
                next_combat_variation(&mut random.0),
            );
            let velocity = ballistic_launch_velocity(
                origin,
                target_position,
                target_velocity,
                c.units.archer.arrow.horizontal_speed,
                c.units.archer.arrow.gravity,
                c.units.archer.arrow.lifetime_seconds,
                variation,
                c.units.archer.arrow.speed_variation,
                c.units.archer.arrow.vertical_variation,
            );
            commands.spawn((
                BattleEntity,
                Projectile {
                    owner,
                    team,
                    damage,
                    velocity,
                    lifetime_remaining: c.units.archer.arrow.lifetime_seconds,
                },
                Sprite::from_color(Color::srgb(0.24, 0.13, 0.06), Vec2::new(34.0, 3.0)),
                Transform::from_xyz(origin.x, origin.y, 8.0)
                    .with_rotation(Quat::from_rotation_z(velocity.y.atan2(velocity.x))),
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
    objects: Query<
        (
            Entity,
            &Team,
            &Transform,
            Has<Unit>,
            Has<Statue>,
            Option<&Health>,
        ),
        (
            Or<(With<Unit>, With<Statue>, With<GoldDeposit>)>,
            Without<Projectile>,
        ),
    >,
) {
    for (entity, mut projectile, mut transform) in &mut projectiles {
        let from = transform.translation.xy();
        let (to, next_velocity) = projectile_step(
            from,
            projectile.velocity,
            c.units.archer.arrow.gravity,
            time.delta_secs(),
        );
        projectile.velocity = next_velocity;
        projectile.lifetime_remaining -= time.delta_secs();
        transform.translation.x = to.x;
        transform.translation.y = to.y;
        transform.rotation = Quat::from_rotation_z(next_velocity.y.atan2(next_velocity.x));

        let hit = objects
            .iter()
            .filter(|(target, team, _, _, _, _)| {
                *target != projectile.owner && projectile_can_hit(projectile.team, **team)
            })
            .filter(|(_, _, target, unit, statue, _)| {
                let (half_width, bottom, top) = if *statue {
                    (
                        c.collision.statue.half_width,
                        c.collision.statue.bottom,
                        c.collision.statue.top,
                    )
                } else if *unit {
                    (
                        c.collision.character.half_width,
                        c.collision.character.bottom,
                        c.collision.character.top,
                    )
                } else {
                    (
                        c.collision.deposit.half_width,
                        c.collision.deposit.bottom,
                        c.collision.deposit.top,
                    )
                };
                let center = target.translation.xy();
                let radius = Vec2::splat(c.units.archer.arrow.collision_radius);
                segment_hits_aabb(
                    from,
                    to,
                    center + Vec2::new(-half_width, bottom) - radius,
                    center + Vec2::new(half_width, top) + radius,
                )
            })
            .min_by(|a, b| {
                a.2.translation
                    .xy()
                    .distance_squared(from)
                    .total_cmp(&b.2.translation.xy().distance_squared(from))
            });
        let hit_ground = from.y >= c.battlefield.ground_y && to.y <= c.battlefield.ground_y;
        if let Some((target, _, _, _, _, health)) = hit {
            if health.is_some() {
                damage.write(DamageMessage {
                    target,
                    amount: projectile.damage,
                });
            }
            commands.spawn((
                BattleEntity,
                TimedEffect(Timer::from_seconds(0.18, TimerMode::Once)),
                Sprite::from_color(Color::srgba(1.0, 0.78, 0.2, 0.9), Vec2::new(28.0, 28.0)),
                Transform::from_xyz(transform.translation.x, transform.translation.y, 9.0)
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            ));
            commands.entity(entity).despawn();
        } else if hit_ground || projectile.lifetime_remaining <= 0.0 {
            commands.spawn((
                BattleEntity,
                TimedEffect(Timer::from_seconds(0.12, TimerMode::Once)),
                Sprite::from_color(Color::srgba(0.65, 0.48, 0.25, 0.8), Vec2::new(20.0, 8.0)),
                Transform::from_xyz(to.x, to.y.max(c.battlefield.ground_y), 9.0),
            ));
            commands.entity(entity).despawn();
        }
    }
}

fn apply_damage(
    mut commands: Commands,
    mut messages: MessageReader<DamageMessage>,
    mut health: Query<(&mut Health, &Transform)>,
) {
    for message in messages.read() {
        if let Ok((mut h, transform)) = health.get_mut(message.target) {
            h.current = apply_damage_value(h.current, message.amount);
            commands.spawn((
                BattleEntity,
                TimedEffect(Timer::from_seconds(0.12, TimerMode::Once)),
                Sprite::from_color(Color::srgba(1.0, 0.18, 0.12, 0.65), Vec2::new(48.0, 92.0)),
                Transform::from_xyz(
                    transform.translation.x,
                    transform.translation.y + 30.0,
                    10.0,
                ),
            ));
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
    state: Res<State<AppState>>,
    mut commands: Commands,
    statues: Query<(&Team, &Health), With<Statue>>,
    mut next: ResMut<NextState<AppState>>,
) {
    if *state.get() == AppState::Sandbox {
        return;
    }
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
