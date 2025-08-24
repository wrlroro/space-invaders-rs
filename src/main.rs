extern crate sdl2;

use sdl2::{
    pixels::Color,
    event::Event,
    keyboard::{
        Scancode,
        Keycode,
    },
    rect::Rect,
    render::TextureQuery,
};

use std::time::{
    Duration,
    Instant,
};

const PIXEL: u32 = 10;
const WINDOW_W: i32 = 800;
const WINDOW_H: i32 = 600;

enum GameState {
    TitleScreen,
    Playing,
}

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
            if y + h > max_y { max_y = y + h; }
        }
    }
    if first { None } else { Some((min_x, max_x, max_y)) }
}

fn wave(alien_1: &Vec<Vec<i32>>, alien_2: &Vec<Vec<i32>>) -> Vec<Alien> {
    let mut aliens = Vec::new();
    aliens.extend(spawner_grid((50, 100), 2, 12, alien_2));
    aliens.extend(spawner_grid((50, 240), 2, 12, alien_1));
    aliens 
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
        Self { x, y, vy: 5, w: PIXEL, h: PIXEL * 2, alive: true }
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

fn text_render(
    text_string: &str,
    center: bool,
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    texture_creator: &sdl2::render::TextureCreator<sdl2::video::WindowContext>,
    font: &sdl2::ttf::Font
) {
    let title_surface = font
        .render(text_string)
        .blended(Color::WHITE)
        .unwrap();
    let title_texture = texture_creator
        .create_texture_from_surface(&title_surface)
        .unwrap();
    let TextureQuery { width: title_w, height: title_h, .. } = title_texture.query();
    let target = if center {
        sdl2::rect::Rect::new(
            (WINDOW_W - title_w as i32) / 2,
            (WINDOW_H - title_h as i32) / 2,
            title_w,
            title_h,
        )
    } else { sdl2::rect::Rect::new(10, 15, title_w, title_h) };

    canvas.copy(&title_texture, None, Some(target)).unwrap();
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

    let mothership_sprite = vec![
        vec![0, 1, 1, 1, 1, 0],
        vec![1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1, 1],
        vec![1, 0, 1, 1, 0, 1],
        vec![1, 0, 0, 0, 0, 1],
    ];

    let mut mothership_cd = Duration::from_millis(5000);
    let mut last_trip = Instant::now();
    let mut mothership = Alien::new(mothership_sprite.clone(), -100, 20); 

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("space-invade-rs", WINDOW_W as u32, WINDOW_H as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut aliens = wave(&alien_1, &alien_2);
    // let mut aliens = Vec::new();
    // aliens.extend(spawner_grid((50, 100), 2, 12, &alien_2.clone()));
    // aliens.extend(spawner_grid((50, 240), 2, 12, &alien_1.clone()));

    // alien fleet
    let mut direction: i32 = 1;
    let mut step_timer = Instant::now();
    let mut step_interval = Duration::from_millis(1200);
    let step = PIXEL as i32;
    let drop = PIXEL as i32 * 2;
    
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut last_shot = Instant::now();
    let fire_cooldown = Duration::from_millis(600);

    let texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font = ttf_context.load_font("assets/PressStart2P-Regular.ttf", 16).unwrap();

    let mut i = 0;
    let mut state = GameState::TitleScreen;

    let mut score: i32 = 0;

    'running: loop {
        i = (i + 1) % 255;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(Keycode::Return), ..} => match state {
                    GameState::TitleScreen => state = GameState::Playing,
                    GameState::Playing => {}
                }

                _ => {}
            }
        }

        match state {
            GameState::TitleScreen => {
                let press_enter: &str = "Press Enter";
                text_render(press_enter, true, &mut canvas, &texture_creator, &font);
                // canvas.copy(&title_texture, None, Some(target)).unwrap();
            }

            GameState::Playing => {
                let total_aliens = aliens.len();

                drawing(&mut canvas, &spaceship, spaceship_x, spaceship_y);

                let score_text = format!("Score: {}", score);
                text_render(&score_text, false, &mut canvas, &texture_creator, &font);

                mothership.draw(&mut canvas);
                for alien in &aliens {
                    alien.draw(&mut canvas);
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

                if last_trip.elapsed() >= mothership_cd {
                    if mothership.alive && mothership.x <= 900 {
                        mothership.translate(5, 0);
                    } else { 
                        if mothership.alive {
                            last_trip = Instant::now();
                            mothership.x = -100;
                        } else {
                            mothership_cd = Duration::from_millis(10000);
                            last_trip = Instant::now();
                            mothership.x = -100;
                            mothership.alive = true;
                        }
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
                    step_interval = Duration::from_millis((400.0 + 800.0 * ratio) as u64);
                }

                for b in &mut bullets { b.update(); }
                for b in &bullets { b.draw(&mut canvas); }

                for b in &mut bullets {
                    if !b.alive { continue; }

                    if mothership.alive && b.rect().has_intersection(mothership.rect()) {
                        mothership.alive = false;
                        b.alive = false;
                        score += 175;
                    }

                    for a in &mut aliens {
                        if !a.alive { continue; }
                        if b.rect().has_intersection(a.rect()) {
                            a.alive = false;
                            b.alive = false;
                            score += 20;
                            break;
                        }
                    }
                }

                if aliens.iter().all(|a| !a.alive) {
                    aliens = wave(&alien_1, &alien_2);

                    direction = 1;
                    step_timer = Instant::now();
                    step_interval = Duration::from_millis(1200);
                }
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
