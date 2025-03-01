use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use colorsys::{Hsl, Rgb};
use rand::Rng;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 128;


fn main() -> Result<(), Error> {
    let mut fire: [[u32;  WIDTH as usize]; HEIGHT as usize] = [[0; WIDTH as usize];  HEIGHT as usize];
    let mut palette: [[u8; 4]; 256] = [[0; 4]; 256];

    for x in 0..256 {
        let hue: i32 = x / 3;
        let saturation: i32 = 255;
        let lightness: i32 = std::cmp::min(255, x * 2); 
        
        let color:Hsl = Hsl::from((hue, saturation, lightness));
        let rgb:Rgb = Rgb::from(color);
        let red: u8 = (rgb.red() * 255.0) as u8;
        let green: u8 = (rgb.green() * 255.0) as u8;
        let blue: u8 = (rgb.blue() * 255.0) as u8;
        
        palette[x as usize] = [red, green, blue, 0xFF];
    }
    
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
        .with_title("Fire")
        .with_inner_size(size)
        .with_min_inner_size(size)
        .build(&event_loop)
        .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let res = event_loop.run(|event, elwt| {
        // Draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {

            let mut rng: rand::prelude::ThreadRng = rand::rng();
        
                for x in 0..WIDTH {
                    fire[(HEIGHT - 1) as usize][x as usize] = (32768 + rng.random_range(0..100)) % 256;
                }
            
                for y in 0..HEIGHT - 1 {
                    for x in 0..WIDTH {                       
                        let left = fire[((y + 1) % HEIGHT) as usize][((x + WIDTH - 1) % WIDTH) as usize];
                        let center = fire[((y + 1) % HEIGHT) as usize][x as usize];
                        let right = fire[((y + 1) % HEIGHT) as usize][((x + 1) % WIDTH) as usize];
                        let below = fire[((y + 2) % HEIGHT) as usize][x as usize];

                        fire[y as usize][x as usize] = (
                            (left + center + right + below)
                            * 32
                        ) / 129;
                    }
                }

            let frame = pixels.frame_mut();
            for (i, pixel) in frame.chunks_exact_mut(4).enumerate() 
            {
                let x = (i % WIDTH as usize) as usize;
                let y = (i / WIDTH as usize) as usize;

                let rgba = palette[fire[y as usize][x as usize] as usize];

                pixel.copy_from_slice(&rgba);
            
            }

            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                elwt.exit();
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}

fn log_error<E: std::error::Error + 'static>(method_name: &str, err: E) {
    error!("{method_name}() failed: {err}");
    for source in err.sources().skip(1) {
        error!("  Caused by: {source}");
    }
}