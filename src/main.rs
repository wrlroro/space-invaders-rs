extern crate sdl2;

use sdl2::{
    pixels::Color,
    event::Event,
    keyboard::Keycode,
    rect::Rect,
};

use std::time::Duration;

const PIXEL: u32 = 10;

fn drawing(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, design: &Vec<Vec<i32>>, x: i32, y: i32) {
    canvas.set_draw_color(Color::WHITE);

    for (row_idx, row) in design.iter().enumerate() {
        for (col_idx, &pixel) in row.iter().enumerate() {
            if pixel == 1 {
                let rect = Rect::new(
                    x + (col_idx as i32 * PIXEL as i32),
                    y + (row_idx as i32 * PIXEL as i32),
                    PIXEL,
                    PIXEL,
                );
                let _ = canvas.fill_rect(rect);
            }
        }
    }
}


pub fn main() {
    let spaceship = vec![
        vec![0, 1, 0],
        vec![1, 1, 1],
        vec![1, 0, 1],
    ];

    let alien_1 = vec![
        vec![1, 1, 1],
        vec![0, 1, 0],
        vec![1, 0, 1],
    ];

    let alien_2 = vec![
        vec![1, 1, 1, 1],
        vec![0, 1, 1, 0],
        vec![0, 1, 1, 0],
    ];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("space-invade-rs", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    
    let mut pos_x = 100;

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        drawing(&mut canvas, &spaceship, pos_x, 550);
        drawing(&mut canvas, &alien_1, 400, 100); 
        drawing(&mut canvas, &alien_2, 200, 100);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },

                Event::KeyDown { keycode: Some(Keycode::A), ..} => {
                    pos_x -= 10;
                    pos_x = pos_x.clamp(0, 770);
                },

                Event::KeyDown { keycode: Some(Keycode::D), ..} => {
                    pos_x += 10;
                    pos_x = pos_x.clamp(0, 770);
                },

                _ => {}
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
