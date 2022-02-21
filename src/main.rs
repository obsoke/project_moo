use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Component, Debug)]
struct Exploder {
    radius: f32,
    timer: Timer,
    parent_id: Entity,
}

fn create_area_bomb(mut commands: Commands) {
    let exploder_outline = commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shapes::Circle::default(),
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: Color::Rgba {
                        red: 0.,
                        green: 0.,
                        blue: 0.,
                        alpha: 0.,
                    },
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode::new(Color::BLACK, 0.01),
            },
            Transform {
                translation: Vec3::new(150., 150., 0.),
                scale: Vec3::splat(150.),
                ..Default::default()
            },
        ))
        .id();

    let exploder_fill = commands
        .spawn()
        .insert(Exploder {
            radius: 150.,
            timer: Timer::from_seconds(5.0, false),
            parent_id: exploder_outline,
        })
        .insert_bundle(GeometryBuilder::build_as(
            &shapes::Circle::default(),
            DrawMode::Outlined {
                fill_mode: FillMode {
                    color: Color::Rgba {
                        red: 0.7,
                        green: 0.,
                        blue: 0.,
                        alpha: 1.,
                    },
                    options: FillOptions::default(),
                },
                outline_mode: StrokeMode::new(Color::CRIMSON, 0.01),
            },
            Transform::default(),
        ))
        .id();

    commands
        .entity(exploder_outline)
        .push_children(&[exploder_fill]);
}

fn update_exploders(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Exploder, &mut Transform, &GlobalTransform)>,
    mut player_query: Query<(&GlobalTransform, &mut Health), (With<Player>, Without<Exploder>)>,
) {
    let (player_transform, mut health) = player_query.single_mut();

    for (mut exploder, mut transform, global_transform) in query.iter_mut() {
        let exploder = &mut *exploder;
        let timer = &mut exploder.timer;
        let parent = exploder.parent_id;
        let player_translation = player_transform.translation;
        let translation = global_transform.translation;
        // println!(
        //     "Coords: Player Left X {:?}, Exploder Left  {:?}",
        //     player_translation.x, translation.x,
        // );
        if timer.tick(time.delta()).finished() {
            // Destroy the Exploder entity via parent
            commands.entity(parent).despawn_recursive();

            // Check for intersection of player & exploder
            // TODO: Cleanup
            if player_translation.x > (translation.x - 150.)
                && player_translation.x < (translation.x + 150.)
                && player_translation.y < (translation.y + 150.)
                && player_translation.y > (translation.y - 150.)
            {
                health.0 -= 100.;
            }
        }

        // Percent is a value between 0 and 1
        // We should be able to use this to scale our "red" image from low to high
        let percent = timer.percent();
        transform.scale = Vec3::splat(percent);
    }

    println!("Player health is {}", health.0);
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Health(f32);

#[derive(Component, Default, Debug)]
struct Direction(Vec3);

fn create_player(mut commands: Commands, server: Res<AssetServer>) {
    // Create default 2d camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // Create player entity
    commands
        .spawn()
        .insert(Player)
        .insert(Speed(300.))
        .insert(Health(100.))
        .insert(Direction::default())
        .insert_bundle(SpriteBundle {
            texture: server.load("player.png"),
            transform: Transform::from_xyz(50., 0., 55.),
            ..Default::default()
        });
}

fn move_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Speed, &mut Transform, &mut Direction), With<Player>>,
) {
    let (speed, mut transform, mut last_direction) = query.single_mut();
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::Left) {
        direction.x -= 1.0;
    } else if keyboard_input.pressed(KeyCode::Right) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Up) {
        direction.y += 1.0;
    } else if keyboard_input.pressed(KeyCode::Down) {
        direction.y -= 1.0;
    }

    transform.translation += direction.normalize_or_zero() * speed.0 * time.delta_seconds();

    if direction != Vec3::ZERO {
        last_direction.0 = direction;
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_startup_system(create_player)
        // .add_system(create_area_bomb)
        .add_startup_system(create_area_bomb)
        .add_system(update_exploders)
        .add_system(move_player)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
