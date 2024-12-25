use bevy::{
    app::{App, FixedUpdate, Plugin, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    audio::{AudioPlayer, AudioSource, PlaybackSettings},
    color::Color,
    input::{keyboard::KeyboardInput, ButtonInput},
    math::{
        bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
        vec2, Quat, Vec2, Vec3,
    },
    prelude::{
        BuildChildren, Camera2d, Circle, Commands, Component, Deref, DerefMut, Entity, Event,
        EventReader, EventWriter, IntoSystemConfigs, KeyCode, Mesh, Mesh2d, Query, Res, ResMut,
        Resource, Single, Text, TextUiWriter, Transform, With,
    },
    sprite::{ColorMaterial, MeshMaterial2d, Sprite},
    text::{TextColor, TextFont, TextSpan},
    time::Time,
    ui::{Node, Val},
    DefaultPlugins,
};

// ###############
// paddle
// ################
const PADDLE_SIZE: Vec2 = Vec2::new(120.0, 20.0);
const PADDLE_SPEED: f32 = 500.0;
const PADDLE_PADDING: f32 = 10.0;
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;

// ################
// ball
// ################
const BALL_STARTING_POSITION: Vec3 = Vec3::new(0.0, -50.0, 1.0);
const BALL_DIAMETER: f32 = 30.;
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = Vec2::new(0.5, -0.5);

// ###############
// scoreboard
// ################
const SCOREBOARD_FONT_SIZE: f32 = 33.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

// ################
// brick
// ################
const BRICK_SIZE: Vec2 = Vec2::new(100., 30.);
const GAP_BETWEEN_BRICKS: f32 = 5.0;

// ###############
// wall
// ###############
// These values are lower bounds, as the number of bricks is computed
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

// ################
// colors
// ################

const BACKGROUND_COLOR: Color = Color::srgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::srgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::srgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::srgb(1.0, 0.5, 0.5);

// ################
// Wall
// ################

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

// ################
// structs
// ################

#[derive(Resource, Deref, DerefMut)]
struct Score(u32);

#[derive(Resource)]
struct ClearColor(pub Color);

#[derive(Event, Default)]
struct CollisionEvent;

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Brick;

#[derive(Resource)]
struct CollisionSound(pub Handle<AudioSource>);

#[derive(Component, Default)]
struct Collider;

#[derive(Component)]
#[require(Sprite, Transform, Collider)]
struct Wall;

impl Wall {
    fn new(location: WallLocation) -> (Wall, Sprite, Transform) {
        (
            Wall,
            Sprite::from_color(WALL_COLOR, Vec2::ONE),
            Transform {
                translation: location.position().extend(0.0),
                scale: location.size().extend(1.0),
                ..Default::default()
            },
        )
    }
}

enum WallLocation {
    Left,
    Top,
    Right,
    Bottom,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;

        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, WALL_THICKNESS + arena_height)
            }
            WallLocation::Top | WallLocation::Bottom => {
                Vec2::new(WALL_THICKNESS + arena_width, WALL_THICKNESS)
            }
        }
    }
}

#[derive(Component)]
struct Velocity(pub Vec2);

impl Velocity {
    pub fn x(&self) -> f32 {
        self.0.x
    }

    pub fn y(&self) -> f32 {
        self.0.y
    }
}

#[derive(Component)]
struct ScoreboardUi;

// ################
// business logic
// ################

pub fn run() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Score(0))
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                apply_velocity,
                move_paddle,
                check_for_collisions,
                play_collision_sound,
            )
                .chain(),
        )
        .add_systems(Update, update_scoreboard)
        .run();
}

fn setup(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // camera
    cmds.spawn(Camera2d);

    // sound
    let ball_collision_sound = asset_server.load("audio/stop.flac");
    cmds.insert_resource(CollisionSound(ball_collision_sound));

    // paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;
    cmds.spawn((
        Sprite::from_color(PADDLE_COLOR, Vec2::ONE),
        Transform {
            translation: Vec3::new(0.0, paddle_y, 0.0),
            scale: PADDLE_SIZE.extend(1.0),
            ..Default::default()
        },
        Paddle,
        Collider,
    ));

    // ball
    cmds.spawn((
        Mesh2d(meshes.add(Circle::default())),
        MeshMaterial2d(materials.add(BALL_COLOR)),
        Transform::from_translation(BALL_STARTING_POSITION)
            .with_scale(Vec2::splat(BALL_DIAMETER).extend(1.)),
        Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED),
    ));

    // scoreboard
    cmds.spawn((
        Text::new("Score: "),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..Default::default()
        },
        TextColor(TEXT_COLOR),
        ScoreboardUi,
        Node {
            position_type: bevy::ui::PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..Default::default()
        },
    ))
    .with_child((
        TextSpan::default(),
        TextFont {
            font_size: SCOREBOARD_FONT_SIZE,
            ..Default::default()
        },
        TextColor(SCORE_COLOR),
    ));

    // wall
    cmds.spawn(Wall::new(WallLocation::Left));
    cmds.spawn(Wall::new(WallLocation::Top));
    cmds.spawn(Wall::new(WallLocation::Right));
    cmds.spawn(Wall::new(WallLocation::Bottom));

    // bricks
    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2.0 * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_FLOOR;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
    // Space taken up by the bricks
    - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
    // Space taken up by the gaps
    - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            cmds.spawn((
                Sprite::from_color(BRICK_COLOR, Vec2::ONE),
                Transform {
                    translation: brick_position.extend(0.0),
                    scale: BRICK_SIZE.extend(1.0),
                    ..Default::default()
                },
                Brick,
                Collider,
            ));
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_transform: Single<&mut Transform, With<Paddle>>,
    time: Res<Time>,
) {
    let mut direction = 0.0;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        direction -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        direction += 1.0;
    }

    let new_paddle_pos =
        paddle_transform.translation.x + direction * PADDLE_SPEED * time.delta_secs();

    // update the paddle position
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_pos.clamp(left_bound, right_bound);
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity), (With<Transform>, With<Velocity>)>,
    time: Res<Time>,
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x() * time.delta_secs();
        transform.translation.y += velocity.y() * time.delta_secs();
    }
}

fn update_scoreboard(
    score: Res<Score>,
    score_root: Single<Entity, (With<ScoreboardUi>, With<Text>)>,
    mut writer: TextUiWriter,
) {
    *writer.text(*score_root, 1) = (*score).to_string()
}

fn check_for_collisions(
    mut commands: Commands,
    mut score: ResMut<Score>,
    ball_query: Single<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_event: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.into_inner();

    for (collider_entity, collider_transform, maybe_brick) in &collider_query {
        let collision = ball_collision(
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER / 2.0),
            Aabb2d::new(
                collider_transform.translation.truncate(),
                collider_transform.scale.truncate() / 2.0,
            ),
        );

        if let Some(collision) = collision {
            collision_event.send_default();

            if maybe_brick.is_some() {
                commands.entity(collider_entity).despawn();
                **score += 1;
            }

            //reflect the ball's velocity when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // reflect only if the velocity is in the opposite direction of the collision
            // This prevents the ball from getting stuck inside the ball
            match collision {
                Collision::Left => reflect_x = ball_velocity.x() > 0.0,
                Collision::Top => reflect_y = ball_velocity.y() < 0.0,
                Collision::Right => reflect_x = ball_velocity.x() < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y() > 0.0,
            }

            if reflect_x {
                ball_velocity.0.x = -ball_velocity.x();
            }

            if reflect_y {
                ball_velocity.0.y = -ball_velocity.y();
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Collision {
    Left,
    Top,
    Right,
    Bottom,
}

fn play_collision_sound(
    mut cmds: Commands,
    mut collision_event: EventReader<CollisionEvent>,
    sound: Res<CollisionSound>,
) {
    if !collision_event.is_empty() {
        // This prevents events staying active on the next frame.
        collision_event.clear();
        cmds.spawn((AudioPlayer(sound.0.clone()), PlaybackSettings::DESPAWN));
    }
}

// Returns `Some` if `ball` collides with `bounding_box`.
// The returned `Collision` is the side of `bounding_box` that `ball` hit.
fn ball_collision(ball: BoundingCircle, bounding_box: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&bounding_box) {
        return None;
    }

    let closest = bounding_box.closest_point(ball.center());

    let offset = ball.center() - closest;

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0.0 {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0.0 {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

#[derive(Clone)]
pub struct Breackout;

impl Plugin for Breackout {
    fn build(&self, app: &mut bevy::prelude::App) {
        todo!()
    }
}
