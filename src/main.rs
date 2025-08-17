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
    alive: bool,
}

impl Alien {
    fn new(sprite: Vec<Vec<i32>>, x: i32, y: i32) -> Self {
        Self {
            sprite,
            x,
            y,
            alive: true,
        }
    }

    fn w(&self) -> i32 {
        (self.sprite.get(0).map(|r| r.len()).unwrap_or(0) as i32) * PIXEL as i32
    }

    fn h(&self) -> i32 {
        (self.sprite.len() as i32) * PIXEL as i32
    }

    fn translate(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

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
) -> Vec<Alien> {
    let (ox, oy) = origin;

    (0..rows)
        .flat_map(|r| {
            (0..cols).map(move |c| {
                Alien::new(
                    sprite.clone(),
                    ox + c as i32 * ((sprite[0].len() + 2) * PIXEL as usize) as i32,
                    oy + r as i32 * ((sprite.len() + 3) * PIXEL as usize) as i32,
                )
            })
        })
        .collect()
}

fn fleet_manager(aliens: &[Alien]) -> Option<(i32, i32, i32)> {
    let mut min_x = 0;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut first = true;

    for a in aliens.iter().filter(|a| a.alive) {
        let (x, y, w, h) = (a.x, a.y, a.w(), a.h());
        if first {
            min_x = x;
            max_x = x + w;
            max_y = y + h;
            first = false;
        } else {
            if x < min_x { min_x = x; }
            if x + w > max_x { max_x = x + w; }
            if y > max_y { max_y = y; }
        }
    }
    Some((min_x, max_x, max_y))
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

    // alien fleet
    let mut direction: i32 = 1;
    let mut step_timer = Instant::now();
    let mut step_interval = Duration::from_millis(600);
    let step = PIXEL as i32;
    let drop = PIXEL as i32 * 2;
    
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut last_shot = Instant::now();
    let fire_cooldown = Duration::from_millis(400);

    let mut i = 0;
    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        let total_aliens = aliens.len();

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

        if step_timer.elapsed() >= step_interval {
            let mut descend = false;

            if let Some((min_x, max_x, _max_y)) = fleet_manager(&aliens) {
                let left_limit = 10;
                let right_limit = WINDOW_W - 10;

                if direction > 0 && max_x + step >= right_limit {
                    direction = -1;
                    descend = true;
                }
                if direction < 0 && min_x - step <= left_limit {
                    direction = 1;
                    descend = true;
                }
            }

            let dx = direction * step;
            let dy = if descend { drop } else { 0 };

            for a in aliens.iter_mut().filter(|a| a.alive) {
                a.translate(if descend { 0 } else { dx }, dy);
            }
            
            step_timer = Instant::now();
            let alive_aliens = aliens.iter().filter(|a| a.alive).count().max(1);
            let ratio = alive_aliens as f32 / total_aliens as f32; // 1.0 .. 0.0
            let step_interval = Duration::from_millis((150.0 + 450.0 * ratio) as u64);
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
