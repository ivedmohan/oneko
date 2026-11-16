#![cfg_attr(windows, windows_subsystem = "windows")]

mod cat;
mod platform;
mod sprites;
mod tray;

use std::time::{Duration, Instant};

use cat::Cat;
use rand::rngs::StdRng;
use sprites::Character;
use rand::SeedableRng;
use tray::{Tray, TrayAction};
use winit::application::ApplicationHandler;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId, WindowLevel};

const TICK: Duration = Duration::from_millis(100);

struct App {
    /// User-facing size multiplier (1 = classic 32px cat).
    scale: f64,
    character: Character,
    frames: Vec<image::RgbaImage>,
    /// Sprite size in backing pixels, fixed once the window exists.
    size_px: u32,
    window: Option<Window>,
    renderer: Option<platform::Renderer>,
    cat: Option<Cat>,
    tray: Option<Tray>,
    rng: StdRng,
    next_tick: Instant,
    paused: bool,
}

impl App {
    fn create_window(&mut self, el: &ActiveEventLoop) {
        let monitor = el.primary_monitor().or_else(|| el.available_monitors().next());
        let scale_factor = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);
        let upp = platform::units_per_pixel(scale_factor);
        let bounds = monitor
            .as_ref()
            .map(|m| (m.size().width as f64 * upp, m.size().height as f64 * upp))
            .unwrap_or((1280.0, 800.0));

        // Cat size in screen units, and in backing pixels for the sprite frames.
        let size_units = sprites::CELL as f64 * self.scale * platform::units_per_logical(scale_factor);
        let size_px = (size_units / upp).round() as u32;

        let cat = Cat::new(size_units, size_units, size_units, bounds);

        let mut attrs = Window::default_attributes()
            .with_title("oneko")
            .with_decorations(false)
            .with_resizable(false)
            .with_transparent(platform::WINIT_TRANSPARENT)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_inner_size(platform::size(size_units))
            .with_position(platform::position(cat.x - size_units / 2.0, cat.y - size_units / 2.0));
        #[cfg(target_os = "macos")]
        {
            use winit::platform::macos::WindowAttributesExtMacOS;
            attrs = attrs.with_has_shadow(false).with_accepts_first_mouse(false);
        }
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowAttributesExtWindows;
            attrs = attrs.with_skip_taskbar(true);
        }

        let window = el.create_window(attrs).expect("create cat window");
        if let Err(e) = window.set_cursor_hittest(false) {
            eprintln!("warning: could not make window click-through: {e}");
        }

        self.size_px = size_px;
        let scaled = sprites::scaled(&self.frames, size_px);
        let mut renderer = platform::Renderer::new(&window, &scaled);
        renderer.show(cat.sprite().0.frame_index(0));

        self.renderer = Some(renderer);
        self.cat = Some(cat);
        self.window = Some(window);
    }

    fn set_character(&mut self, character: Character) {
        if character == self.character {
            return;
        }
        self.character = character;
        self.frames = sprites::load_frames(character);
        if let Some(renderer) = &mut self.renderer {
            renderer.set_frames(&sprites::scaled(&self.frames, self.size_px));
            if let Some(cat) = &self.cat {
                let (sprite, frame) = cat.sprite();
                renderer.show(sprite.frame_index(frame));
            }
        }
        if let Some(tray) = &self.tray {
            let (rgba, w, h) = sprites::tray_icon_rgba(&self.frames, cfg!(target_os = "macos"));
            tray.set_icon(rgba, w, h);
        }
    }

    fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if let Some(window) = &self.window {
            window.set_visible(!paused);
        }
    }

    fn tick(&mut self) {
        let (Some(cat), Some(window), Some(renderer)) = (&mut self.cat, &self.window, &mut self.renderer) else {
            return;
        };
        if self.paused {
            return;
        }
        cat.tick(platform::cursor_position(), &mut self.rng);
        window.set_outer_position(platform::position(cat.x - cat.size / 2.0, cat.y - cat.size / 2.0));
        let (sprite, frame) = cat.sprite();
        renderer.show(sprite.frame_index(frame));
    }
}

impl ApplicationHandler for App {
    fn new_events(&mut self, _el: &ActiveEventLoop, cause: StartCause) {
        if cause == StartCause::Init && self.tray.is_none() {
            let (rgba, w, h) = sprites::tray_icon_rgba(&self.frames, cfg!(target_os = "macos"));
            match Tray::new(self.character, rgba, w, h) {
                Ok(t) => self.tray = Some(t),
                Err(e) => eprintln!("warning: no tray icon: {e}"),
            }
        }
    }

    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_none() {
            self.create_window(el);
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if event == WindowEvent::CloseRequested {
            el.exit();
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if let Some(action) = self.tray.as_ref().and_then(Tray::poll) {
            match action {
                TrayAction::TogglePause => self.set_paused(!self.paused),
                TrayAction::SetCharacter(c) => self.set_character(c),
                TrayAction::Quit => el.exit(),
            }
        }
        let now = Instant::now();
        if now >= self.next_tick {
            self.tick();
            // Step from the scheduled time so we don't drift, but never schedule into the past.
            self.next_tick += TICK;
            if self.next_tick < now {
                self.next_tick = now + TICK;
            }
        }
        el.set_control_flow(ControlFlow::WaitUntil(self.next_tick));
    }
}

fn main() {
    let scale = std::env::args()
        .skip_while(|a| a != "--scale")
        .nth(1)
        .and_then(|v| v.parse::<f64>().ok())
        .filter(|v| *v > 0.0 && *v <= 8.0)
        .unwrap_or(1.0);

    let character = std::env::args()
        .skip_while(|a| a != "--character")
        .nth(1)
        .and_then(|v| Character::from_name(&v.to_lowercase()))
        .unwrap_or(Character::Neko);

    let mut builder = EventLoop::builder();
    #[cfg(target_os = "macos")]
    {
        use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
        builder.with_activation_policy(ActivationPolicy::Accessory);
    }
    let event_loop = builder.build().expect("event loop");

    let mut app = App {
        scale,
        character,
        frames: sprites::load_frames(character),
        size_px: sprites::CELL,
        window: None,
        renderer: None,
        cat: None,
        tray: None,
        rng: StdRng::from_entropy(),
        next_tick: Instant::now(),
        paused: false,
    };
    event_loop.run_app(&mut app).expect("run");
}
