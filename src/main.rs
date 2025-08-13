extern crate sdl2;

use sdl2::{
    pixels::Color,
    event::Event,
    keyboard::{
        Scancode,
        Keycode,
    },
    rect::Rect,
};

use std::time::{
    Duration,
    Instant,
};

const PIXEL: u32 = 10;
const WINDOW_W: i32 = 800;
const WINDOW_H: i32 = 600;

#[derive(Clone)]
struct Alien {
    sprite: Vec<Vec<i32>>,
    x: i32,
    y: i32,
    // current_health: i32,
    alive: bool,
}

impl Alien {
    // fn new(sprite: Vec<Vec<i32>>, x: i32, y: i32, current_health: i32) -> Self {
    fn new(sprite: Vec<Vec<i32>>, x: i32, y: i32) -> Self {
        Self {
            sprite,
            x,
            y,
            // current_health,
            alive: true,
        }
    }

    // fn take_damage(&mut self, damage: i32) {
    //     if !self.alive { return; }
    //     if self.alive { self.alive = false; }
        // self.current_health = (self.current_health - damage).max(0);
        // if self.current_health == 0 {
        //     self.alive = false;
        // }
    // }
     
    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if !self.alive { return; }
        drawing(canvas, &self.sprite, self.x, self.y)
    }

    fn rect(&self) -> Rect {
        let w = (self.sprite.get(0).map(|r| r.len()).unwrap_or(0) as u32) * PIXEL;
        let h = (self.sprite.len() as u32) * PIXEL;
        Rect::new(self.x, self.y, w, h)
    }

}

fn spawner_grid(
    origin: (i32, i32),
    rows: u32,
    cols: u32,
    sprite: &Vec<Vec<i32>>,
    // health: i32,
) -> Vec<Alien> {
    let (ox, oy) = origin;

    (0..rows)
        .flat_map(|r| {
            (0..cols).map(move |c| {
                Alien::new(
                    sprite.clone(),
                    ox + c as i32 * ((sprite[0].len() + 2) * PIXEL as usize) as i32,
                    oy + r as i32 * ((sprite.len() + 3) * PIXEL as usize) as i32,
                    // health,
                )
            })
        })
        .collect()
}

struct Bullet {
    x: i32,
    y: i32,
    vy: i32,
    w: u32,
    h: u32,
    alive: bool,
}

impl Bullet {
    fn new(x:i32, y: i32) -> Self {
        Self { x, y, vy: 6, w: PIXEL, h: PIXEL * 2, alive: true }
    }

    fn update(&mut self) {
        self.y -= self.vy;
        if self.y + self.h as i32 <= 0 { self.alive = false; }
    }

    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if !self.alive { return; }
        canvas.set_draw_color(Color::WHITE);
        let _ = canvas.fill_rect(Rect::new(self.x, self.y, self.w, self.h));
    }

    fn rect(&self) -> Rect { 
        Rect::new(self.x, self.y, self.w, self.h) 
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

    let spaceship_width = (spaceship[0].len() as i32) * PIXEL as i32;
    let spaceship_y: i32 = WINDOW_H -50;
    let mut spaceship_x: i32 = 100;


    let alien_1 = vec![
        vec![1, 1, 1, 1],
        vec![0, 1, 1, 0],
        vec![1, 0, 0, 1],
        vec![1, 0, 0, 1],
    ];

    let alien_2 = vec![
        vec![1, 1, 1, 1],
        vec![0, 1, 1, 0],
        vec![1, 0, 0, 1],
        vec![0, 1, 1, 0],
    ];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("space-invade-rs", WINDOW_W as u32, WINDOW_H as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut aliens = Vec::new();
    aliens.extend(spawner_grid((50, 100), 2, 12, &alien_2.clone()));
    aliens.extend(spawner_grid((50, 240), 2, 12, &alien_1.clone()));

    let mut bullets: Vec<Bullet> = Vec::new();
    let mut last_shot = Instant::now();
    let fire_cooldown = Duration::from_millis(400);

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        drawing(&mut canvas, &spaceship, spaceship_x, spaceship_y);

        for alien in &aliens {
            alien.draw(&mut canvas);
        }

        
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                
                // Event::KeyDown { keycode: Some(Keycode::A), ..} => {
                //     spaceship_x -= 10;
                //     spaceship_x = spaceship_x.clamp(0, 770);
                // },
                //
                // Event::KeyDown { keycode: Some(Keycode::D), ..} => {
                //     spaceship_x += 10;
                //     spaceship_x = spaceship_x.clamp(0, 770);
                // },
                //
                // // Testing if health works
                // // Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                // //     for alien in aliens.iter_mut() {
                // //         alien.take_damage(1);
                // //     }
                // // }
                //
                // Event::KeyDown { keycode: Some(Keycode::Space), repeat: false, .. } => {
                //     if last_shot.elapsed() >= fire_cooldown {
                //         // spawn from the cannon tip (center column of the 3-wide sprite)
                //         let tip_x = spaceship_x + (spaceship_width / 2) - (PIXEL as i32 / 2);
                //         let tip_y = spaceship_y - PIXEL as i32 * 2;
                //         bullets.push(Bullet::new(tip_x, tip_y));
                //         last_shot = Instant::now();
                //     }
                // }

                _ => {}
            }
        }

        let key_state  = event_pump.keyboard_state();

        if key_state.is_scancode_pressed(Scancode::A) {            
            spaceship_x -= 5;
            spaceship_x = spaceship_x.clamp(0, 770);
        }

        if key_state.is_scancode_pressed(Scancode::D) {
            spaceship_x += 5;
            spaceship_x = spaceship_x.clamp(0, 770);
        }

        if key_state.is_scancode_pressed(Scancode::Space) {
            if last_shot.elapsed() >= fire_cooldown {
                let tip_x = spaceship_x + (spaceship_width / 2) - (PIXEL as i32 / 2);
                let tip_y = spaceship_y - PIXEL as i32 * 2;
                bullets.push(Bullet::new(tip_x, tip_y));
                last_shot = Instant::now();
            }
        }

        for b in &mut bullets { b.update(); }
        for b in &bullets { b.draw(&mut canvas); }

        for b in &mut bullets {
            if !b.alive { continue; }
            for a in &mut aliens {
                if !a.alive { continue; }
                if b.rect().has_intersection(a.rect()) {
                    a.alive = false;
                    b.alive = false;
                    break;
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
