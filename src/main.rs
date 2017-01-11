extern crate sdl2;

mod phi;
mod views;

//#[macro_use]
//mod events;

use sdl2::pixels::Color;
use phi::Events;

//pub enum ViewAction {
//    None,
//    Quit,
//}

//pub struct Phi<'window> {
//    pub events: Events,
//    pub renderer: Renderer<'window>,
//}
//
//pub trait View {
//    fn render(&mut self, context: &mut Phi, elapsed: f64) -> ViewAction;
//}
//
//
//struct_events! {
//    keyboard: {
//        key_escape: Escape,
//        key_up: Up,
//        key_down: Down
//    },
//    else: {
//        quit: Quit { .. }
//    }
//}


fn main() {
    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();
    let video = sdl_context.video().unwrap();

    // Create the window
    let window = video.window("ArcadeRS Shooter", 800, 600)
        .position_centered().opengl()
        .build().unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build().unwrap();

    // Prepare the events record
    let mut events = Events::new(sdl_context.event_pump().unwrap());


    loop {
        events.pump();

        if events.now.quit || events.now.key_escape == Some(true) {
            break;
        }

        renderer.set_draw_color(Color::RGB(0, 0, 0));
        renderer.clear();
        renderer.present();
    }
}