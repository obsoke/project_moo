use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

#[derive(Component, Debug)]
struct Exploder {
    radius: f32,
    timer: Timer,
    parent_id: Entity,
}

fn create_area_bomb(mut commands: Commands, window: Res<Windows>) {
    let primary_window = window.get_primary().unwrap();
    let edge_w = primary_window.width() / 2.;
    let edge_h = primary_window.height() / 2.;

    let mut rng = thread_rng();
    let x_pos = rng.gen_range(-edge_w..=edge_w);
    let y_pos = rng.gen_range(-edge_h..=edge_h);
    let radius = rng.gen_range(50.0..=150.);
    let timer: f32 = rng.gen_range(2..=5) as f32;

    let exploder_outline = commands
        .spawn()
        .insert_bundle(GeometryBuilder::build_as(
            &shapes::Circle::default(),
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::NONE),
                outline_mode: StrokeMode::new(Color::BLACK, 0.01),
            },
            Transform {
                translation: Vec3::new(x_pos, y_pos, 0.),
                scale: Vec3::splat(radius),
                ..Default::default()
            },
        ))
        .id();

    let exploder_fill = commands
        .spawn()
        .insert(Exploder {
            radius,
            timer: Timer::from_seconds(timer, false),
            parent_id: exploder_outline,
        })
        .insert_bundle(GeometryBuilder::build_as(
            &shapes::Circle::default(),
            DrawMode::Outlined {
                fill_mode: FillMode::color(Color::CRIMSON),
                outline_mode: StrokeMode::new(Color::CRIMSON, 0.01),
            },
            Transform {
                scale: Vec3::splat(0.01), // So small, it can't be seen at first - hacky?
                ..Default::default()
            },
        ))
        .id();

    commands
        .entity(exploder_outline)
        .push_children(&[exploder_fill]);
}

// TODO (Next next time!): Render a health bar over Player's head
// TODO (Next next time!): When health is 0, game ends
// TODO (Next steps): More complicated exploders (effects, different patterns)
fn update_exploders(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Exploder, &mut Transform, &GlobalTransform)>,
    mut player_query: Query<(&GlobalTransform, &mut Health), (With<Player>, Without<Exploder>)>,
) {
    let (player_transform, mut health) = player_query.single_mut();

    for (mut exploder, mut transform, global_transform) in query.iter_mut() {
        let exploder = &mut *exploder; // Mut borrow technique to get borrow checker to stop complaining

        let timer = &mut exploder.timer;
        if timer.tick(time.delta()).finished() {
            // Destroy the Exploder entity via parent
            commands.entity(exploder.parent_id).despawn_recursive();

            // Check for intersection of player & exploder
            let radius = exploder.radius;
            let player_translation = player_transform.translation;
            let translation = global_transform.translation;

            if player_translation.x > (translation.x - radius)
                && player_translation.x < (translation.x + radius)
                && player_translation.y < (translation.y + radius)
                && player_translation.y > (translation.y - radius)
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
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(bevy::core::FixedTimestep::step(2.0))
                .with_system(create_area_bomb),
        )
        .add_system(update_exploders)
        .add_system(move_player)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}
