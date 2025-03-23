use std::f32::consts::PI;

use macroquad::prelude::*;

struct Rail {
    position: Vec2,
    points: Vec<Vec2>,
}

struct World {
    rails: Vec<Rail>,
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
            position: position,
            points: circle_points,
        }
    }

    fn new_straight(position: Vec2, dist: f32, n_points: usize, start_angle: f32) -> Self {
        Self::new_curved(position, dist, n_points, start_angle, 0.0)
    }
}

fn get_last_rail_world_position(world: &World) -> Vec2 {
    let last_rail = world.rails.last().unwrap();
    let last_point = world.rails.last().unwrap().points.last().unwrap();

    last_rail.position + *last_point
}

#[macroquad::main("MyGame")]
async fn main() {
    let mut world = World { rails: Vec::new() };

    world
        .rails
        .push(Rail::new_straight(vec2(100.0, 100.0), 30.0, 8, 0.0));

    world.rails.push(Rail::new_straight(
        get_last_rail_world_position(&world),
        30.0,
        8,
        0.0,
    ));

    world.rails.push(Rail::new_curved(
        get_last_rail_world_position(&world),
        30.0,
        8,
        0.0,
        PI / 8.0,
    ));

    world.rails.push(Rail::new_straight(
        get_last_rail_world_position(&world),
        30.0,
        8,
        PI,
    ));

    world.rails.push(Rail::new_curved(
        get_last_rail_world_position(&world),
        30.0,
        8,
        PI / 2.0,
        PI / 8.0,
    ));

    let mut current_point_idx = 0;
    let mut current_rail_idx = 0;

    let mut ms_timer = 0.0;
    let ms_to_next_point = 100.0;

    loop {
        clear_background(RED);

        for rail in &world.rails {
            let mut color = WHITE;

            for point in &rail.points {
                let point_world_position = rail.position + *point;

                draw_circle(point_world_position.x, point_world_position.y, 5.0, color);
                color = BLUE;
            }

            for point_pair in rail.points.windows(2) {
                let point_world_position_1 = rail.position + point_pair[0];
                let point_world_position_2 = rail.position + point_pair[1];

                draw_line(
                    point_world_position_1.x,
                    point_world_position_1.y,
                    point_world_position_2.x,
                    point_world_position_2.y,
                    5.0,
                    BLUE,
                );
            }
        }

        let current_rail = &world.rails[current_rail_idx];
        let next_rail = &world.rails[(current_rail_idx + 1) % world.rails.len()];

        let current_point = current_rail.points[current_point_idx];
        let current_point_world_position = current_rail.position + current_point;

        let next_point_world_position = if current_rail.points.len() > current_point_idx + 1 {
            current_rail.points[current_point_idx + 1] + current_rail.position
        } else {
            next_rail.points[0] + next_rail.position
        };

        let next_next_point_world_position = if current_rail.points.len() > current_point_idx + 2 {
            current_rail.points[current_point_idx + 2] + current_rail.position
        } else {
            next_rail.points[1] + next_rail.position
        };

        if current_point_world_position == next_point_world_position {
            current_point_idx += 1;
            if current_point_idx >= current_rail.points.len() {
                current_point_idx = 0;
                current_rail_idx += 1;
                if current_rail_idx >= world.rails.len() {
                    current_rail_idx = 0;
                }
            }

            continue;
        }

        let rotation0 = (next_point_world_position - current_point_world_position).to_angle();
        let rotation1 = (next_next_point_world_position - next_point_world_position).to_angle();

        let progress = ms_timer / ms_to_next_point;

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

        ms_timer += get_frame_time() * 1000.0;

        if ms_timer >= ms_to_next_point {
            ms_timer = 0.0;
            current_point_idx += 1;
            if current_point_idx >= current_rail.points.len() {
                current_point_idx = 0;
                current_rail_idx += 1;
                if current_rail_idx >= world.rails.len() {
                    current_rail_idx = 0;
                }
            }
        }

        next_frame().await
    }
}
