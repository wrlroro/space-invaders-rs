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

use std::{
    fs,
    rc::Rc,
    time::{
        Duration,
        Instant,
    }
};

const PIXEL: u32 = 5;
const WINDOW_W: i32 = 800;
const WINDOW_H: i32 = 600;
const HIGHSCORE_PATH: &str = "assets/highscore.txt";

enum GameState {
    TitleScreen,
    Playing,
    Pause,
    Lost,
}

#[derive(Clone)]
struct Player {
    sprite: Vec<Vec<i32>>,
    x: i32,
    y: i32,
    lives: i32,
}

impl Player {
    fn new(sprite: Vec<Vec<i32>>, x: i32, y: i32, lives: i32) -> Self {
        Self {
            sprite,
            x,
            y,
            lives,
        }
    }
    fn rect(&self) -> Rect {
        let w = (self.sprite.get(0).map(|r| r.len()).unwrap_or(0) as u32) * PIXEL;
        let h = (self.sprite.len() as u32) * PIXEL;
        Rect::new(self.x, self.y, w, h)
    }
}

#[derive(Clone)]
struct Alien {
    // sprite: Vec<Vec<i32>>,
    frames: Rc<Vec<Vec<Vec<i32>>>>,
    frame_ix: usize,
    frame_interval: Duration,
    last_frame: Instant,

    x: i32,
    y: i32,
    alive: bool,
}

impl Alien {
    fn new(frames: Rc<Vec<Vec<Vec<i32>>>>, x: i32, y: i32) -> Self {
        Self {
            // sprite,
            frames,
            frame_ix: 0,
            frame_interval: Duration::from_millis(800),
            last_frame: Instant::now(),
            x,
            y,
            alive: true,
        }
    }

    fn current_sprite(&self) -> &Vec<Vec<i32>> {
        &self.frames[self.frame_ix]
    }

    fn update_animation(&mut self) {
        if self.last_frame.elapsed() >= self.frame_interval {
            self.frame_ix = (self.frame_ix + 1) % self.frames.len();
            self.last_frame = Instant::now();
        }
    }

    fn w(&self) -> i32 {
        (self.current_sprite().get(0).map(|r| r.len()).unwrap_or(0) as i32) * PIXEL as i32
    }

    fn h(&self) -> i32 {
        (self.current_sprite().len() as i32) * PIXEL as i32
    }

    fn translate(&mut self, dx: i32, dy: i32) {
        self.x += dx;
        self.y += dy;
    }

    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if !self.alive { return; }
        drawing(canvas, &self.current_sprite(), self.x, self.y)
    }

    fn rect(&self) -> Rect {
        let w = (self.current_sprite().get(0).map(|r| r.len()).unwrap_or(0) as u32) * PIXEL;
        let h = (self.current_sprite().len() as u32) * PIXEL;
        Rect::new(self.x, self.y, w, h)
    }
}

fn spawner_grid(
    origin: (i32, i32),
    rows: u32,
    cols: u32,
    // sprite: &Vec<Vec<i32>>,
    frames: Rc<Vec<Vec<Vec<i32>>>>,
) -> Vec<Alien> {
    let (ox, oy) = origin;

    let first = &frames[0];
    let sprite_h = first.len() as i32;
    let sprite_w = first.get(0).map(|r| r.len()).unwrap_or(0) as i32;

    let cell_w = (sprite_w + 4) * PIXEL as i32; 
    let cell_h = (sprite_h + 6) * PIXEL as i32;

    (0..rows)
        .flat_map(|r| {
            let f = frames.clone();
            (0..cols).map(move |c| {
                Alien::new(
                    f.clone(),
                    ox + c as i32 * cell_w, 
                    oy + r as i32 * cell_h, 
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

fn wave(
    alien_1_a: &Vec<Vec<i32>>, 
    alien_1_b: &Vec<Vec<i32>>, 
    alien_2_a: &Vec<Vec<i32>>, 
    alien_2_b: &Vec<Vec<i32>>
) -> Vec<Alien> {
    let mut aliens = Vec::new();
    let cols: i32 = 12;
    let origin_x = WINDOW_W / cols;

    let alien_1_frames = Rc::new(vec![alien_1_a.clone(), alien_1_b.clone()]);
    let alien_2_frames = Rc::new(vec![alien_2_a.clone(), alien_2_b.clone()]);

    aliens.extend(spawner_grid((origin_x, WINDOW_H * 2 / 10), 1, cols as u32, alien_1_frames));
    aliens.extend(spawner_grid((origin_x, WINDOW_H * 3 / 10), 3, cols as u32, alien_2_frames));
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
    fn new(x:i32, y: i32, vy: i32) -> Self {
        Self { x, y, vy, w: PIXEL, h: PIXEL * 2, alive: true }
    }

    fn update(&mut self) {
        self.y += self.vy;
        if self.y + self.h as i32 <= 0 || self.y >= WINDOW_H { 
            self.alive = false; 
        }
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

fn overlap_x(a: &Alien, b: &Alien) -> bool {
    let ax0 = a.x;
    let ax1 = a.x + a.w();
    let bx0 = b.x;
    let bx1 = b.x + b.w();
    ax0 < bx1 && bx0 < ax1
}

fn is_bottommost(a: &Alien, aliens: &[Alien]) -> bool {
    if !a.alive { return false; }
    !aliens.iter().any(|other| other.alive && overlap_x(a, other) && other.y > a.y)
}

fn bottom_shooters(aliens: &[Alien]) -> Vec<usize> {
    aliens.iter().enumerate()
        .filter(|(_, a)| is_bottommost(a, aliens))
        .map(|(i, _)| i)
        .collect()
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

enum Position {
    Center,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

fn text_render(
    text_string: &str,
    position: Position,
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
    let target = match position {
        Position::Center => sdl2::rect::Rect::new(
            (WINDOW_W - title_w as i32) / 2,
            (WINDOW_H - title_h as i32) / 2,
            title_w,
            title_h,
        ),
        Position::TopLeft => sdl2::rect::Rect::new(10, 15, title_w, title_h), 
        Position::TopRight => sdl2::rect::Rect::new(WINDOW_W - 10 - title_w as i32, 15, title_w, title_h),
        Position::BottomLeft => sdl2::rect::Rect::new(10, WINDOW_H * 4 / 5, title_w, title_h),
        Position::BottomRight => sdl2::rect::Rect::new(WINDOW_W - 10 - title_w as i32, WINDOW_H * 4 / 5, title_w, title_h),
    };

    canvas.copy(&title_texture, None, Some(target)).unwrap();
}

fn load_highscore() -> i32 {
    fs::read_to_string(HIGHSCORE_PATH)
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
        .unwrap_or(0)
}

fn save_highscore(score: i32) {
    let _ = fs::write(HIGHSCORE_PATH, score.to_string());
}

pub fn main() {
    let mut high_score: i32 = load_highscore();

    let spaceship = vec![
        vec![0, 0, 1, 0, 0],
        vec![0, 1, 1, 1, 0],
        vec![1, 1, 1, 1, 1],
        vec![1, 1, 0, 1, 1],
        vec![1, 0, 0, 0, 1],
    ];
    
    let mut player = Player::new(spaceship.clone(), WINDOW_W / 2, WINDOW_H - 50, 3);
    let spaceship_width = (spaceship[0].len() as i32) * PIXEL as i32;
    // let spaceship_y: i32 = WINDOW_H -50;
    // let mut spaceship_x: i32 = WINDOW_W / 2;
 
    let hearts = vec![
        vec![0, 1, 0, 1, 0],
        vec![1, 1, 1, 1, 1],
        vec![1, 1, 1, 1, 1],
        vec![0, 1, 1, 1, 0],
        vec![0, 0, 1, 0 ,0],
    ];

    let h_w = (hearts[0].len() as i32) * PIXEL as i32;
    let h_h = (hearts.len() as i32) * PIXEL as i32;

    let alien_1_a = vec![
        vec![1, 1, 0, 0, 0, 1, 1],
        vec![0, 1, 1, 1, 1, 1, 0],
        vec![0, 1, 0, 0, 0, 1, 0],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1],
        vec![0, 1, 0, 0, 0, 1, 0],
    ];

    let alien_1_b = vec![
        vec![0, 1, 0, 0, 0, 1, 0],
        vec![1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 0, 0, 0, 1, 1],
        vec![0, 1, 1, 0, 1, 1, 0],
        vec![0, 1, 0, 1, 0, 1, 0],
        vec![1, 0, 0, 1, 0, 0, 1],
    ];

    let alien_2_a = vec![
        vec![1, 1, 0, 1, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 1, 0, 1, 1, 1],
        vec![0, 1, 0, 0, 0, 1, 0],
        vec![0, 1, 0, 0, 0, 1, 0],
        vec![0, 0, 1, 0, 1, 0, 0],
    ];

    let alien_2_b = vec![
        vec![1, 1, 0, 1, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 1, 1, 1, 0, 1],
        vec![0, 1, 0, 1, 0, 1, 0],
        vec![0, 1, 0, 0, 0, 1, 0],
        vec![0, 1, 1, 0, 1, 1, 0],
    ];

    let mothership_sprite = vec![
        vec![0, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0],
        vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 1, 0, 1, 0, 1, 1, 0, 1, 0, 1, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 1, 0],
        vec![0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0],
    ];

    let mut mothership_cd = Duration::from_millis(5000);
    let mut last_trip = Instant::now();
    let mut mothership = Alien::new(Rc::new(vec![mothership_sprite.clone()]), -100, 20); 

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("space-invade-rs", WINDOW_W as u32, WINDOW_H as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut aliens = wave(&alien_1_a, &alien_1_b, &alien_2_a, &alien_2_b);

    let mut direction: i32 = 1;
    let mut step_timer = Instant::now();
    let mut step_interval = Duration::from_millis(1200);
    let step = PIXEL as i32;
    let drop = PIXEL as i32 * 2;

    let mut player_bullet: Vec<Bullet> = Vec::new();

    let mut enemy_bullet: Vec<Bullet> = Vec::new();
    let mut enemy_fire_timer = Instant::now();
    let mut enemy_fire_interval = Duration::from_millis(900);

    let texture_creator = canvas.texture_creator();
    let ttf_context = sdl2::ttf::init().unwrap();
    let font_big = ttf_context.load_font("assets/PressStart2P-Regular.ttf", 32).unwrap();
    let font_small = ttf_context.load_font("assets/PressStart2P-Regular.ttf", 16).unwrap();

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
                    GameState::Playing => {},
                    GameState::Pause => state = GameState::TitleScreen,
                    GameState::Lost => state = GameState::TitleScreen,
                }
                Event::KeyDown { keycode: Some(Keycode::P), ..} => match state {
                    GameState::Playing => state = GameState::Pause,
                    GameState::Pause => state = GameState::Playing,
                    GameState::TitleScreen => {},
                    GameState::Lost => {},
                }

                _ => {}
            }
        }

        match state {
            GameState::TitleScreen => {
                let press_enter: &str = "Press Enter";
                text_render(press_enter, Position::Center, &mut canvas, &texture_creator, &font_big);
                let high_text = format!("High Score: {}", high_score);
                text_render(&high_text, Position::BottomLeft, &mut canvas, &texture_creator, &font_small);
                score = 0;
                aliens = wave(&alien_1_a, &alien_1_b, &alien_2_a, &alien_2_b);
                mothership = Alien::new(Rc::new(vec![mothership_sprite.clone()]), -100, 20); 
                last_trip = Instant::now();
                player_bullet.clear();
                enemy_bullet.clear();
                enemy_fire_timer = Instant::now();
                enemy_fire_interval = Duration::from_millis(900);
                direction = 1;
                // spaceship_x = WINDOW_W / 2;
                player.x = WINDOW_W / 2;
                player.lives = 3;
            }

            GameState::Playing => {
                let total_aliens = aliens.len();

                drawing(&mut canvas, &player.sprite, player.x, player.y);

                let score_text = format!("Score: {}", score);
                text_render(&score_text, Position::TopLeft, &mut canvas, &texture_creator, &font_small);
                // let exit_text: &str = "Escape to exit";
                // text_render(exit_text, Position::TopRight, &mut canvas, &texture_creator, &font_small);

                let mut h_x = WINDOW_W - h_w - 10;
                let h_y = 15;
                for _ in 0..player.lives {
                    drawing(&mut canvas, &hearts, h_x, h_y);
                    h_x -= h_w + 5;
                }

                mothership.draw(&mut canvas);

                for a in aliens.iter_mut() {
                    a.update_animation();
                }

                for alien in &aliens {
                    alien.draw(&mut canvas);
                }

                let key_state  = event_pump.keyboard_state();

                if key_state.is_scancode_pressed(Scancode::A) {            
                    player.x -= 5;
                    player.x = player.x.max(0);
                }

                if key_state.is_scancode_pressed(Scancode::D) {
                    player.x += 5;
                    player.x = player.x.min(WINDOW_W - spaceship_width);
                }

                if key_state.is_scancode_pressed(Scancode::Space) {
                    if player_bullet.is_empty() {
                        let tip_x = player.x + (spaceship_width / 2) - (PIXEL as i32 / 2);
                        let tip_y = player.y - PIXEL as i32 * 2;
                        player_bullet.push(Bullet::new(tip_x, tip_y, -6));
                    }
                }

                if last_trip.elapsed() >= mothership_cd {
                    if mothership.alive && mothership.x <= WINDOW_W + mothership.w() {
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
                    step_interval = Duration::from_millis((200.0 + 400.0 * ratio) as u64);
                }

                for b in player_bullet.iter_mut() {
                    b.update();

                    if mothership.alive && b.rect().has_intersection(mothership.rect()) {
                        mothership.alive = false;
                        b.alive = false;
                        score += 175;
                        if score > high_score {
                            high_score = score;
                            save_highscore(high_score);
                        }
                    }

                    if b.alive { 
                        for a in &mut aliens {
                            if !a.alive { continue; }
                            if b.rect().has_intersection(a.rect()) {
                                a.alive = false;
                                b.alive = false;
                                score += 20;
                                if score > high_score {
                                    high_score = score;
                                    save_highscore(high_score);
                                } 
                                break;
                            }
                        }
                    }

                    b.draw(&mut canvas);
                }

                player_bullet.retain(|b| b.alive);

                if enemy_fire_timer.elapsed() >= enemy_fire_interval {
                    let shooters = bottom_shooters(&aliens);
                    if !shooters.is_empty() {
                        let idx = shooters[(i as usize) % shooters.len()];
                        let a = &aliens[idx];
                        let bx = a.x + (a.w() / 2) - (PIXEL as i32 / 2);
                        let by = a.y + a.h();
                        enemy_bullet.push(Bullet::new(bx, by, 5));
                    }
                    enemy_fire_timer = Instant::now();
                }

                for eb in enemy_bullet.iter_mut() {
                    eb.update();

                    if eb.rect().has_intersection(player.rect()) {
                        player.lives -= 1;
                        eb.alive = false;
                        if player.lives < 1 {
                            state = GameState::Lost;
                        }
                        break;
                    }

                    eb.draw(&mut canvas);
                }
                enemy_bullet.retain(|b| b.alive);

                if aliens.iter().all(|a| !a.alive) {
                    aliens = wave(&alien_1_a, &alien_1_b, &alien_2_a, &alien_2_b);
                    direction = 1;
                    step_timer = Instant::now();
                    step_interval = Duration::from_millis(1200);
                }
            }

            GameState::Pause => {
                let game_paused: &str = "Game Paused";
                let p_continue: &str = "P to continue";
                let enter_title: &str = "Enter to go to title";
                text_render(game_paused, Position::Center, &mut canvas, &texture_creator, &font_big);
                text_render(p_continue, Position::BottomLeft, &mut canvas, &texture_creator, &font_small);
                text_render(enter_title, Position::BottomRight, &mut canvas, &texture_creator, &font_small);
            }

            GameState::Lost => {
                if score > high_score {
                    high_score = score;
                    save_highscore(high_score);
                }
                let game_lost: &str = "You Lost!";
                let enter_title: &str = "Enter to go to title";
                let exit_text: &str = "Esc to exit";
                text_render(game_lost, Position::Center, &mut canvas, &texture_creator, &font_big);
                text_render(enter_title, Position::BottomLeft, &mut canvas, &texture_creator, &font_small);
                text_render(exit_text, Position::BottomRight, &mut canvas, &texture_creator, &font_small);
            }
        }

        canvas.present();
        ::std::thread::sleep(Duration::from_millis(1000 / 60));
    }
}
