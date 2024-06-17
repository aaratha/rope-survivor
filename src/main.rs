use macroquad::camera::{set_camera, Camera2D};
use macroquad::prelude::*;
use macroquad::rand::gen_range;

// cargo run --release
// cargo build --release --target wasm32-unknown-unknown
// basic-http-server target/wasm32-unknown-unknown/release
// zip target/wasm32-unknown-unknown/release.zip -j target/wasm32-unknown-unknown/release/*
// butler push target/wasm32-unknown-unknown/release.zip aaratha/rope-survivor:html5
// butler status aaratha/rope-survivor:html5

const ROPE_THICKNESS: f32 = 2.0;
const ROPE_BALL_RADIUS: f32 = 7.0;
const ROPE_COLOR: Color = Color::new(0.7, 0.8, 1.0, 1.0);
const SEGMENT_LENGTH: f32 = 10.0;
const CONSTRAINT_ITERATIONS: usize = 5;
const CONSTRAINT_STRENGTH: f32 = 0.1;

const TIME_STEP: f32 = 0.016;
const FRICTION: f32 = 0.98;
const ROPE_FRICTION: f32 = 0.99;
const SUBSTEPS: usize = 5;
const LERP_FACTOR: f32 = 0.2;

const ENEMY_SPEED: f32 = 2.;
const ENEMY_SPAWN_INTERVAL: f32 = 2.0; // in seconds
const ENEMY_RADIUS: f32 = 10.0;

const POINT_SPAWN_INTERVAL: f32 = 1.0; // in seconds
const MAX_POINTS: usize = 20;
const POINT_RADIUS: f32 = 5.0;

const BORDER_THICKNESS: f32 = 5.0;
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.0); // Adjust border color as needed

const CANVAS_WIDTH: f32 = 400.0; // Set the canvas width based on 16:9 aspect ratio
const CANVAS_HEIGHT: f32 = CANVAS_WIDTH * 16.0 / 9.0; // Calculate canvas height

const DRAG_SENSITIVITY: f32 = 1.9;

fn window_conf() -> Conf {
    Conf {
        window_title: "Window Conf".to_owned(),
        window_height: CANVAS_HEIGHT as i32,
        window_width: CANVAS_WIDTH as i32,
        platform: miniquad::conf::Platform {
            linux_backend: miniquad::conf::LinuxBackend::WaylandOnly,
            ..Default::default()
        },
        ..Default::default()
    }
}

#[derive(Clone, Copy)]
struct Frame {
    width: f32,
    height: f32,
    mouse_held: bool,
}

impl Frame {
    fn new() -> Self {
        Self {
            width: screen_width(),
            height: screen_height(),
            mouse_held: false,
        }
    }

    fn update(&mut self) {
        self.width = screen_width();
        self.height = screen_height();
    }
}

struct FpsCounter {
    last_update: f32,
    fps: f32,
    fps_text: String,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            last_update: 0.0,
            fps: 0.0,
            fps_text: String::new(),
        }
    }

    fn update(&mut self) {
        let current_time = get_time();
        let elapsed_time = current_time - self.last_update as f64;

        if elapsed_time >= 1.0 {
            self.fps = get_fps() as f32;
            self.last_update = current_time as f32;
            self.fps_text = format!("FPS: {:.2}", self.fps);
        }
    }

    fn draw(&self) {
        draw_text(&self.fps_text, screen_width() - 100.0, 20.0, 20.0, WHITE);
    }
}

#[derive(Clone, Copy, PartialEq)]
struct Particle {
    position: Vec2,
    old_position: Vec2,
    acceleration: Vec2,
    friction: f32,
}

impl Particle {
    fn new(position: Vec2) -> Self {
        Self {
            position,
            old_position: position,
            acceleration: Vec2::ZERO,
            friction: FRICTION,
        }
    }

    fn update(&mut self) {
        let mut velocity = self.position - self.old_position;
        velocity *= self.friction; // Apply friction to the velocity
        self.old_position = self.position;
        self.position += velocity; // + self.acceleration * TIME_STEP * TIME_STEP;
        self.acceleration = Vec2::ZERO; // Reset acceleration
    }
}

struct Rope {
    particles: Vec<Particle>,
    thickness: f32,
    ball_radius: f32,
    constraint_strength: f32,
}

impl Rope {
    fn new(start: Vec2, num_particles: usize) -> Self {
        let mut particles = Vec::with_capacity(num_particles);
        for i in 0..num_particles {
            particles.push(Particle::new(start + vec2(i as f32 * SEGMENT_LENGTH, 0.0)));
            particles[i].friction = ROPE_FRICTION;
        }
        Self {
            particles,
            thickness: ROPE_THICKNESS,
            ball_radius: ROPE_BALL_RADIUS,
            constraint_strength: CONSTRAINT_STRENGTH,
        }
    }

    fn update(&mut self, target: Vec2) {
        self.particles[0].position = target;
        for _ in 0..CONSTRAINT_ITERATIONS {
            for i in 0..self.particles.len() - 1 {
                let particle_a = self.particles[i];
                let particle_b = self.particles[i + 1];
                let delta = particle_b.position - particle_a.position;
                let delta_length = delta.length();
                let diff = (delta_length - SEGMENT_LENGTH) / delta_length;
                let offset = delta * diff * self.constraint_strength / SUBSTEPS as f32;

                if i != 0 {
                    self.particles[i].position += offset;
                }
                self.particles[i + 1].position -= offset;
            }
        }

        for i in 1..self.particles.len() {
            self.particles[i].update();
        }
    }

    fn draw(&self) {
        for i in 0..self.particles.len() - 1 {
            draw_line(
                self.particles[i].position.x,
                self.particles[i].position.y,
                self.particles[i + 1].position.x,
                self.particles[i + 1].position.y,
                self.thickness,
                WHITE,
            );
        }
        draw_circle(
            self.particles[0].position.x,
            self.particles[0].position.y,
            self.ball_radius,
            WHITE,
        );
        draw_circle(
            self.particles[self.particles.len() - 1].position.x,
            self.particles[self.particles.len() - 1].position.y,
            self.ball_radius,
            WHITE,
        );
    }
}

struct Enemy {
    particle: Particle,
    active: bool,
    radius: f32,
}

impl Enemy {
    fn new(frame: Frame, target: Vec2) -> Self {
        let spawn_x = if rand::gen_range(0.0, 1.0) > 0.5 {
            gen_range(target.x - frame.width / 2.0, target.x)
        } else {
            gen_range(target.x, target.x + frame.width / 2.0)
        };

        let spawn_y = if rand::gen_range(0.0, 1.0) > 0.5 {
            gen_range(target.y - frame.height / 2.0, target.y)
        } else {
            gen_range(target.y, target.y + frame.height / 2.0)
        };

        Self {
            particle: Particle::new(Vec2::new(spawn_x, spawn_y)),
            active: true,
            radius: ENEMY_RADIUS,
        }
    }

    fn update(&mut self, target: Vec2, frame: Frame) {
        let direction = target - self.particle.position;
        let distance = direction.length();
        if distance > 0.0 {
            let step = direction.normalize() * ENEMY_SPEED * TIME_STEP;
            self.particle.position += step;
        }
        self.particle.update();
    }

    fn draw(&self) {
        if self.active {
            draw_circle(
                self.particle.position.x,
                self.particle.position.y,
                self.radius,
                ROPE_COLOR,
            );
        }
    }
}

struct Point {
    position: Vec2,
    active: bool,
    radius: f32,
}

impl Point {
    fn new(frame: Frame, target: Vec2) -> Self {
        let pos = Vec2::new(
            gen_range(target.x - frame.width / 2.0, target.x + frame.width / 2.0),
            gen_range(target.y - frame.height / 2.0, target.y + frame.height / 2.0),
        );
        Self {
            position: pos,
            active: true,
            radius: POINT_RADIUS,
        }
    }

    fn draw(&self) {
        if self.active {
            draw_circle(
                self.position.x,
                self.position.y,
                self.radius,
                Color::new(1.0, 0.8, 0.0, 1.0),
            );
        }
    }
}

fn check_collisions(
    rope: &mut Rope,
    enemies: &mut [Enemy],
    points: &mut Vec<Point>,
    score: &mut i32,
    game_over: &mut bool, // Pass by mutable reference
) {
    for _ in 0..SUBSTEPS {
        let particle_0 = rope.particles[0].clone();
        for particle in rope.particles.iter_mut() {
            check_enemy_collisions_with_particle(particle, enemies, game_over, particle_0.position);
            check_point_collisions_with_particle(particle, points, score);
        }
    }
}

fn check_enemy_collisions_with_particle(
    particle: &mut Particle,
    enemies: &mut [Enemy],
    game_over: &mut bool,
    player_position: Vec2,
) {
    for enemy in enemies.iter_mut() {
        let dist = enemy.particle.position - particle.position;
        let len = dist.length();
        if len < ROPE_BALL_RADIUS + ENEMY_RADIUS {
            let offset = (ROPE_BALL_RADIUS + ENEMY_RADIUS - len) * dist.normalize();
            enemy.particle.position += offset * 0.5;
            particle.position -= offset * 0.5;
            if particle.position == player_position {
                *game_over = true; // Dereference and modify the original game_over
            }
        }
    }
}

fn check_point_collisions_with_particle(
    particle: &mut Particle,
    points: &mut Vec<Point>,
    score: &mut i32,
) {
    for point in points.iter_mut() {
        let dist = point.position - particle.position;
        let len = dist.length();
        if len < POINT_RADIUS + ENEMY_RADIUS {
            point.active = false;
            *score += 1;
        }
    }
}

fn check_enemy_collisions(enemies: &mut [Enemy]) {
    for i in 0..enemies.len() {
        for j in (i + 1)..enemies.len() {
            let dist = enemies[j].particle.position - enemies[i].particle.position;
            let len = dist.length();
            if len < ENEMY_RADIUS * 2.0 {
                let offset = (ENEMY_RADIUS * 2.0 - len) * dist.normalize();
                enemies[i].particle.position -= offset * 0.5;
                enemies[j].particle.position += offset * 0.5;
            }
        }
    }
}

fn draw_ring(rope: &Rope) {
    let center = rope.particles[0].position;
    let radius = 200.0; // Adjust the radius as needed
    let color = Color::new(1.0, 1.0, 1.0, 0.5); // Adjust the color and alpha as needed
    draw_circle_lines(center.x, center.y, radius, 2.0, color); // Adjust the line thickness as needed
}

fn is_in_frame(particle: &Particle, frame: Frame) -> bool {
    let x = particle.position.x;
    let y = particle.position.y;
    x >= (screen_width() - frame.width) / 2.
        && x <= (screen_width() + frame.width) / 2.
        && y >= (screen_height() - frame.height) / 2.
        && y <= (screen_height() + frame.height) / 2.
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game_over = false;
    let mut rope = Rope::new(vec2(0.0, 100.0), 10);
    let mut enemies: Vec<Enemy> = Vec::new();
    let mut points: Vec<Point> = Vec::new();
    let mut last_spawn_time = get_time();
    let mut last_point_spawn_time = get_time();
    let mut score = 0;
    let mut frame = Frame::new();
    let mut target = Vec2::ZERO;
    let mut last_target = Vec2::ZERO; // Track the last target position
    let mut camera_target = Vec2::ZERO;
    let mut start_drag_position = Vec2::ZERO;

    // let mut fps_counter = FpsCounter::new();

    let canvas = render_target(CANVAS_WIDTH as u32, CANVAS_HEIGHT as u32);
    canvas.texture.set_filter(FilterMode::Nearest);

    loop {
        // fps_counter.update();
        // fps_counter.draw();
        frame.update();

        // camera_target = update_camera(frame, target);
        camera_target = camera_target.lerp(target, 0.1);

        set_camera(&Camera2D {
            target: camera_target,
            render_target: Some(canvas.clone()), // Clone the canvas to avoid move error
            zoom: Vec2::new(CANVAS_HEIGHT / CANVAS_WIDTH, 1.0) * 0.003,
            ..Default::default()
        });

        clear_background(BLACK);
        if game_over {
            // Handle game over screen
            clear_background(BLACK);
            set_default_camera();

            // Draw the canvas to the screen
            draw_texture(
                &canvas.texture,
                (screen_width() - CANVAS_WIDTH) / 2.0,
                (screen_height() - CANVAS_HEIGHT) / 2.0,
                WHITE,
            );
            draw_text(
                &format!("Game Over!"),
                CANVAS_WIDTH / 2. - 85.,
                CANVAS_HEIGHT / 2. - 50.,
                40.,
                WHITE,
            );
            draw_text(
                &format!("Your score is: {}", score),
                CANVAS_WIDTH / 2. - 140.,
                CANVAS_HEIGHT / 2. - 20.,
                40.,
                WHITE,
            );
            if is_mouse_button_pressed(MouseButton::Left) {
                let mouse_position: Vec2 = mouse_position().into();
                if mouse_position.x >= CANVAS_WIDTH / 2. - 100.
                    && mouse_position.x <= CANVAS_WIDTH / 2. + 100.
                    && mouse_position.y >= CANVAS_HEIGHT / 2.
                    && mouse_position.y <= CANVAS_HEIGHT / 2. + 50.
                {
                    // Reset the game
                    game_over = false;
                    rope = Rope::new(vec2(0.0, 100.0), 10);
                    enemies.clear();
                    points.clear();
                    score = 0;
                    last_spawn_time = get_time();
                    last_point_spawn_time = get_time();
                }
            }

            // Draw replay button
            draw_rectangle(
                CANVAS_WIDTH / 2. - 100.,
                CANVAS_HEIGHT / 2.,
                200.,
                50.,
                BLUE,
            );
            draw_text(
                "Replay",
                CANVAS_WIDTH / 2. - 50.,
                CANVAS_HEIGHT / 2. + 30.,
                30.,
                WHITE,
            );

            next_frame().await;
            continue;
        }

        let mouse_position: Vec2 = mouse_position().into();
        if is_mouse_button_down(MouseButton::Left) {
            if !frame.mouse_held {
                start_drag_position = mouse_position;
                last_target = target;
            }
            frame.mouse_held = true;

            // Calculate the drag vector relative to start_drag_position
            let drag_vector = (mouse_position - start_drag_position) * DRAG_SENSITIVITY;

            // Update target based on drag direction and magnitude relative to last_target
            target = last_target + drag_vector;

            // Smoothly move target towards last_target using LERP
            target += (last_target - target) * LERP_FACTOR;
        } else {
            frame.mouse_held = false;
        }

        for _ in 0..SUBSTEPS {
            rope.update(target);
            check_collisions(
                &mut rope,
                &mut enemies,
                &mut points,
                &mut score,
                &mut game_over,
            );
            check_enemy_collisions(&mut enemies);
        }

        if get_time() - last_spawn_time >= ENEMY_SPAWN_INTERVAL as f64 {
            enemies.push(Enemy::new(frame, target));
            last_spawn_time = get_time();
        }

        if get_time() - last_point_spawn_time >= POINT_SPAWN_INTERVAL as f64
            && points.len() < MAX_POINTS
        {
            points.push(Point::new(frame, target));
            last_point_spawn_time = get_time();
        }

        for enemy in &mut enemies {
            enemy.update(rope.particles[0].position, frame);
        }

        for enemy in &mut enemies {
            enemy.particle.update();
        }

        points.retain(|point| point.active);
        enemies.retain(|enemy| enemy.active);

        rope.draw();

        for enemy in &enemies {
            enemy.draw();
        }

        for point in &points {
            point.draw();
        }
        set_default_camera();

        // Draw the canvas to the screen
        draw_texture(
            &canvas.texture,
            (screen_width() - CANVAS_WIDTH) / 2.0,
            (screen_height() - CANVAS_HEIGHT) / 2.0,
            WHITE,
        );

        draw_text(&format!("Score: {}", score), 20.0, 20.0, 30.0, WHITE);

        draw_rectangle_lines(
            (screen_width() - CANVAS_WIDTH) / 2.0,
            (screen_height() - CANVAS_HEIGHT) / 2.0,
            frame.width,
            frame.height,
            BORDER_THICKNESS,
            BORDER_COLOR,
        );

        next_frame().await;
    }
}
