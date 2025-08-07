extern crate sdl2;

use sdl2::{
    pixels::Color,
    event::Event,
    keyboard::Keycode,
    rect::Rect,
};

use std::time::Duration;

fn draw_spaceship(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, x: i32, y: i32, pixel_size: u32) {
    let spaceship = [
        [0, 1, 0],
        [1, 1, 1],
        [1, 0, 1],
    ];
    
    canvas.set_draw_color(Color::WHITE);

    for (row_idx, row) in spaceship.iter().enumerate() {
        for (col_idx, &pixel) in row.iter().enumerate() {
            if pixel == 1 {
                let rect = Rect::new(
                    x + (col_idx as i32 * pixel_size as i32),
                    y + (row_idx as i32 * pixel_size as i32),
                    pixel_size,
                    pixel_size,
                );
                let _ = canvas.fill_rect(rect);
            }
        }
    }
}


pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("space-invade-rs", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_spaceship(&mut canvas, 100, 100, 10);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
