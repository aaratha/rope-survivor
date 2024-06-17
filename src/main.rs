use macroquad::prelude::*;
use macroquad::rand::gen_range;

// cargo run --release
// cargo build --release --target wasm32-unknown-unknown
// basic-http-server target/wasm32-unknown-unknown/release
// zip target/wasm32-unknown-unknown/release.zip -j target/wasm32-unknown-unknown/release/*
// butler push target/wasm32-unknown-unknown/release.zip aaratha/rope:html5
// butler status aaratha/rope:html5

const ROPE_THICKNESS: f32 = 2.0;
const ROPE_BALL_RADIUS: f32 = 7.0;
const ROPE_COLOR: Color = Color::new(0.7, 0.8, 1.0, 1.0);
const SEGMENT_LENGTH: f32 = 10.0;
const CONSTRAINT_ITERATIONS: usize = 8;

const TIME_STEP: f32 = 0.016;
const FRICTION: f32 = 0.98;
const SUBSTEPS: usize = 5;
const LERP_FACTOR: f32 = 0.5;

const ENEMY_SPEED: f32 = 7.0;
const ENEMY_SPAWN_INTERVAL: f32 = 2.0; // in seconds
const ENEMY_RADIUS: f32 = 10.0;

const POINT_SPAWN_INTERVAL: f32 = 1.0; // in seconds
const MAX_POINTS: usize = 20;
const POINT_RADIUS: f32 = 5.0;

const BORDER_THICKNESS: f32 = 5.0;
const BORDER_COLOR: Color = Color::new(1.0, 1.0, 1.0, 0.0); // Adjust border color as needed

#[derive(Clone, Copy)]
struct Frame {
    width: f32,
    height: f32,
}

impl Frame {
    fn new() -> Self {
        Self {
            width: screen_width(),
            height: screen_height(),
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
        }
        Self {
            particles,
            thickness: ROPE_THICKNESS,
            ball_radius: ROPE_BALL_RADIUS,
            constraint_strength: 0.5,
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

    fn extend(&mut self) {
        let last_particle = self.particles.last().unwrap();
        let direction = last_particle.position - self.particles[self.particles.len() - 2].position;
        let new_particle = Particle::new(last_particle.position + direction);
        self.particles.push(new_particle);
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
    fn new(frame: Frame) -> Self {
        let pos = if gen_range(0., 1.) > 0.5 {
            // Spawn on the left or right side of the rectangle
            Vec2::new(
                if gen_range(0., 1.) > 0.5 {
                    (screen_width() - frame.width) / 2.
                } else {
                    (screen_width() + frame.width) / 2.
                },
                rand::gen_range(
                    (screen_height() - frame.height) / 2.,
                    (screen_height() + frame.height) / 2.,
                ),
            )
        } else {
            // Spawn on the top or bottom side of the rectangle
            Vec2::new(
                rand::gen_range(
                    (screen_width() - frame.width) / 2.,
                    (screen_width() + frame.width) / 2.,
                ),
                if gen_range(0., 1.) > 0.5 {
                    (screen_height() - frame.height) / 2.
                } else {
                    (screen_height() + frame.height) / 2.
                },
            )
        };
        Self {
            particle: Particle::new(pos),
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
        if !is_in_frame(&self.particle, frame) {
            self.active = false;
        }
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
    fn new(frame: Frame) -> Self {
        let pos = Vec2::new(
            rand::gen_range(
                (screen_width() - frame.width) / 2.,
                (screen_width() + frame.width) / 2.,
            ),
            rand::gen_range(
                (screen_height() - frame.height) / 2.,
                (screen_height() + frame.height) / 2.,
            ),
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

#[macroquad::main("Rope Simulation")]
async fn main() {
    let mut game_over = false;
    let mut rope = Rope::new(vec2(0.0, 100.0), 10);
    let mut enemies: Vec<Enemy> = Vec::new();
    let mut points: Vec<Point> = Vec::new();
    let mut last_spawn_time = get_time();
    let mut last_point_spawn_time = get_time();
    let mut score = 0;
    let mut last_extended_score = 0;
    let mut frame = Frame::new();
    // let mut fps_counter = FpsCounter::new();

    loop {

        // fps_counter.update();
        // fps_counter.draw();

        if game_over {
            clear_background(BLACK);
            draw_text(
                &format!("Game Over!"),
                screen_width() / 2. - 85.,
                screen_height() / 2. - 50.,
                40.,
                WHITE,
            );
            draw_text(
                &format!("Your score is: {}", score),
                screen_width() / 2. - 140.,
                screen_height() / 2. - 20.,
                40.,
                WHITE,
            );
            if is_mouse_button_pressed(MouseButton::Left) {
                let mouse_position: Vec2 = mouse_position().into();
                if mouse_position.x >= screen_width() / 2. - 100.
                    && mouse_position.x <= screen_width() / 2. + 100.
                    && mouse_position.y >= screen_height() / 2.
                    && mouse_position.y <= screen_height() / 2. + 50.
                {
                    // Reset the game
                    game_over = false;
                    rope = Rope::new(vec2(0.0, 100.0), 10);
                    enemies.clear();
                    points.clear();
                    score = 0;
                    last_spawn_time = get_time();
                    last_point_spawn_time = get_time();
                    last_extended_score = 0;
                }
            }

            // Draw replay button
            draw_rectangle(
                screen_width() / 2. - 100.,
                screen_height() / 2.,
                200.,
                50.,
                BLUE,
            );
            draw_text(
                "Replay",
                screen_width() / 2. - 50.,
                screen_height() / 2. + 30.,
                30.,
                WHITE,
            );

            next_frame().await;
            continue;
        }

        let mouse_position: Vec2 = mouse_position().into();
        let target = rope.particles[0].position
            + (mouse_position - rope.particles[0].position) * LERP_FACTOR;

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
            enemies.push(Enemy::new(frame));
            last_spawn_time = get_time();
        }

        if get_time() - last_point_spawn_time >= POINT_SPAWN_INTERVAL as f64
            && points.len() < MAX_POINTS
        {
            points.push(Point::new(frame));
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

        draw_text(&format!("Score: {}", score), 20.0, 20.0, 30.0, WHITE);

        draw_rectangle_lines(
            (screen_width() - frame.width) / 2.,
            (screen_height() - frame.height) / 2.,
            frame.width,
            frame.height,
            BORDER_THICKNESS,
            BORDER_COLOR,
        );

        if score % 5 == 0 && score != last_extended_score {
            rope.extend();
            last_extended_score = score;
            if rope.constraint_strength < 1.5 {
                rope.constraint_strength += 0.1;
            }
        }
        frame.update();

        next_frame().await;
    }
}
