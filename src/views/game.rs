use phi::{Phi, View, ViewAction};
use sdl2::pixels::Color;
use phi::data::Rectangle;
use phi::gfx::{CopySprite, Sprite, AnimatedSprite};
use views::shared::BgSet;
use sdl2::render::Renderer;

const PLAYER_SPEED: f64 = 180.0;
const SHIP_W: f64 = 43.0;
const SHIP_H: f64 = 39.0;

const ASTEROID_PATH: &'static str = "assets/asteroid.png";
const ASTEROIDS_WIDE: usize = 21;
const ASTEROIDS_HIGH: usize = 7;
const ASTEROIDS_TOTAL: usize = ASTEROIDS_WIDE * ASTEROIDS_HIGH - 4;
const ASTEROID_SIDE: f64 = 96.0;

const BULLET_SPEED: f64 = 240.0;
const BULLET_W: f64 = 8.0;
const BULLET_H: f64 = 4.0;

const DEBUG: bool = false;

#[derive(Clone, Copy)]
enum ShipFrame {
    UpNorm = 0,
    UpFast = 1,
    UpSlow = 2,
    MidNorm = 3,
    MidFast = 4,
    MidSlow = 5,
    DownNorm = 6,
    DownFast = 7,
    DownSlow = 8
}

struct Ship {
    rect: Rectangle,
    sprites: Vec<Sprite>,
    current: ShipFrame,
    cannon: CannonType
}

impl Ship {
    fn spawn_bullets(&self) -> Vec<Box<Bullet>> {
        let cannons_x = self.rect.x + 30.0;
        let cannon1_y = self.rect.y + 6.0;
        let cannon2_y = self.rect.y + SHIP_H - 10.0;

        match self.cannon {
            CannonType::RectBullet => vec![
                Box::new(RectBullet {
                    rect: Rectangle {
                        x: cannons_x,
                        y: cannon1_y,
                        w: BULLET_W,
                        h: BULLET_H
                    }
                }),
                Box::new(RectBullet {
                    rect: Rectangle {
                        x: cannons_x,
                        y: cannon2_y,
                        w: BULLET_W,
                        h: BULLET_H
                    }
                })
            ],

            CannonType::SineBullet {
                amplitude,
                angular_vel
            } => vec![
                Box::new(SineBullet {
                    pos_x: cannons_x,
                    origin_y: cannon1_y,
                    amplitude: amplitude,
                    angular_vel: angular_vel,
                    total_time: 0.0,
                }),
                Box::new(SineBullet {
                    pos_x: cannons_x,
                    origin_y: cannon2_y,
                    amplitude: amplitude,
                    angular_vel: angular_vel,
                    total_time: 0.0,
                }),
            ],
            CannonType::DivergentBullet { a, b } => vec![
                // If a,b > 0, eventually goes upwards
                Box::new(DivergentBullet {
                    pos_x: cannons_x,
                    origin_y: cannon1_y,
                    a: -a,
                    b: b,
                    total_time: 0.0,
                }),
                // If a,b > 0, eventually goes downwards
                Box::new(DivergentBullet {
                    pos_x: cannons_x,
                    origin_y: cannon2_y,
                    a: a,
                    b: b,
                    total_time: 0.0,
                }),
            ]       
        }

    }
}

pub struct ShipView {
    player: Ship,
    asteroid: Asteroid,
    bullets: Vec<Box<Bullet>>,
    bg: BgSet
}

impl ShipView {
    #[allow(dead_code)]
    pub fn new(phi: &mut Phi) -> ShipView {
        let bg = BgSet::new(&mut phi.renderer);
        ShipView::with_backgrounds(phi, bg)
    }

    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> ShipView {
        let spritesheet = Sprite::load(&mut phi.renderer, "assets/spaceship.png").unwrap();
        let mut sprites = Vec::with_capacity(9);

        for y in 0..3 {
            for x in 0..3 {
                sprites.push(spritesheet.region(Rectangle {
                    w: SHIP_W,
                    h: SHIP_H,
                    x: SHIP_W * x as f64,
                    y: SHIP_H * y as f64,
                }).unwrap());
            }
        }

        ShipView {
            player: Ship {
                rect: Rectangle {
                    x: 64.0,
                    y: 64.0,
                    w: SHIP_W,
                    h: SHIP_H,
                },
                sprites: sprites,
                current: ShipFrame::MidNorm,
                cannon: CannonType::RectBullet
            },

            asteroid: Asteroid::new(phi),

            bullets: vec![],

            bg: bg,
        }
    }

    
}

impl View for ShipView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit {
            return ViewAction::Quit;
        }

        if phi.events.now.key_escape == Some(true) {
            return ViewAction::ChangeView(Box::new(
                ::views::main_menu::MainMenuView::with_backgrounds(
                    phi, self.bg.clone())));
        }

        // Insert the moving logic here
        let diagonal =
        (phi.events.key_up ^ phi.events.key_down) &&
            (phi.events.key_left ^ phi.events.key_right);

        let moved =
        if diagonal { 1.0 / 2.0f64.sqrt() } else { 1.0 } * PLAYER_SPEED * elapsed;

        let dx = match (phi.events.key_left, phi.events.key_right) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        let dy = match (phi.events.key_up, phi.events.key_down) {
            (true, true) | (false, false) => 0.0,
            (true, false) => -moved,
            (false, true) => moved,
        };

        self.player.rect.x += dx;
        self.player.rect.y += dy;

        let movable_region = Rectangle {
            x: 0.0,
            y: 0.0,
            w: phi.output_size().0 * 0.70,
            h: phi.output_size().1,
        };

        // If the player cannot fit in the screen, then there is a problem and
        // the game should be promptly aborted.
        self.player.rect = self.player.rect.move_inside(movable_region).unwrap();

        self.player.current =
            if dx == 0.0 && dy < 0.0 {
                ShipFrame::UpNorm
            } else if dx > 0.0 && dy < 0.0 {
                ShipFrame::UpFast
            } else if dx < 0.0 && dy < 0.0 {
                ShipFrame::UpSlow
            } else if dx == 0.0 && dy == 0.0 {
                ShipFrame::MidNorm
            } else if dx > 0.0 && dy == 0.0 {
                ShipFrame::MidFast
            } else if dx < 0.0 && dy == 0.0 {
                ShipFrame::MidSlow
            } else if dx == 0.0 && dy > 0.0 {
                ShipFrame::DownNorm
            } else if dx > 0.0 && dy > 0.0 {
                ShipFrame::DownFast
            } else if dx < 0.0 && dy > 0.0 {
                ShipFrame::DownSlow
            } else {
                unreachable!()
            };


        self.asteroid.update(phi, elapsed);

        let old_bullets = ::std::mem::replace(&mut self.bullets, vec![]);
        self.bullets = old_bullets.into_iter().filter_map(|bullet| bullet.update(phi, elapsed)).collect();

        if phi.events.now.key_space == Some(true) {
            self.bullets.append(&mut self.player.spawn_bullets());
        }
        // Clear the screen
        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);

        if DEBUG {
            phi.renderer.set_draw_color(Color::RGB(200, 200, 50));
            phi.renderer.fill_rect(self.player.rect.to_sdl().unwrap());
        }

        if phi.events.now.key_1 == Some(true) {
            self.player.cannon = CannonType::RectBullet;
        }

        if phi.events.now.key_2 == Some(true) {
            self.player.cannon = CannonType::SineBullet {
                amplitude: 10.0,
                angular_vel: 15.0
            }
        }

        if phi.events.now.key_3 == Some(true) {
            self.player.cannon = CannonType::DivergentBullet {
                a: 100.0,
                b: 1.2,
            }
        }

        phi.renderer.copy_sprite(
            &self.player.sprites[self.player.current as usize],
            self.player.rect);

        self.asteroid.render(phi);

        for bullet in &self.bullets {
            bullet.render(phi);
        }

        self.bg.front.render(&mut phi.renderer, elapsed);

        ViewAction::None
    }
}

struct Asteroid {
    sprite: AnimatedSprite,
    rect: Rectangle,
    vel: f64
}

impl Asteroid {
    fn new(phi: &mut Phi) -> Asteroid {
        let mut asteroid = Asteroid {
            sprite: Asteroid::get_sprite(phi, 15.0),
            rect: Rectangle {
                w: 0.0,
                h: 0.0,
                x: 0.0,
                y: 0.0,
            },
            vel: 0.0,
        };

        asteroid.reset(phi);
        asteroid
    }

    fn get_sprite(phi: &mut Phi, fps: f64) -> AnimatedSprite {
        let asteroid_spritesheet = Sprite::load(&mut phi.renderer, ASTEROID_PATH).unwrap();
        let mut asteroid_sprites = Vec::with_capacity(ASTEROIDS_TOTAL);

        for yth in 0..ASTEROIDS_HIGH {
            for xth in 0..ASTEROIDS_WIDE {
                //? There are four asteroids missing at the end of the
                //? spritesheet: we do not want to render those.
                if ASTEROIDS_WIDE * yth + xth >= ASTEROIDS_TOTAL {
                    break;
                }

                asteroid_sprites.push(
                    asteroid_spritesheet.region(Rectangle {
                        w: ASTEROID_SIDE,
                        h: ASTEROID_SIDE,
                        x: ASTEROID_SIDE * xth as f64,
                        y: ASTEROID_SIDE * yth as f64,
                    }).unwrap());
            }
        }

        AnimatedSprite::with_fps(asteroid_sprites, fps)
    }


    fn update(&mut self, phi: &mut Phi, dt: f64) {
        self.rect.x -= dt * self.vel;
        self.sprite.add_time(dt);

        if self.rect.x <= -ASTEROID_SIDE {
            self.reset(phi);
        }
    }

    fn render(&mut self, phi: &mut Phi) {
        phi.renderer.copy_sprite(&self.sprite, self.rect);
    }

    fn reset(&mut self, phi: &mut Phi) {
        let (w, h) = phi.output_size();

        self.sprite.set_fps(::rand::random::<f64>().abs() * 20.0 + 10.0);

        self.rect = Rectangle {
            w: ASTEROID_SIDE,
            h: ASTEROID_SIDE,
            x: w,
            y: ::rand::random::<f64>().abs() * (h - ASTEROID_SIDE),
        };

        // vel in [50.0, 150.0)
        self.vel = ::rand::random::<f64>().abs() * 100.0 + 50.0;
    }
}

trait Bullet {
     fn update(self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>>;
     fn render(&self, phi: &mut Phi);
     fn rect(&self) -> Rectangle;
}


#[derive(Clone, Copy)]
struct RectBullet {
    rect: Rectangle
}

impl RectBullet {
    fn update(mut self, phi: &mut Phi, dt: f64) -> Option<Self> {
        let (w, _) = phi.output_size();
        self.rect.x += BULLET_SPEED * dt;

        if self.rect.x > w {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230,230,30));
        phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}


impl Bullet for RectBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        let (w, _) = phi.output_size();
        self.rect.x += BULLET_SPEED * dt;

        if self.rect.x > w {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect.to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        self.rect
    }
}

#[derive(Clone, Copy)]
enum CannonType {
    RectBullet,
    SineBullet {
        amplitude: f64,
        angular_vel: f64
    },
    DivergentBullet {
        a: f64,
        b: f64
    }
}

struct SineBullet {
    pos_x: f64,
    origin_y: f64,
    amplitude: f64,
    angular_vel: f64,
    total_time: f64,
}

impl Bullet for SineBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        //? We store the total time...
        self.total_time += dt;

        //? And move at the same speed as regular bullets.
        self.pos_x += BULLET_SPEED * dt;

        // If the bullet has left the screen, then delete it.
        let (w, _) = phi.output_size();

        if self.rect().x > w {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        // We will render this kind of bullet in yellow.
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        //? Just the general form of the sine function, minus the initial time.
        let dy = self.amplitude * f64::sin(self.angular_vel * self.total_time);
        Rectangle {
            x: self.pos_x,
            y: self.origin_y + dy,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}

struct DivergentBullet {
    pos_x: f64,
    origin_y: f64,
    a: f64, // Influences the bump's height
    b: f64, // Influences the bump's width
    total_time: f64,
}

impl Bullet for DivergentBullet {
    fn update(mut self: Box<Self>, phi: &mut Phi, dt: f64) -> Option<Box<Bullet>> {
        self.total_time += dt;
        self.pos_x += BULLET_SPEED * dt;

        // If the bullet has left the screen, then delete it.
        let (w, h) = phi.output_size();
        let rect = self.rect();

        if rect.x > w || rect.x < 0.0 ||
           rect.y > h || rect.y < 0.0 {
            None
        } else {
            Some(self)
        }
    }

    fn render(&self, phi: &mut Phi) {
        // We will render this kind of bullet in yellow.
        phi.renderer.set_draw_color(Color::RGB(230, 230, 30));
        phi.renderer.fill_rect(self.rect().to_sdl().unwrap());
    }

    fn rect(&self) -> Rectangle {
        let dy = self.a *
                    ((self.total_time / self.b).powi(3) -
                     (self.total_time / self.b).powi(2));

        Rectangle {
            x: self.pos_x,
            y: self.origin_y + dy,
            w: BULLET_W,
            h: BULLET_H,
        }
    }
}