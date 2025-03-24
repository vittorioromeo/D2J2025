mod preset;

use std::f32::consts::PI;

use macroquad::{audio::*, prelude::*};
use preset::*;

type Angle = f32;

const CRT_FRAGMENT_SHADER: &'static str = r#"#version 100
precision lowp float;

varying vec4 color;
varying vec2 uv;

uniform sampler2D Texture;
uniform float iTime;

// https://www.shadertoy.com/view/XtlSD7

vec2 CRTCurveUV(vec2 uv)
{
    uv = uv * 2.0 - 1.0;
    vec2 offset = abs( uv.yx ) / vec2( 6.0, 4.0 );
    uv = uv + uv * offset * offset;
    uv = uv * 0.5 + 0.5;
    return uv;
}

void DrawVignette( inout vec3 color, vec2 uv )
{
    float vignette = uv.x * uv.y * ( 1.0 - uv.x ) * ( 1.0 - uv.y );
    vignette = clamp( pow( 8.0 * vignette, 1.0 ), 0.0, 1.0 );
    color *= vignette * 1.2;
}


void DrawScanline( inout vec3 color, vec2 uv )
{
    float scanline 	= clamp( 0.95 + 0.55 * cos( 3.14 * ( uv.y + 0.008 * mod(iTime, 1000.0) ) * 240.0 * 1.0 ), 0.0, 1.0 );
    float grille 	= 0.85 + 0.15 * clamp( 1.5 * cos( 3.14 * uv.x * 640.0 * 1.0 ), 0.0, 1.0 );
    color *= scanline * grille * 1.2;
}

void main() {
    vec2 crtUV = CRTCurveUV(uv);
    vec3 res = texture2D(Texture, uv).rgb * color.rgb;
    if (crtUV.x < 0.0 || crtUV.x > 1.0 || crtUV.y < 0.0 || crtUV.y > 1.0)
    {
        res = vec3(0.0, 0.0, 0.0);
    }
    DrawVignette(res, crtUV);
    DrawScanline(res, uv);
    gl_FragColor = vec4(res, 1.0) * 2.2;

}
"#;

const CRT_VERTEX_SHADER: &'static str = "#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;

varying lowp vec2 uv;
varying lowp vec4 color;

uniform mat4 Model;
uniform mat4 Projection;

uniform float iTime;

void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
";

struct Rail {
    position: Vec2,
    points: Vec<Vec2>,
    is_wall: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Letter {
    A,
    B,
    C,
}

struct Fork {
    rail1: Rail,
    rail2: Rail,
    which: ForkSelection,
    letter: Letter,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ForkSelection {
    Rail1,
    Rail2,
}

impl ForkSelection {
    fn toggle(&self) -> Self {
        match self {
            ForkSelection::Rail1 => ForkSelection::Rail2,
            ForkSelection::Rail2 => ForkSelection::Rail1,
        }
    }
}

enum Block {
    Rail(Rail),
    Fork(Fork),
}

impl Block {
    fn points(&self) -> &[Vec2] {
        match self {
            Block::Rail(rail) => &rail.points,
            Block::Fork(fork) => fork.points(),
        }
    }

    fn position(&self) -> Vec2 {
        match self {
            Block::Rail(rail) => rail.position,
            Block::Fork(fork) => fork.position(),
        }
    }
}

pub struct World {
    rails: Vec<Block>,
}

impl Fork {
    fn last_angle(&self) -> Angle {
        if !self.rail1.is_wall {
            self.rail1.last_angle()
        } else {
            self.rail2.last_angle()
        }
    }

    fn points(&self) -> &[Vec2] {
        match self.which {
            ForkSelection::Rail1 => &self.rail1.points,
            ForkSelection::Rail2 => &self.rail2.points,
        }
    }

    fn position(&self) -> Vec2 {
        match self.which {
            ForkSelection::Rail1 => self.rail1.position,
            ForkSelection::Rail2 => self.rail2.position,
        }
    }
}

impl World {
    fn find_next_fork_index(&self, starting_idx: usize, letter: Letter) -> Option<usize> {
        for i in starting_idx..self.rails.len() {
            if let Block::Fork(f) = &self.rails[i] {
                if f.letter == letter {
                    return Some(i);
                }
            }
        }

        None
    }
}

struct State {
    current_rail_idx: usize,
    current_point_idx: usize,
    ms_timer: f32,
    speedup_timer: f32,
    alive: bool,
}

impl State {
    fn new() -> Self {
        Self {
            current_rail_idx: 0,
            current_point_idx: 0,
            ms_timer: 0.0,
            speedup_timer: 0.0,
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
            Block::Fork(fork) => match fork.which {
                ForkSelection::Rail1 => {
                    fork.rail1.position + fork.rail1.points[self.current_point_idx]
                }
                ForkSelection::Rail2 => {
                    fork.rail2.position + fork.rail2.points[self.current_point_idx]
                }
            },
        }
    }

    fn get_current_rail_points<'a>(&self, world: &'a World) -> &'a [Vec2] {
        self.get_current_rail(world).points()
    }

    fn get_next_point_world_position(&self, world: &World) -> Vec2 {
        let current_rail = self.get_current_rail(world);
        let next_rail = self.get_next_rail(world);

        let current_rail_points = current_rail.points();

        let current_rail_position = current_rail.position();

        let next_rail_points = next_rail.points();

        let next_rail_position = next_rail.position();

        if current_rail_points.len() > self.current_point_idx + 1 {
            current_rail_points[self.current_point_idx + 1] + current_rail_position
        } else {
            next_rail_points[0] + next_rail_position
        }
    }

    fn get_next_next_point_world_position(&self, world: &World) -> Vec2 {
        let current_rail = self.get_current_rail(world);
        let next_rail = self.get_next_rail(world);

        let current_rail_points = current_rail.points();

        let current_rail_position = current_rail.position();

        let next_rail_points = next_rail.points();

        let next_rail_position = next_rail.position();

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
        let is_wall = match current_rail {
            Block::Rail(rail) => rail.is_wall,
            Block::Fork(fork) => match fork.which {
                ForkSelection::Rail1 => fork.rail1.is_wall,
                ForkSelection::Rail2 => fork.rail2.is_wall,
            },
        };
        if is_wall && self.current_point_idx >= current_len - 1 {
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
        start_angle: f32,
        is_wall: bool,
        dist: f32,
        n_points: usize,
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
            position,
            points: circle_points,
            is_wall,
        }
    }

    fn new_straight(
        position: Vec2,
        start_angle: f32,
        is_wall: bool,
        dist: f32,
        n_points: usize,
    ) -> Self {
        Self::new_curved(position, start_angle, is_wall, dist, n_points, 0.0)
    }

    fn last_angle(&self) -> Angle {
        let p1 = self.points.get(self.points.len() - 2).unwrap();
        let p2 = self.points.last().unwrap();

        // Calculate direction vector between points
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;

        // Calculate angle using arctangent (atan2 handles all quadrants correctly)
        dy.atan2(dx)
    }
}

pub fn get_last_rail_world_position(world: &World) -> Vec2 {
    if world.rails.is_empty() {
        return vec2(0.0, 0.0);
    }

    let last_rail = world.rails.last().unwrap();

    match last_rail {
        Block::Rail(rail) => {
            let last_point = rail.points.last().unwrap();
            rail.position + *last_point
        }
        Block::Fork(fork) => {
            if !fork.rail1.is_wall {
                let last_point = fork.rail1.points.last().unwrap();
                fork.rail1.position + *last_point
            } else {
                let last_point = fork.rail2.points.last().unwrap();
                fork.rail2.position + *last_point
            }
        }
    }
}

pub fn get_last_rail_world_start_angle(world: &World) -> Angle {
    if world.rails.is_empty() {
        return 0.0;
    }

    let last_rail = world.rails.last().unwrap();

    match last_rail {
        Block::Rail(rail) => rail.last_angle(),
        Block::Fork(fork) => fork.last_angle(),
    }
}

fn draw_rail(
    state: &State,
    idx: usize,
    position: Vec2,
    points: &[Vec2],
    rail_color: Color,
    is_wall: bool,
    is_fork: bool,
) {
    let mut rail_color = rail_color;

    for (i, point) in points.iter().enumerate() {
        let point_world_position = position + *point;

        let mut color = if i == 0 {
            if is_fork {
                YELLOW // First point is yellow
            } else {
                WHITE // First point is white
            }
        } else if i == points.len() - 1 && is_wall {
            RED // Last point is red
        } else {
            BLUE // Middle points are blue
        };

        let idx_dist = (state.current_rail_idx as i32 - idx as i32).abs();
        let alpha = (0.1 * (10 - idx_dist) as f32).clamp(0.0, 1.0);

        color.a = alpha;
        rail_color.a = alpha;

        draw_circle(point_world_position.x, point_world_position.y, 5.0, color);
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

fn draw_rail_text(
    font: &Font,
    state: &State,
    idx: usize,
    position: Vec2,
    points: &[Vec2],
    is_fork: bool,
    letter: Option<Letter>,
) {
    for (i, point) in points.iter().enumerate() {
        let point_world_position = position + *point;

        let idx_dist = (state.current_rail_idx as i32 - idx as i32).abs();
        let alpha = (0.1 * (10 - idx_dist) as f32).clamp(0.0, 1.0);

        if letter.is_some() && is_fork && i == 0 {
            let str = match letter.unwrap() {
                Letter::A => "A",
                Letter::B => "B",
                Letter::C => "C",
            };

            draw_text_ex(
                str,
                point_world_position.x + 0.0,
                point_world_position.y - 25.0,
                TextParams {
                    font: Some(font),
                    font_size: 45 as u16,
                    font_scale: 1.0,
                    color: Color::new(1.00, 1.00, 1.00, alpha),
                    ..Default::default()
                },
            );
        }
    }
}

/// Draws a texture at the given position with a uniform scale and rotation (in radians).
///
/// # Arguments
/// * `texture` - A reference to the Texture to be drawn.
/// * `position` - The top-left position where the texture will be drawn.
/// * `scale` - A uniform scale factor to apply to the texture.
/// * `rotation` - The rotation in radians.
pub fn draw_texture_helper(texture: &Texture2D, position: Vec2, scale: f32, rotation: f32) {
    let dest_size = Some(Vec2::new(texture.width() * scale, texture.height() * scale));

    draw_texture_ex(
        texture,
        position.x,
        position.y,
        WHITE,
        DrawTextureParams {
            dest_size,
            // Set pivot to the center of the scaled texture
            pivot: Some(Vec2::new(
                texture.width() * scale / 2.0,
                texture.height() * scale / 2.0,
            )),
            rotation,
            ..Default::default()
        },
    );
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

fn window_conf() -> Conf {
    Conf {
        window_title: "Railway Panic".to_owned(),
        fullscreen: false,
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf())]
async fn main() {
    let crt_render_target = render_target(1280, 720);

    let texture = load_texture("overlay.png").await.unwrap();
    let font = load_ttf_font("BebasNeue-Regular.ttf").await.unwrap();
    let sound_horn = load_sound("horn.ogg").await.unwrap();
    let sound_switch = load_sound("switch.ogg").await.unwrap();
    let sound_step = load_sound("step.ogg").await.unwrap();
    let lever0_texture = load_texture("lever0.png").await.unwrap();
    let lever1_texture = load_texture("lever1.png").await.unwrap();
    let player_texture = load_texture("player.png").await.unwrap();

    let mut world = World { rails: Vec::new() };

    let crt_material = load_material(
        ShaderSource::Glsl {
            vertex: CRT_VERTEX_SHADER,
            fragment: CRT_FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![UniformDesc::new("iTime", UniformType::Float1)],
            ..Default::default()
        },
    )
    .unwrap();

    preset_0_straight(&mut world);
    preset_0_straight(&mut world);
    preset_0_straight(&mut world);
    preset_0_straight(&mut world);

    let mut state = State::new();
    let mut ms_to_next_point = 100.0;

    let mut camera_pos = vec2(0.0, 0.0);

    let mut selected_index = 0;
    let mut lever_state: [bool; 3] = [false; 3];

    loop {
        clear_background(BLACK);

        let current_point_world_position = state.get_current_point_world_position(&world);
        let next_point_world_position = state.get_next_point_world_position(&world);
        let next_next_point_world_position = state.get_next_next_point_world_position(&world);

        if current_point_world_position == next_point_world_position {
            state.move_to_next_point(&world);
            continue;
        }

        let rotation0 = (next_point_world_position - current_point_world_position).to_angle();
        let rotation1 = (next_next_point_world_position - next_point_world_position).to_angle();

        let progress = if state.alive {
            state.ms_timer / ms_to_next_point
        } else {
            1.0
        };

        let train_position = current_point_world_position.lerp(next_point_world_position, progress);
        let train_rotation = interpolate_angle(rotation0, rotation1, progress);

        let train_width = 80.0;
        let train_height = 40.0;

        state.ms_timer += get_frame_time() * 1000.0;
        state.speedup_timer += get_frame_time() * 1000.0;

        if state.alive && state.ms_timer >= ms_to_next_point {
            state.ms_timer = 0.0;
            state.move_to_next_point(&world);
        }

        if state.alive && state.speedup_timer >= 10000.0 && ms_to_next_point > 30.0 {
            ms_to_next_point -= 10.0;
            state.speedup_timer = 0.0;

            play_sound(&sound_horn, PlaySoundParams::default());
        }

        crt_material.set_uniform("iTime", state.ms_timer);

        camera_pos = exponential_approach_vec2(camera_pos, train_position, 5.0, get_frame_time());

        while world.rails.len() < 25 {
            preset::preset_random(&mut world);
        }

        if state.current_rail_idx >= world.rails.len() - 24 {
            preset::preset_random(&mut world);
        }

        while world.rails.len() > 100 {
            world.rails.remove(0);
            state.current_rail_idx -= 1;
        }

        if is_key_pressed(KeyCode::Left) {
            if selected_index == 0 {
                selected_index = 2;
            } else {
                selected_index -= 1;
            }

            play_sound(&sound_step, PlaySoundParams::default());
        } else if is_key_pressed(KeyCode::Right) {
            selected_index += 1;
            if selected_index > 2 {
                selected_index = 0;
            }
            play_sound(&sound_step, PlaySoundParams::default());
        }

        if is_key_pressed(KeyCode::Space) {
            let selected_letter = match selected_index % 3 {
                0 => Letter::A,
                1 => Letter::B,
                2 => Letter::C,
                _ => unreachable!(),
            };

            lever_state[selected_index % 3] = !lever_state[selected_index % 3];

            if let Some(fork_idx) =
                world.find_next_fork_index(state.current_rail_idx + 1, selected_letter)
            {
                if let Block::Fork(fork) = &mut world.rails[fork_idx] {
                    fork.which = fork.which.toggle();
                }
            }

            play_sound(&sound_switch, PlaySoundParams::default());
        }

        let zoom_level = 0.0019 * ms_to_next_point.remap(100.0, 30.0, 1.0, 0.75);

        let aspect_ratio = screen_width() / screen_height();
        macroquad::camera::set_camera(&Camera2D {
            zoom: vec2(zoom_level / aspect_ratio, zoom_level),
            target: camera_pos + vec2(0.0, 100.0),
            render_target: Some(crt_render_target.clone()),
            ..Default::default()
        });

        clear_background(Color::new(0.1, 0.1, 0.1, 1.0));

        for (i, block) in world.rails.iter().enumerate() {
            match block {
                Block::Rail(rail) => {
                    draw_rail(
                        &state,
                        i,
                        rail.position,
                        &rail.points,
                        BLUE,
                        rail.is_wall,
                        false,
                    );
                }
                Block::Fork(fork) => {
                    let color1 = if fork.which == ForkSelection::Rail1 {
                        BLUE
                    } else {
                        GRAY.with_alpha(0.75)
                    };

                    let color2 = if fork.which == ForkSelection::Rail2 {
                        BLUE
                    } else {
                        GRAY.with_alpha(0.75)
                    };

                    draw_rail(
                        &state,
                        i,
                        fork.rail1.position,
                        &fork.rail1.points,
                        color1,
                        fork.rail1.is_wall,
                        true,
                    );

                    draw_rail(
                        &state,
                        i,
                        fork.rail2.position,
                        &fork.rail2.points,
                        color2,
                        fork.rail2.is_wall,
                        true,
                    );
                }
            }
        }

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

        for (i, block) in world.rails.iter().enumerate() {
            match block {
                Block::Rail(rail) => {
                    draw_rail_text(&font, &state, i, rail.position, &rail.points, false, None);
                }
                Block::Fork(fork) => {
                    draw_rail_text(
                        &font,
                        &state,
                        i,
                        fork.rail1.position,
                        &fork.rail1.points,
                        true,
                        Some(fork.letter),
                    );

                    draw_rail_text(
                        &font,
                        &state,
                        i,
                        fork.rail2.position,
                        &fork.rail2.points,
                        true,
                        Some(fork.letter),
                    );
                }
            }
        }

        macroquad::camera::set_camera(&Camera2D {
            zoom: vec2(0.0025 / aspect_ratio, 0.0025) * 1.15,
            target: vec2(screen_width() / 2.0, screen_height() / 2.0),
            render_target: None,
            ..Default::default()
        });

        gl_use_material(&crt_material);
        draw_texture_ex(
            &crt_render_target.texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        gl_use_default_material();

        draw_texture(&texture, 0.0, 0.0, WHITE);

        for i in 0..3 {
            let position = vec2(110.0 + i as f32 * 400.0, 500.0);
            let texture = if lever_state[i] {
                &lever0_texture
            } else {
                &lever1_texture
            };

            if selected_index % 3 == i {
                let x_offset = if lever_state[i] { -60.0 } else { 60.0 };

                draw_texture_helper(
                    &player_texture,
                    position + vec2(80.0 + x_offset, -100.0),
                    0.75,
                    0.0,
                );
            }

            draw_texture_helper(texture, position, 0.75, 0.0);

            let str = match i {
                0 => "A",
                1 => "B",
                2 => "C",
                _ => unreachable!(),
            };

            draw_text_ex(
                str,
                position.x + 136.0,
                position.y + 194.0,
                TextParams {
                    font: Some(&font),
                    font_size: 37 as u16,
                    font_scale: 1.0,
                    color: Color::new(0.00, 0.00, 0.00, 1.00),
                    ..Default::default()
                },
            );
        }

        next_frame().await
    }
}

// TODO:
// - survival timer that stops on death
// - show current speed
// - release exe
// - game over and restart
// - cleanup code
// - credits: Vittorio Romeo, Marco Ieni, Sonia Misericordia
