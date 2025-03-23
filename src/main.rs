use std::f32::consts::PI;

use macroquad::prelude::*;

struct Rail {
    position: Vec2,
    points: Vec<Vec2>,
    is_wall: bool,
}

struct Fork {
    rail1: Rail,
    rail2: Rail,
    which: bool,
}

impl Fork {
    fn is_wall(&self) -> bool {
        if self.which {
            self.rail1.is_wall
        } else {
            self.rail2.is_wall
        }
    }
}

enum Block {
    Rail(Rail),
    Fork(Fork),
}

impl Block {
    fn is_wall(&self) -> bool {
        match self {
            Block::Rail(rail) => rail.is_wall,
            Block::Fork(fork) => fork.is_wall(),
        }
    }
}

struct World {
    rails: Vec<Block>,
}

impl World {
    fn find_next_fork_index(&self, starting_idx: usize) -> Option<usize> {
        let mut idx = starting_idx;

        // Check all rails starting from the current one
        for _ in 0..self.rails.len() {
            if let Block::Fork(_) = &self.rails[idx] {
                return Some(idx);
            }

            idx = (idx + 1) % self.rails.len();
        }

        None
    }
}

struct State {
    current_rail_idx: usize,
    current_point_idx: usize,
    ms_timer: f32,
    alive: bool,
}

impl State {
    fn new() -> Self {
        Self {
            current_rail_idx: 0,
            current_point_idx: 0,
            ms_timer: 0.0,
            alive: true,
        }
    }

    fn get_current_rail<'a>(&self, world: &'a World) -> &'a Block {
        &world.rails[self.current_rail_idx]
    }

    fn get_next_rail<'a>(&self, world: &'a World) -> &'a Block {
        &world.rails[(self.current_rail_idx + 1) % world.rails.len()]
    }

    fn get_current_point_world_position(&self, world: &World) -> Vec2 {
        let current_rail = self.get_current_rail(world);

        match current_rail {
            Block::Rail(rail) => rail.position + rail.points[self.current_point_idx],
            Block::Fork(fork) => {
                if fork.which {
                    fork.rail1.position + fork.rail1.points[self.current_point_idx]
                } else {
                    fork.rail2.position + fork.rail2.points[self.current_point_idx]
                }
            }
        }
    }

    fn get_current_rail_points<'a>(&mut self, world: &'a World) -> &'a Vec<Vec2> {
        let current_rail = self.get_current_rail(world);

        match current_rail {
            Block::Rail(rail) => &rail.points,
            Block::Fork(fork) => {
                if fork.which {
                    &fork.rail1.points
                } else {
                    &fork.rail2.points
                }
            }
        }
    }

    fn get_next_point_world_position(&self, world: &World) -> Vec2 {
        let current_rail = self.get_current_rail(world);
        let next_rail = self.get_next_rail(world);

        let current_rail_points = match current_rail {
            Block::Rail(rail) => &rail.points,
            Block::Fork(fork) => {
                if fork.which {
                    &fork.rail1.points
                } else {
                    &fork.rail2.points
                }
            }
        };

        let current_rail_position = match current_rail {
            Block::Rail(rail) => rail.position,
            Block::Fork(fork) => {
                if fork.which {
                    fork.rail1.position
                } else {
                    fork.rail2.position
                }
            }
        };

        let next_rail_points = match next_rail {
            Block::Rail(rail) => &rail.points,
            Block::Fork(fork) => {
                if fork.which {
                    &fork.rail1.points
                } else {
                    &fork.rail2.points
                }
            }
        };

        let next_rail_position = match next_rail {
            Block::Rail(rail) => rail.position,
            Block::Fork(fork) => {
                if fork.which {
                    fork.rail1.position
                } else {
                    fork.rail2.position
                }
            }
        };

        if current_rail_points.len() > self.current_point_idx + 1 {
            current_rail_points[self.current_point_idx + 1] + current_rail_position
        } else {
            next_rail_points[0] + next_rail_position
        }
    }

    fn get_next_next_point_world_position(&self, world: &World) -> Vec2 {
        let current_rail = self.get_current_rail(world);
        let next_rail = self.get_next_rail(world);

        let current_rail_points = match current_rail {
            Block::Rail(rail) => &rail.points,
            Block::Fork(fork) => {
                if fork.which {
                    &fork.rail1.points
                } else {
                    &fork.rail2.points
                }
            }
        };

        let current_rail_position = match current_rail {
            Block::Rail(rail) => rail.position,
            Block::Fork(fork) => {
                if fork.which {
                    fork.rail1.position
                } else {
                    fork.rail2.position
                }
            }
        };

        let next_rail_points = match next_rail {
            Block::Rail(rail) => &rail.points,
            Block::Fork(fork) => {
                if fork.which {
                    &fork.rail1.points
                } else {
                    &fork.rail2.points
                }
            }
        };

        let next_rail_position = match next_rail {
            Block::Rail(rail) => rail.position,
            Block::Fork(fork) => {
                if fork.which {
                    fork.rail1.position
                } else {
                    fork.rail2.position
                }
            }
        };

        if current_rail_points.len() > self.current_point_idx + 2 {
            current_rail_points[self.current_point_idx + 2] + current_rail_position
        } else {
            next_rail_points[1] + next_rail_position
        }
    }

    fn move_to_next_point(&mut self, world: &World) {
        self.current_point_idx += 1;

        let current_len = self.get_current_rail_points(world).len();
        let current_rail = &world.rails[self.current_rail_idx];
        if current_rail.is_wall() && self.current_point_idx >= current_len -1 {
            self.alive = false;
            self.current_point_idx = current_len - 2;
            return;
        }

        if self.current_point_idx >= current_len {
            self.current_point_idx = 0;
            self.current_rail_idx += 1;

            if self.current_rail_idx >= world.rails.len() {
                self.current_rail_idx = 0;
            }
        }
    }
}

fn interpolate_angle(start: f32, end: f32, t: f32) -> f32 {
    let two_pi = 2.0 * PI;
    // Compute the difference, ensuring it is in the range [-PI, PI]
    let diff = (end - start + PI).rem_euclid(two_pi) - PI;
    // Interpolate by moving from start towards the target angle by t percent of the difference
    let angle = start + diff * t;
    // Normalize the result to the range [0, 2π)
    angle.rem_euclid(two_pi)
}

impl Rail {
    fn new_curved(
        position: Vec2,
        dist: f32,
        n_points: usize,
        start_angle: f32,
        angle_step: f32,
        is_wall: bool,
    ) -> Self {
        let mut circle_points = vec![];

        circle_points.push(vec2(0.0, 0.0));

        for i in 1..n_points {
            let prev_point = circle_points[i - 1];
            let x = dist * (start_angle + i as f32 * angle_step).cos();
            let y = dist * (start_angle + i as f32 * angle_step).sin();
            circle_points.push(vec2(prev_point.x + x, prev_point.y + y));
        }

        Self {
            position,
            points: circle_points,
            is_wall,
        }
    }

    fn new_straight(
        position: Vec2,
        dist: f32,
        n_points: usize,
        start_angle: f32,
        is_wall: bool,
    ) -> Self {
        Self::new_curved(position, dist, n_points, start_angle, 0.0, is_wall)
    }
}

fn get_last_rail_world_position(world: &World) -> Vec2 {
    let last_rail = world.rails.last().unwrap();

    match last_rail {
        Block::Rail(rail) => {
            let last_point = rail.points.last().unwrap();
            rail.position + *last_point
        }
        Block::Fork(fork) => {
            if fork.which {
                let last_point = fork.rail1.points.last().unwrap();
                fork.rail1.position + *last_point
            } else {
                let last_point = fork.rail2.points.last().unwrap();
                fork.rail2.position + *last_point
            }
        }
    }
}

fn draw_rail(position: Vec2, points: &Vec<Vec2>, rail_color: Color) {
    let mut color = WHITE;

    for point in points {
        let point_world_position = position + *point;

        draw_circle(point_world_position.x, point_world_position.y, 5.0, color);
        color = BLUE;
    }

    for point_pair in points.windows(2) {
        let point_world_position_1 = position + point_pair[0];
        let point_world_position_2 = position + point_pair[1];

        draw_line(
            point_world_position_1.x,
            point_world_position_1.y,
            point_world_position_2.x,
            point_world_position_2.y,
            5.0,
            rail_color,
        );
    }
}

/// Moves `current` exponentially toward `target`.
///
/// - `speed` controls how fast the approach is (higher means faster).
/// - `dt` is the time delta (typically from `get_frame_time()`).
///
/// The exponential smoothing factor is calculated as:
/// t = 1.0 - exp(-speed * dt)
///
/// This value is then used in linear interpolation.
fn exponential_approach_vec2(current: Vec2, target: Vec2, speed: f32, dt: f32) -> Vec2 {
    // Calculate the interpolation factor
    let t = 1.0 - (-speed * dt).exp();
    // Lerp between current and target using the factor
    current.lerp(target, t)
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut world = World { rails: Vec::new() };

    world.rails.push(Block::Rail(Rail::new_straight(
        vec2(100.0, 100.0),
        30.0,
        8,
        0.0,
        false,
    )));

    world.rails.push(Block::Rail(Rail::new_straight(
        get_last_rail_world_position(&world),
        30.0,
        8,
        0.0,
        false,
    )));

    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(&world),
        30.0,
        8,
        0.0,
        PI / 8.0,
        false,
    )));

    world.rails.push(Block::Rail(Rail::new_straight(
        get_last_rail_world_position(&world),
        30.0,
        8,
        PI,
        false,
    )));

    world.rails.push(Block::Rail(Rail::new_curved(
        get_last_rail_world_position(&world),
        30.0,
        8,
        PI / 2.0,
        PI / 8.0,
        false,
    )));

    world.rails.push(Block::Fork(Fork {
        which: false,
        rail1: Rail::new_straight(
            get_last_rail_world_position(&world),
            30.0,
            8,
            PI + PI / 6.0,
            false,
        ),
        rail2: Rail::new_straight(
            get_last_rail_world_position(&world),
            30.0,
            8,
            PI + -PI / 6.0,
            true,
        ),
    }));

    let mut state = State::new();
    let ms_to_next_point = 100.0;

    let mut camera_pos = vec2(0.0, 0.0);

    loop {
        clear_background(RED);

        for rail in &world.rails {
            match rail {
                Block::Rail(rail) => {
                    draw_rail(rail.position, &rail.points, BLUE);
                }
                Block::Fork(fork) => {
                    let color1 = if fork.which { BLUE } else { GRAY };
                    let color2 = if fork.which { GRAY } else { BLUE };

                    draw_rail(fork.rail1.position, &fork.rail1.points, color1);
                    draw_rail(fork.rail2.position, &fork.rail2.points, color2);
                }
            }
        }

        let current_point_world_position = state.get_current_point_world_position(&world);
        let next_point_world_position = state.get_next_point_world_position(&world);
        let next_next_point_world_position = state.get_next_next_point_world_position(&world);

        if current_point_world_position == next_point_world_position {
            state.move_to_next_point(&world);
            continue;
        }

        let rotation0 = (next_point_world_position - current_point_world_position).to_angle();
        let rotation1 = (next_next_point_world_position - next_point_world_position).to_angle();

        let progress = state.ms_timer / ms_to_next_point;

        let train_position = current_point_world_position.lerp(next_point_world_position, progress);
        let train_rotation = interpolate_angle(rotation0, rotation1, progress);

        let train_width = 80.0;
        let train_height = 40.0;

        draw_rectangle_ex(
            train_position.x,
            train_position.y,
            train_width,
            train_height,
            DrawRectangleParams {
                offset: vec2(0.5, 0.5),
                rotation: train_rotation,
                color: GREEN,
            },
        );

        if state.alive {
            state.ms_timer += get_frame_time() * 1000.0;
        }

        if state.ms_timer >= ms_to_next_point {
            state.ms_timer = 0.0;
            state.move_to_next_point(&world);
        }

        camera_pos = exponential_approach_vec2(camera_pos, train_position, 10.0, get_frame_time());

        macroquad::camera::set_camera(&Camera2D {
            zoom: vec2(0.0025, 0.0025),
            target: camera_pos,
            render_target: None,
            ..Default::default()
        });

        if is_key_pressed(KeyCode::Space) {
            if let Some(fork_idx) = world.find_next_fork_index(state.current_rail_idx) {
                if let Block::Fork(fork) = &mut world.rails[fork_idx] {
                    fork.which = !fork.which;
                }
            }
        }

        next_frame().await
    }
}
