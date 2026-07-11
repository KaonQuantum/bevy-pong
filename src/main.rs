use bevy::{
    math::{
        bounding::{Aabb2d, BoundingVolume, IntersectsVolume},
        ops::sqrt,
    },
    prelude::*,
};

const BALL_SIZE: f32 = 5.;
const BALL_SHAPE: Circle = Circle::new(BALL_SIZE);
const BALL_COLOR: Color = Color::srgb_u8(0xf7, 0x25, 0x85);
const BALL_SPEED: f32 = 2.1;
const SQUARE_BALL_SPEED: f32 = BALL_SPEED * BALL_SPEED;
const FRACTIONAL_BALL_SPEED: f32 = BALL_SPEED / 50.;

const PADDLE_SHAPE: Rectangle = Rectangle::new(20., 50.);
const PADDLE_COLOR: Color = Color::srgb_u8(0x73, 0xee, 0xdc);
const PADDLE_SPEED: f32 = 4.2;

const GUTTER_COLOR: Color = Color::srgb_u8(0x43, 0x61, 0xee);
const GUTTER_HEIGHT: f32 = 20.;

const WIN_MESSAGES: &[&str] = &[
    "You Win",
    "Easy Dubs",
    "Aura Farmed",
    "A+",
    "Slaughtered",
    "Veni, Vidi, Vici",
    "Easy Mode",
    "You Ate That",
];
const NUM_WIN_MESSAGES: usize = WIN_MESSAGES.len();

const LOSS_MESSAGES: &[&str] = &[
    "Game Over",
    "Wops Moment",
    "Major L",
    "Rigged!",
    "Skill Issue",
    "Forgot Keybinds",
    "It Glitched!",
    "Couldn't Count to 10",
];
const NUM_LOSS_MESSAGES: usize = LOSS_MESSAGES.len();

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    Menu,
    Playing,
    End,
}

#[derive(Component)]
struct MenuScreen;

#[derive(Component, Default)]
struct PlayScreen;

#[derive(Component, Default)]
#[require(Transform)]
struct Position(Vec2);

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component, Default)]
struct Collider(Rectangle);

impl Collider {
    fn half_size(&self) -> Vec2 {
        self.0.half_size
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Component)]
#[require(
    Position,
    Velocity = Velocity(Vec2::new(-BALL_SPEED, BALL_SPEED)),
    Collider = Collider(Rectangle::new(BALL_SIZE * 2., BALL_SIZE * 2.)),
    PlayScreen,
)]
struct Ball;

#[derive(Component)]
#[require(
    Position,
    Velocity,
    Collider = Collider(PADDLE_SHAPE),
    PlayScreen,
)]
struct Paddle;

#[derive(Component)]
#[require(Position, Collider, PlayScreen)]
struct Gutter;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Ai;

#[derive(Resource)]
struct AiMovementTimer(Timer);

#[derive(Resource)]
struct Score {
    player: u8,
    ai: u8,
}

#[derive(Component)]
struct PlayerScore;

#[derive(Component)]
struct AiScore;

#[derive(EntityEvent)]
struct Scored {
    #[event_target]
    scorer: Entity,
}

#[derive(Resource)]
struct Rng(fastrand::Rng);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb_u8(0x22, 0x1e, 0x22)))
        .insert_resource(Score { player: 0, ai: 0 })
        .insert_resource(Rng(fastrand::Rng::new()))
        .insert_resource(AiMovementTimer(Timer::from_seconds(
            0.1,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, spawn_camera)
        .add_systems(OnEnter(GameState::Menu), spawn_menu)
        .add_systems(
            Update,
            handle_start_button.run_if(in_state(GameState::Menu).or_else(in_state(GameState::End))),
        )
        .add_systems(OnExit(GameState::Menu), despawn_menu)
        .add_systems(
            OnEnter(GameState::Playing),
            (spawn_ball, spawn_paddles, spawn_gutters, spawn_scoreboard),
        )
        .add_systems(
            FixedUpdate,
            (
                handle_player_input.before(move_paddles),
                move_ball.before(project_positions),
                move_paddles.before(project_positions),
                project_positions,
                handle_collisions.after(move_ball),
                constrain_paddle_position.after(move_paddles).after(move_ai),
                detect_goal.after(move_ball),
                update_scoreboard,
                move_ai.before(project_positions),
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_play)
        .add_systems(OnEnter(GameState::End), spawn_end)
        .add_systems(OnExit(GameState::End), despawn_menu)
        .add_observer(reset_ball)
        .add_observer(update_score)
        .run();
}

fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/PixelifySans-Regular.ttf");

    let container = Node {
        width: percent(100.),
        height: percent(100.),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    let button = (
        Button,
        Node {
            width: px(150.),
            height: px(65.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.)),
            ..default()
        },
        BorderColor::all(Color::BLACK),
        BackgroundColor(Color::srgb_u8(0x72, 0x09, 0xb7)),
    );

    let label = (
        Text::new("Start"),
        TextFont {
            font: FontSource::Handle(font),
            font_size: FontSize::Px(40.),
            ..default()
        },
        TextColor(Color::BLACK),
    );

    commands.spawn((MenuScreen, container, children![(button, children![label])]));
}

fn spawn_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(BALL_SHAPE);
    let material = materials.add(BALL_COLOR);
    commands.spawn((Ball, Mesh2d(mesh), MeshMaterial2d(material)));
}

fn spawn_paddles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let mesh = meshes.add(PADDLE_SHAPE);
    let material = materials.add(PADDLE_COLOR);
    let half_window_size = window.resolution.size() / 2.;
    let padding = 20.;

    let player_position = Vec2::new(-half_window_size.x + padding, 0.);

    commands.spawn((
        Player,
        Paddle,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Position(player_position),
    ));

    let ai_position = Vec2::new(half_window_size.x - padding, 0.);

    commands.spawn((
        Ai,
        Paddle,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Position(ai_position),
    ));
}

fn spawn_gutters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    let material = materials.add(GUTTER_COLOR);
    let padding = 20.;

    let gutter_shape = Rectangle::new(window.resolution.width(), GUTTER_HEIGHT);
    let mesh = meshes.add(gutter_shape);

    let top_gutter_position = Vec2::new(0., window.resolution.height() / 2. - padding);

    commands.spawn((
        Gutter,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Position(top_gutter_position),
        Collider(gutter_shape),
    ));

    let bottom_gutter_position = Vec2::new(0., -window.resolution.height() / 2. + padding);

    commands.spawn((
        Gutter,
        Mesh2d(mesh.clone()),
        MeshMaterial2d(material.clone()),
        Position(bottom_gutter_position),
        Collider(gutter_shape),
    ));
}

fn spawn_scoreboard(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/PixelifySans-Regular.ttf");

    let container = Node {
        width: percent(100.),
        height: percent(100.),
        justify_content: JustifyContent::Center,
        ..default()
    };

    let header = Node {
        width: px(200.),
        height: px(100.),
        ..default()
    };

    let player_score = (
        PlayerScore,
        Text::new("0"),
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: FontSize::Px(72.),
            ..default()
        },
        TextColor(Color::srgb_u8(0x72, 0x09, 0xb7)),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: px(20.),
            left: px(25.),
            ..default()
        },
    );

    let ai_score = (
        AiScore,
        Text::new("0"),
        TextFont {
            font: FontSource::Handle(font),
            font_size: FontSize::Px(72.),
            ..default()
        },
        TextColor(Color::srgb_u8(0x72, 0x09, 0xb7)),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            top: px(20.),
            right: px(25.),
            ..default()
        },
    );

    commands.spawn((
        container,
        PlayScreen,
        children![(header, children![player_score, ai_score])],
    ));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_end(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut score: ResMut<Score>,
    mut rng: ResMut<Rng>,
) {
    let font = asset_server.load("fonts/PixelifySans-Regular.ttf");

    let container = Node {
        height: percent(100.),
        width: percent(100.),
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        row_gap: Val::Px(40.),
        ..default()
    };

    let button = (
        Button,
        Node {
            width: px(200.),
            height: px(65.),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.)),
            ..default()
        },
        BorderColor::all(Color::BLACK),
        BackgroundColor(Color::srgb_u8(0x72, 0x09, 0xb7)),
    );

    let label = (
        Text::new("Restart"),
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: FontSize::Px(40.),
            ..default()
        },
        TextColor(Color::BLACK),
    );

    let (text, color) = if score.player == 10 {
        (
            WIN_MESSAGES[rng.0.usize(0..NUM_WIN_MESSAGES)],
            Color::srgb_u8(0x73, 0xee, 0xdc),
        )
    } else {
        (
            LOSS_MESSAGES[rng.0.usize(0..NUM_LOSS_MESSAGES)],
            Color::srgb_u8(0xf7, 0x25, 0x85),
        )
    };

    score.player = 0;
    score.ai = 0;

    let message = (
        Text::new(text),
        TextFont {
            font: FontSource::Handle(font),
            font_size: FontSize::Px(100.),
            ..default()
        },
        TextColor(color),
    );

    commands.spawn((
        MenuScreen,
        container,
        children![message, (button, children![label])],
    ));
}

fn handle_start_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in interaction_query {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Playing);
        }
    }
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

fn despawn_play(mut commands: Commands, query: Query<Entity, With<PlayScreen>>) {
    for entity in query {
        commands.entity(entity).despawn();
    }
}

fn project_positions(mut positionables: Query<(&mut Transform, &Position)>) {
    for (mut transform, position) in &mut positionables {
        transform.translation = position.0.extend(0.);
    }
}

fn move_ball(ball: Single<(&mut Position, &Velocity), With<Ball>>) {
    let (mut position, velocity) = ball.into_inner();
    position.0 += velocity.0 * BALL_SPEED;
}

fn handle_collisions(
    ball: Single<(&mut Velocity, &mut Position, &Collider), With<Ball>>,
    other_things: Query<(&Position, &Collider), Without<Ball>>,
    mut rng: ResMut<Rng>,
) {
    let (mut ball_velocity, mut ball_position, ball_collider) = ball.into_inner();

    for (other_position, other_collider) in &other_things {
        if let Some(collision) = collide_with_side(
            Aabb2d::new(ball_position.0, ball_collider.half_size()),
            Aabb2d::new(other_position.0, other_collider.half_size()),
        ) {
            let rnd = (rng.0.f32_inclusive() - 0.5) * FRACTIONAL_BALL_SPEED;
            let temp_rnd = BALL_SPEED - rnd;
            let other_rnd = sqrt(2.0 * SQUARE_BALL_SPEED - temp_rnd * temp_rnd);
            match collision {
                Collision::Left => {
                    ball_position.0.x = other_position.0.x
                        - other_collider.half_size().x
                        - ball_collider.half_size().x;
                    ball_velocity.0 = Vec2::new(-temp_rnd, other_rnd * ball_velocity.0.y.signum());
                }

                Collision::Right => {
                    ball_position.0.x = other_position.0.x
                        + other_collider.half_size().x
                        + ball_collider.half_size().x;
                    ball_velocity.0 = Vec2::new(temp_rnd, other_rnd * ball_velocity.0.y.signum());
                }

                Collision::Top => {
                    ball_position.0.y = other_position.0.y
                        + other_collider.half_size().y
                        + ball_collider.half_size().x;
                    ball_velocity.0 = Vec2::new(other_rnd * ball_velocity.0.x.signum(), temp_rnd);
                }

                Collision::Bottom => {
                    ball_position.0.y = other_position.0.y
                        - other_collider.half_size().y
                        - ball_collider.half_size().x;
                    ball_velocity.0 = Vec2::new(other_rnd * ball_velocity.0.x.signum(), -temp_rnd);
                }
            }
        }
    }
}

fn handle_player_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut paddle_velocity: Single<&mut Velocity, With<Player>>,
) {
    if keyboard_input.pressed(KeyCode::ArrowUp) || keyboard_input.pressed(KeyCode::KeyW) {
        paddle_velocity.0.y = PADDLE_SPEED;
    } else if keyboard_input.pressed(KeyCode::ArrowDown) || keyboard_input.pressed(KeyCode::KeyS) {
        paddle_velocity.0.y = -PADDLE_SPEED;
    } else {
        paddle_velocity.0.y = 0.;
    }
}

#[allow(clippy::type_complexity)]
fn constrain_paddle_position(
    mut paddles: Query<(&mut Position, &Collider), (With<Paddle>, Without<Gutter>)>,
    gutters: Query<(&Position, &Collider), (With<Gutter>, Without<Paddle>)>,
) {
    for (mut paddle_position, paddle_collider) in &mut paddles {
        for (gutter_position, gutter_collider) in &gutters {
            let paddle_aabb = Aabb2d::new(paddle_position.0, paddle_collider.half_size());
            let gutter_aabb = Aabb2d::new(gutter_position.0, gutter_collider.half_size());

            if let Some(collision) = collide_with_side(paddle_aabb, gutter_aabb) {
                match collision {
                    Collision::Top => {
                        paddle_position.0.y = gutter_position.0.y
                            + gutter_collider.half_size().y
                            + paddle_collider.half_size().y;
                    }

                    Collision::Bottom => {
                        paddle_position.0.y = gutter_position.0.y
                            - gutter_collider.half_size().y
                            - paddle_collider.half_size().y;
                    }

                    _ => {}
                }
            }
        }
    }
}

fn move_paddles(mut paddles: Query<(&mut Position, &Velocity), With<Paddle>>) {
    for (mut position, velocity) in &mut paddles {
        position.0 += velocity.0;
    }
}

fn move_ai(
    ai: Single<(&mut Velocity, &Position), With<Ai>>,
    ball: Single<&Position, With<Ball>>,
    time: Res<Time>,
    mut timer: ResMut<AiMovementTimer>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let (mut velocity, position) = ai.into_inner();
        let a_to_b = ball.0 - position.0;
        velocity.0.y = a_to_b.y.signum() * PADDLE_SPEED;
    }
}

fn collide_with_side(ball: Aabb2d, wall: Aabb2d) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let offset = ball.center() - wall.closest_point(ball.center());

    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x < 0. {
            Collision::Left
        } else {
            Collision::Right
        }
    } else if offset.y > 0. {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}

fn detect_goal(
    ball: Single<(&Position, &Collider), With<Ball>>,
    player: Single<Entity, (With<Player>, Without<Ai>)>,
    ai: Single<Entity, (With<Ai>, Without<Player>)>,
    window: Single<&Window>,
    mut commands: Commands,
) {
    let (ball_position, ball_collider) = ball.into_inner();
    let half_window_size = window.resolution.size() / 2.;

    if ball_position.0.x + ball_collider.half_size().x > half_window_size.x {
        commands.trigger(Scored { scorer: *player });
    }

    if ball_position.0.x + ball_collider.half_size().x < -half_window_size.x {
        commands.trigger(Scored { scorer: *ai });
    }
}

fn reset_ball(
    event: On<Scored>,
    ball: Single<(&mut Position, &mut Velocity), With<Ball>>,
    is_player: Query<&Player>,
    mut rng: ResMut<Rng>,
) {
    let (mut ball_position, mut ball_velocity) = ball.into_inner();
    ball_position.0 = Vec2::ZERO;
    let rnd = (rng.0.f32_inclusive() - 0.5) * FRACTIONAL_BALL_SPEED;
    let temp_rnd = BALL_SPEED - rnd;
    let other_rnd = sqrt(2.0 * SQUARE_BALL_SPEED - temp_rnd * temp_rnd) * {
        if rng.0.bool() { 1. } else { -1. }
    };

    let v_x_mult = if is_player.get(event.scorer).is_ok() {
        1.
    } else {
        -1.
    };

    let v_y_mult = if rng.0.bool() { 1. } else { -1. };

    ball_velocity.0 = Vec2::new(v_x_mult * (-BALL_SPEED + rnd), v_y_mult * other_rnd);
}

fn update_score(
    event: On<Scored>,
    mut score: ResMut<Score>,
    is_ai: Query<&Ai>,
    is_player: Query<&Player>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if is_ai.get(event.scorer).is_ok() {
        score.ai += 1;
        info!("AI scored! {} – {}", score.player, score.ai);
    }

    if is_player.get(event.scorer).is_ok() {
        score.player += 1;
        info!("Player scored! {} – {}", score.player, score.ai);
    }

    if score.ai == 10 || score.player == 10 {
        next_state.set(GameState::End);
    }
}

fn update_scoreboard(
    mut player_score: Single<&mut Text, (With<PlayerScore>, Without<AiScore>)>,
    mut ai_score: Single<&mut Text, (With<AiScore>, Without<PlayerScore>)>,
    score: Res<Score>,
) {
    if score.is_changed() {
        player_score.0 = score.player.to_string();
        ai_score.0 = score.ai.to_string();
    }
}
