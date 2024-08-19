use minifb::{Key, Window, WindowOptions};
use core::num;
use std::time::{Duration, Instant};
use nalgebra_glm::{Vec2};
use std::f32::consts::PI;
use once_cell::sync::Lazy;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream, Sink, Source};

mod framebuffer;
use framebuffer::Framebuffer;

mod maze;
use maze::load_maze;

mod player;
use player::{process_events, Player};

mod caster;
use caster::{cast_ray, Intersect};

mod texture;
use texture::Texture;

static WALL1: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("./assets/wall1.jpg")));
static WALL2: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("./assets/wall2.jpg")));
static WALL3: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("./assets/wall3.jpg")));
static WALL4: Lazy<Arc<Texture>> = Lazy::new(|| Arc::new(Texture::new("./assets/wall4.jpg")));



fn cell_to_color(cell: char) -> u32 {
    match cell {
        '+' => 0xFF00FF,
        '-' => 0xDD11DD,
        '|' => 0xCC11CC,
        'g' => 0xFF0000,
        _ => 0x000000,
    }
}

fn cell_to_texture_color(cell: char, tx: u32, ty: u32 ) -> u32 {
    match cell {
        '+' => WALL1.get_pixel_color(tx, ty),
        '-' => WALL2.get_pixel_color(tx, ty),
        '|' => WALL3.get_pixel_color(tx, ty),
        'g' => WALL4.get_pixel_color(tx, ty),
        _ => 0x000000,
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            if cell != ' ' {
                let color = cell_to_color(cell);
                framebuffer.set_current_color(color);
                framebuffer.point(x, y);
            }
        }
    }
}

fn render3d(framebuffer: &mut Framebuffer, player: &Player) {
    let maze = load_maze("./map.txt");
    let num_rays = framebuffer.width;
    let block_size = 50;    

    for i in 0..framebuffer.width {
        framebuffer.set_current_color(0x383838);
        for j in 0..(framebuffer.height / 2) {
            framebuffer.point(i, j);
        }
        framebuffer.set_current_color(0x717171);
        for j in (framebuffer.height / 2)..framebuffer.height {
            framebuffer.point(i, j);
        }
    }

    let hh = framebuffer.height as f32 / 2.0;

    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        let interesct = cast_ray(framebuffer, &maze, player, a, block_size, true);

        let stake_height = (framebuffer.height as f32 / interesct.distance) * 70.0;

        let stake_top = (hh - (stake_height / 2.0)) as usize;
        let stake_bottom = (hh + (stake_height / 2.0)) as usize;

        for y in stake_top..stake_bottom {
            let ty = (y as f32 - stake_top as f32) / (stake_bottom as f32 - stake_top as f32) * 230.0;
            let tx = interesct.tx;  
            let color = cell_to_texture_color(interesct.impact, tx as u32, ty as u32);
            framebuffer.set_current_color(color);
            framebuffer.point(i, y);
        }
    }

}

fn render2d(framebuffer: &mut Framebuffer, player: &Player) {
    let maze = load_maze("./map.txt");
    let block_size = 50;

    for row in 0..maze.len() {
        for col in 0..maze[row].len() {
            draw_cell(framebuffer, col * block_size, row * block_size, block_size, maze[row][col])
        }
    }

    framebuffer.set_current_color(0xFFFFFF);
    framebuffer.point(player.pos.x as usize, player.pos.y as usize);

    let num_rays = 100;
    for i in 0..num_rays {
        let current_ray = i as f32 / num_rays as f32;
        let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
        cast_ray(framebuffer, &maze, player, a, block_size, true);
    }
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    let file = File::open("./message_in_a_bottle_taylors_version.mp3").unwrap();
    let source = Decoder::new(BufReader::new(file)).unwrap();

    let source = source.repeat_infinite();

    sink.append(source);

    let window_width = 900;
    let window_height = 450;

    let framebuffer_width = 900;
    let framebuffer_height = 450;

    let frame_delay = Duration::from_millis(0);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);

    let mut window = Window::new(
        "Labreinto ooooOoOO",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_position(100, 100);
    window.update();

    framebuffer.set_background_color(0x333355);
    let mut player = Player {
        pos: Vec2::new(50.0, 50.0),
        a: PI/3.0,
        fov: PI/3.0,
    };
    let mut mode = "2D";
    let mut fps_counter = 0;
    let mut fps = 0;
    let mut last_time = Instant::now();

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            sink.sleep_until_end();
            break;
        }
        if window.is_key_down(Key::M) {
            mode = if mode == "2D" { "3D" } else { "2D" };
        }
        process_events(&window, &mut player);

        framebuffer.clear();

        if mode == "2D" {
            render2d(&mut framebuffer, &player);
        } else {
            render3d(&mut framebuffer, &player);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        let current_time = Instant::now();
        fps_counter += 1;
        if current_time.duration_since(last_time) >= Duration::from_secs(1) {
            fps = fps_counter;
            fps_counter = 0;
            last_time = current_time;
            println!("FPS: {}", fps);
        }

        std::thread::sleep(frame_delay);
    }
}