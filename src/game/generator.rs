use super::*;
use bevy_spritesheet_animation::prelude::SpritesheetLibrary;
use crate::game::config::{EnemyConfig, WaveConfig};
use crate::animes::{setup_player};
use crate::animes::{AnimationIndices, AnimationTimer};
use core::f32::consts::PI;

// use bevy_rand::prelude::GlobalEntropy;
// use bevy_rand::prelude::WyRand;
// use rand::{thread_rng, Rng};

pub fn gen_user_plane(
    library: ResMut<SpritesheetLibrary>,
    atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>, 
    level: u32) -> PlayerPlaneBundle {
    let plane_y = BOTTOM_WALL + GAP_BETWEEN_PLANE_AND_WALL;
    PlayerPlaneBundle {
        plane_bundle: PlaneBundle {
            plane: Plane,
            sprite_bundle: setup_player(
                library, 
                atlas_layouts, 
                asset_server, 
                "textures/entities/player.png".to_string(), 
                9),
            hp: HP(match level {
                1 => 100,
                2 => 400,
                3 => 500,
                _ => PLAYER_PLANE_HP,
            }),
            animation_indices: AnimationIndices { first: 0, last: 8 },
            animation_timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
            on_game_screen: OnGameScreen,
            gun: GatlingGun {
                bullet_config: BulletConfig {
                    color: BULLET_COLOR,
                    diameter: BULLET_DIAMETER,
                    relative_position: BULLET_STARTING_RELATIVE_POSITION,
                    speed: match level{
                        1 => 300.0,
                        2 => 400.0,
                        _ => USER_BULLET_SPEED,
                    },
                    direction: BulletDirection::Fix(PI / 2.0),
                },
                shoot_timer: Timer::from_seconds(match level {
                    1 => 0.6,
                    2 => 1.5,
                    _ => BULLET_SHOOTING_INTERVAL
                }, TimerMode::Repeating),
            },
            laser: Laser {
                enabled: true,
                duration_timer: Some(Timer::from_seconds(match level{
                    1 => 0.0, 
                    _ => laser::LASER_DURATION
                }, TimerMode::Once)),
            },
            bullet_target: AttackTarget,
        },
        player: Player,
    }
}

pub fn gen_wave(level: u32, wave: u32) -> Vec<EnemyBundle> {
    let config = WaveConfig::get(level, wave);
    match config {
        WaveConfig::Duplicate(enemy_config, enemy_num) => {
            (0..enemy_num).map(|_| gen_enemy(&enemy_config)).collect()
        }

        WaveConfig::Detailed(enemy_configs) => enemy_configs
            .iter()
            .map(|enemy_config| gen_enemy(&enemy_config))
            .collect(),
    }
}

fn gen_enemy(
    enemy_config: &EnemyConfig,
    // mut _rng: &mut ResMut<GlobalEntropy<WyRand>>,
) -> EnemyBundle {
    // let plane_x = rng
    //     .gen_range(LEFT_WALL + GAP_BETWEEN_PLANE_AND_WALL..RIGHT_WALL - GAP_BETWEEN_PLANE_AND_WALL);
    // let plane_y = TOP_WALL - GAP_BETWEEN_PLANE_AND_WALL;

    EnemyBundle {
        plane_bundle: PlaneBundle {
            sprite_bundle: SpriteSheetBundle {
                transform: Transform {
                    translation: enemy_config.position.gen().extend(0.0),
                    scale: enemy_config.scale.extend(0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: enemy_config.color,
                    ..default()
                },
                ..default()
            },
            plane: Plane,
            gun: GatlingGun {
                bullet_config: BulletConfig {
                    color: enemy_config.color,
                    relative_position: enemy_config.bullet_relative_position.extend(0.0),
                    diameter: enemy_config.bullet_diameter,
                    speed: enemy_config.bullet_speed,
                    direction: enemy_config.bullet_direction.gen(),
                },
                shoot_timer: Timer::from_seconds(
                    enemy_config.shooting_interval,
                    TimerMode::Repeating,
                ),
            },
            laser: Laser {
                enabled: false,
                duration_timer: None,
            },
            bullet_target: AttackTarget,
            on_game_screen: OnGameScreen,
            hp: HP(enemy_config.hp),
            animation_indices: AnimationIndices { first: 0, last: 0 },
            animation_timer: AnimationTimer(Timer::from_seconds(2000000000.0, TimerMode::Repeating)),
        },
        enemy: Enemy {},
    }
}

pub fn gen_bullet(
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    gun: &GatlingGun,
    weapon_location: Vec3,
    player_plane_location: Vec3,
) -> impl Bundle {
    let bullet_position = weapon_location + gun.bullet_config.relative_position;
    return (
        MaterialMesh2dBundle {
            mesh: meshes.add(Circle::default()).into(),
            material: materials.add(gun.bullet_config.color).into(),
            transform: Transform::from_translation(bullet_position)
                .with_scale(Vec2::splat(gun.bullet_config.diameter).extend(1.)),
            ..default()
        },
        match gun.bullet_config.direction {
            BulletDirection::Fix(angle) => {
                Velocity(Vec2::from_angle(angle) * gun.bullet_config.speed)
            }
            BulletDirection::Trace => {
                let direction = (player_plane_location - bullet_position)
                    .truncate()
                    .try_normalize()
                    .unwrap_or(Vec2::from_angle(DEFAULT_ENEMY_BULLET_DIRECTION));
                Velocity(direction * gun.bullet_config.speed)
            }
        },
        Bullet,
        OnGameScreen,
    );
}
