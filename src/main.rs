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

#[macroquad::main("MyGame")]
async fn main() {
    let mut world = World { rails: Vec::new() };

    world.rails.push(Rail {
        position: vec2(100.0, 100.0),
        points: vec![
            vec2(0.0, 0.0),
            vec2(50.0, 0.0),
            vec2(100.0, 0.0),
            vec2(150.0, 0.0),
            vec2(200.0, 0.0),
        ],
    });

    world.rails.push(Rail {
        position: vec2(300.0, 100.0),
        points: vec![
            vec2(0.0, 0.0),
            vec2(50.0, 0.0),
            vec2(100.0, 0.0),
            vec2(150.0, 0.0),
            vec2(200.0, 0.0),
        ],
    });

    let mut curved_rail = Rail {
        position: vec2(500.0, 100.0),
        points: vec![],
    };

    let n_points = 8;
    let full_angle = PI / 2.0;
    let angle_step = full_angle / (n_points as f32);
    let angle_bias = -PI / 2.0;

    for i_point in 0..n_points {
        let radius = 180.0;
        let x = radius * (i_point as f32 * angle_step + angle_bias).cos();
        let y = radius * (i_point as f32 * angle_step + angle_bias).sin();

        curved_rail.points.push(vec2(x, y + radius));
    }

    world.rails.push(curved_rail);

    let mut current_point_idx = 0;
    let mut current_rail_idx = 0;

    let mut ms_timer = 0.0;
    let ms_to_next_point = 1000.0;

    loop {
        clear_background(RED);

        for rail in &world.rails {
            for point in &rail.points {
                let point_world_position = rail.position + *point;

                draw_circle(point_world_position.x, point_world_position.y, 5.0, BLUE);
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

        let next_point = current_rail.points[current_point_idx + 1];
        let next_point_world_position = current_rail.position + next_point;

        let next_next_point_world_position = if current_rail.points.len() > current_point_idx + 2 {
            current_rail.points[current_point_idx + 2] + current_rail.position
        } else {
            next_rail.points[0] + next_rail.position
        };

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
            if current_point_idx >= current_rail.points.len() - 1 {
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
