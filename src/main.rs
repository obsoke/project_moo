use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

#[derive(Component, Debug)]
struct Exploder {
    radius: f32,
    timer: Timer,
}

// TODO: NEXT STEPS: (1) Interpolate AoE circle growing effect to coincide with timer going off
// CURRENT ISSUE: Draw two circles: One is the "outline", one is the timer/"red"
fn create_area_bomb(mut commands: Commands) {
    let parent_circle = commands
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

    let child_circle = commands
        .spawn()
        .insert(Exploder {
            radius: 150.,
            timer: Timer::from_seconds(5.0, false),
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
        .entity(parent_circle)
        .push_children(&[child_circle]);
}

fn update_exploders(time: Res<Time>, mut query: Query<(&mut Exploder, &mut Transform)>) {
    for (mut exploder, mut transform) in query.iter_mut() {
        let timer = &mut exploder.timer;
        if timer.tick(time.delta()).finished() {
            println!("WE DONE!");
            // TODO: If any Players are in here, it should damage them
        }

        // Percent is a value between 0 and 1
        // We should be able to use this to scale our "red" image from low to high
        let percent = timer.percent();
        transform.scale = Vec3::splat(percent);
        // println!(
        //     "Percent elapsed: {}, current Scale: {:?}",
        //     percent, transform.scale,
        // );
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Speed(f32);

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
        .insert(Direction::default())
        .insert_bundle(SpriteBundle {
            texture: server.load("player.png"),
            transform: Transform::from_xyz(50., 0., 0.),
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
