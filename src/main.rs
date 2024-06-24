use bevy::{
    input::{
        mouse::{MouseButtonInput, MouseMotion},
        ButtonState,
    },
    math::vec2,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    window::{PresentMode, WindowTheme},
};

// cargo run --release
// cargo build --release --target wasm32-unknown-unknown
// wasm-bindgen --no-typescript --target wasm32-unknown-unknown web --out-dire ./out/ --out-name "sketch" ./target/wasm32-unknown-unknown/release/sketch.wasm
// basic-http-server out
// zip out/release.zip -j out/*
// butler push out/release.zip aaratha/rope-survivor:html5

// basic-http-server target/wasm32-unknown-unknown/release
// zip target/wasm32-unknown-unknown/release.zip -j target/wasm32-unknown-unknown/release/*
// butler push target/wasm32-unknown-unknown/release.zip aaratha/rope-survivor:html5
// butler status aaratha/rope-survivor:html5

const ROPE_THICKNESS: f32 = 4.0;
const ROPE_BALL_RADIUS: f32 = 7.0;
const ROPE_COLOR: Color = Color::rgba(0.7, 0.8, 1.0, 1.0);
const SEGMENT_LENGTH: f32 = 10.0;
const NUM_PARTICLES: usize = 10;
const CONSTRAINT_ITERATIONS: usize = 8;
const CONSTRAINT_STRENGTH: f32 = 0.7;

const TIME_STEP: f32 = 0.016;
const FRICTION: f32 = 0.98;
const ROPE_FRICTION: f32 = 0.98;
const SUBSTEPS: usize = 5;
const LERP_FACTOR: f32 = 0.2;

const ENEMY_SPEED: f32 = 2.;
const ENEMY_SPAWN_INTERVAL: f32 = 2.0; // in seconds
const ENEMY_RADIUS: f32 = 10.0;

const POINT_SPAWN_INTERVAL: f32 = 1.0; // in seconds
const MAX_POINTS: usize = 20;
const POINT_RADIUS: f32 = 5.0;

const BORDER_THICKNESS: f32 = 5.0;
const BORDER_COLOR: Color = Color::rgba(1.0, 1.0, 1.0, 0.0); // Adjust border color as needed

const CANVAS_WIDTH: f32 = 400.0; // Set the canvas width based on 16:9 aspect ratio
const CANVAS_HEIGHT: f32 = CANVAS_WIDTH * 16.0 / 9.0; // Calculate canvas height

const DRAG_SENSITIVITY: f32 = 1.9;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "I am a window!".into(),
                name: Some("bevy.app".into()),
                resolution: (CANVAS_WIDTH, CANVAS_HEIGHT).into(),
                present_mode: PresentMode::AutoVsync,
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                },
                visible: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (update_rope_particles, camera_controller))
        .run();
}

#[derive(Resource)]
struct CameraEntity {
    entity: Entity,
}

#[derive(Resource)]
struct ParticleEntities(Vec<Entity>);

#[derive(Resource)]
struct PlayerEntity {
    entity: Entity,
}

#[derive(Component, Clone, Copy)]
struct Particle {
    position: Vec2,
    last_position: Vec2,
    acceleration: Vec2,
    friction: f32,
    radius: f32,
    color: Color,
}

impl Particle {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            last_position: position,
            acceleration: Vec2::ZERO,
            friction: FRICTION,
            radius: ROPE_BALL_RADIUS,
            color: ROPE_COLOR,
        }
    }
}

#[derive(Component)]
struct RopeParameters {
    num_particles: usize,
    segment_length: f32,
    constraint_iterations: usize,
    constraint_strength: f32,
}

#[derive(Component)]
struct Rope {
    particles: Vec<Particle>,
    parameters: RopeParameters,
}

impl Rope {
    fn update(&mut self, target: Vec2) {
        self.particles[0].position = target;
        for _ in 0..self.parameters.constraint_iterations {
            for i in 0..self.particles.len() - 1 {
                let particle_a = self.particles[i];
                let particle_b = self.particles[i + 1];
                let delta = particle_b.position - particle_a.position;
                let delta_length = delta.length();
                let diff = (delta_length - self.parameters.segment_length) / delta_length;
                let offset = delta * diff * self.parameters.constraint_strength / SUBSTEPS as f32;

                if i != 0 {
                    self.particles[i].position += offset;
                }
                self.particles[i + 1].position -= offset;
            }
        }

        for i in 1..self.particles.len() {
            let mut velocity = self.particles[i].position - self.particles[i].last_position;
            velocity *= self.particles[i].friction; // Apply friction to the velocity
            self.particles[i].last_position = self.particles[i].position;
            self.particles[i].position += velocity; // + self.particles[i].acceleration * TIME_STEP * TIME_STEP;
            self.particles[i].acceleration = Vec2::ZERO; // Reset acceleration
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Bundle)]
struct PlayerBundle {
    sprite: SpriteBundle,
    rope: Rope,
    player: Player,
}

impl PlayerBundle {
    fn new(start: Vec2) -> Self {
        let mut particles = Vec::with_capacity(NUM_PARTICLES);
        for i in 0..NUM_PARTICLES {
            particles.push(Particle::new(start + vec2(i as f32 * SEGMENT_LENGTH, 0.0)));
            particles[i].friction = ROPE_FRICTION;
        }
        Self {
            sprite: SpriteBundle {
                sprite: Sprite {
                    color: ROPE_COLOR,
                    custom_size: Some(Vec2::new(ROPE_BALL_RADIUS * 2.0, ROPE_BALL_RADIUS * 2.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            rope: Rope {
                particles,
                parameters: RopeParameters {
                    num_particles: NUM_PARTICLES,
                    segment_length: SEGMENT_LENGTH,
                    constraint_iterations: CONSTRAINT_ITERATIONS,
                    constraint_strength: CONSTRAINT_STRENGTH,
                },
            },
            player: Player,
        }
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn the camera
    let camera_entity = commands.spawn(Camera2dBundle::default()).id();

    // Initialize the rope at the camera's initial position
    let camera_start_position = Vec2::new(0., 0.);
    let player_entity = commands
        .spawn(PlayerBundle::new(camera_start_position))
        .id();

    // Store the camera entity for future reference
    commands.insert_resource(CameraEntity {
        entity: camera_entity,
    });

    // Store player entity for future reference
    commands.insert_resource(PlayerEntity {
        entity: player_entity,
    });

    // Spawn particles as separate entities
    let mut particle_entities = Vec::new();
    for i in 0..NUM_PARTICLES {
        if i == 0 || i == NUM_PARTICLES - 1 {
            let entity = commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Circle {
                        radius: ROPE_BALL_RADIUS,
                    })),
                    material: materials.add(ROPE_COLOR),
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * SEGMENT_LENGTH,
                        0.0,
                        0.0,
                    )),
                    ..Default::default()
                })
                .id();
            particle_entities.push(entity);
        } else {
            let entity = commands
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Circle {
                        radius: ROPE_THICKNESS,
                    })),
                    material: materials.add(ROPE_COLOR),
                    transform: Transform::from_translation(Vec3::new(
                        i as f32 * SEGMENT_LENGTH,
                        0.0,
                        0.0,
                    )),
                    ..Default::default()
                })
                .id();
            particle_entities.push(entity);
        }
    }

    commands.insert_resource(ParticleEntities(particle_entities));
}

fn camera_controller(
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    mut mousemv_evr: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    // Track if the left mouse button is pressed
    static mut IS_PRESSED: bool = false;

    // Handle mouse button events
    for ev in mousebtn_evr.read() {
        if ev.button == MouseButton::Left {
            unsafe {
                IS_PRESSED = ev.state == ButtonState::Pressed;
            }
        }
    }

    // Handle mouse motion events
    for ev in mousemv_evr.read() {
        unsafe {
            if IS_PRESSED {
                for mut transform in query.iter_mut() {
                    transform.translation.x -= ev.delta.x * DRAG_SENSITIVITY;
                    transform.translation.y += ev.delta.y * DRAG_SENSITIVITY;
                }
            }
        }
    }
}

fn update_rope_particles(
    mut param_set: ParamSet<(Query<&mut Transform>, Query<&Transform, With<Camera>>)>,
    mut player_query: Query<&mut Rope>,
    particle_entities: Res<ParticleEntities>,
    time: Res<Time>,
) {
    if let Some(camera_transform) = param_set.p1().iter().next() {
        let camera_position = camera_transform.translation.truncate();
        for mut rope in player_query.iter_mut() {
            let target_position = rope.particles[0]
                .position
                .lerp(camera_position, LERP_FACTOR);
            rope.update(target_position); // Update rope particles based on target position

            // Update the transform of each particle entity
            for (i, particle) in rope.particles.iter().enumerate() {
                if let Ok(mut transform) = param_set.p0().get_mut(particle_entities.0[i]) {
                    transform.translation.x = particle.position.x;
                    transform.translation.y = particle.position.y;
                    // Optionally update other components or systems based on particle state
                }
            }
        }
    }
}
