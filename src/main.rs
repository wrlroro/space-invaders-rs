extern crate sdl2;

use sdl2::{
    pixels::Color,
    event::Event,
    keyboard::Keycode,
    rect::Rect,
};

use std::time::Duration;

const PIXEL: u32 = 10;

#[derive(Clone)]
struct Alien {
    sprite: Vec<Vec<i32>>,
    x: i32,
    y: i32,
    current_health: i32,
    alive: bool,
}

impl Alien {
    fn new(sprite: Vec<Vec<i32>>, x: i32, y: i32, current_health: i32) -> Self {
        Self {
            sprite,
            x,
            y,
            current_health,
            alive: true,
        }
    }

    fn take_damage(&mut self, damage: i32) {
        if !self.alive { return; }
        self.current_health = (self.current_health - damage).max(0);
        if self.current_health == 0 {
            self.alive = false;
        }
    }
     
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if !self.alive { return; }
        drawing(canvas, &self.sprite, self.x, self.y)
    }

}

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

    let mut aliens = vec![
        Alien::new(alien_1.clone(), 100, 200, 3),
        Alien::new(alien_1.clone(), 200, 200, 3),
        Alien::new(alien_1.clone(), 300, 200, 3),
        Alien::new(alien_1.clone(), 400, 200, 3),
        Alien::new(alien_1.clone(), 500, 200, 3),
        Alien::new(alien_2.clone(), 150, 100, 5),
        Alien::new(alien_2.clone(), 300, 100, 5),
        Alien::new(alien_2.clone(), 450, 100, 5),
    ];

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        drawing(&mut canvas, &spaceship, pos_x, 550);

        for alien in &aliens {
            alien.draw(&mut canvas);
        }

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

                // Testing if health works
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    for alien in aliens.iter_mut() {
                        alien.take_damage(1);
                    }
                }

                _ => {}
            }
        }
        canvas.present();
        ::std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
