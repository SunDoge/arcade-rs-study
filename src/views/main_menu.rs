use phi::{Phi, View, ViewAction};
use sdl2::pixels::Color;
use phi::gfx::{CopySprite, Sprite};
use phi::data::Rectangle;
use views::shared::BgSet;

// // Load the font from a file
// let font = ::sdl2_ttf::Font::from_file(Path::new(font_path), size).unwrap();

// // For historical reasons, we must first render the text as a `Surface`
// // (a purely 2D image).
// let surface = font.render(text, ::sdl2_ttf::blended(color)).unwrap()

// // Then, we turn this image into a `Texture`, which is more efficient.
// let texture = self.renderer.create_texture_from_surface(&surface).unwrap();

// // Finally, we make a sprite out of this texture.
// let sprite = Sprite::new(texture);

const ACTION_FONT: &'static str = "assets/belligerent.ttf";

struct Action {
    func: Box<Fn(&mut Phi, BgSet) -> ViewAction>,

    idle_sprite: Sprite,

    hover_sprite: Sprite
}

impl Action {
    fn new(phi: &mut Phi, label: &'static str, func: Box<Fn(&mut Phi, BgSet) -> ViewAction>) -> Action {
        Action {
            func: func,
            idle_sprite: phi.ttf_str_sprite(label, ACTION_FONT, 32, Color::RGB(220, 220, 220)).unwrap(),
            hover_sprite: phi.ttf_str_sprite(label, ACTION_FONT, 38, Color::RGB(255, 255, 255)).unwrap()
        }
    }
}

pub struct MainMenuView {
    actions:Vec<Action>,
    selected: i8,
    bg: BgSet
}

impl MainMenuView {
    pub fn new(phi: &mut Phi) -> MainMenuView {
        let bg = BgSet::new(&mut phi.renderer);
        MainMenuView::with_backgrounds(phi, bg)
    }

    pub fn with_backgrounds(phi: &mut Phi, bg: BgSet) -> MainMenuView {
         MainMenuView {
            actions: vec![
                Action::new(phi, "New Game", Box::new(|phi, bg| {
                    ViewAction::ChangeView(Box::new(::views::game::ShipView::with_backgrounds(phi, bg)))
                })),
                Action::new(phi, "Quit", Box::new(|_, _| {
                    ViewAction::Quit
                })),
            ],
            selected: 0,
            bg: bg
        }
    }
}

impl View for MainMenuView {
    fn render(&mut self, phi: &mut Phi, elapsed: f64) -> ViewAction {
        if phi.events.now.quit || phi.events.now.key_escape == Some(true) {
            return ViewAction::Quit;
        }

        if phi.events.now.key_space == Some(true) || phi.events.now.key_enter == Some(true) {
            let bg = self.bg.clone();
            return (self.actions[self.selected as usize].func)(phi, bg);
        }

        // Change the selected action using the keyboard.
        if phi.events.now.key_up == Some(true){
            self.selected -= 1;
            //? If we go past the value at the top of the list, we go 'round
            //? to the bottom.
            if self.selected < 0 {
                self.selected = self.actions.len() as i8 - 1;
            }
        }

        if phi.events.now.key_down == Some(true) {
            self.selected += 1;
            //? If we go past the value at the bottom of the list, we go 'round
            //? to the top.
            if self.selected >= self.actions.len() as i8 {
                self.selected = 0;
            }
        }

        phi.renderer.set_draw_color(Color::RGB(0, 0, 0));
        phi.renderer.clear();

        // Render the labels in the menu
        self.bg.back.render(&mut phi.renderer, elapsed);
        self.bg.middle.render(&mut phi.renderer, elapsed);
        self.bg.front.render(&mut phi.renderer, elapsed);
        
        let (win_w, win_h) = phi.output_size();
        let label_h = 50.0;
        let border_width = 3.0;
        let box_w = 360.0;
        let box_h = self.actions.len() as f64 * label_h;
        let margin_h = 10.0;

        // Render the border of the colored box which holds the labels
        phi.renderer.set_draw_color(Color::RGB(70, 15, 70));
        phi.renderer.fill_rect(Rectangle {
            w: box_w + border_width * 2.0,
            h: box_h + border_width * 2.0 + margin_h * 2.0,
            x: (win_w - box_w) / 2.0 - border_width,
            y: (win_h - box_h) / 2.0 - margin_h - border_width,
        }.to_sdl().unwrap());

        // Render the colored box which holds the labels
        phi.renderer.set_draw_color(Color::RGB(140, 30, 140));
        phi.renderer.fill_rect(Rectangle {
            w: box_w,
            h: box_h + margin_h * 2.0,
            x: (win_w - box_w) / 2.0,
            y: (win_h - box_h) / 2.0 - margin_h,
        }.to_sdl().unwrap());

        // Render the labels in the menu
        for (i, action) in self.actions.iter().enumerate() {
            if self.selected as usize == i {
                let (w, h) = action.hover_sprite.size();
                phi.renderer.copy_sprite(&action.hover_sprite, Rectangle {
                    w: w,
                    h: h,
                    x: (win_w - w) / 2.0,
                    y: (win_h - box_h + label_h - h) / 2.0 + label_h * i as f64,
                });
            } else {
                let (w, h) = action.idle_sprite.size();
                phi.renderer.copy_sprite(&action.idle_sprite, Rectangle {
                    w: w,
                    h: h,
                    x: (win_w - w) / 2.0,
                    y: (win_h - box_h + label_h - h) / 2.0 + label_h * i as f64,
                });
            }
        }

        ViewAction::None
    }
}

