extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::time::Duration;

const WIDTH: usize = 100; // Grid width
const HEIGHT: usize = 100; // Grid height
const SCALE: usize = 5; // Cell size in pixels
const DIFFUSION: f32 = 0.1; // Diffusion rate
const VISCOSITY: f32 = 0.0001; // Fluid viscosity

// Fluid grid struct
struct Fluid {
    size: usize,
    dt: f32,
    density: Vec<f32>,    // Density of fluid
    velocity_x: Vec<f32>, // Velocity in x direction
    velocity_y: Vec<f32>, // Velocity in y direction
}

impl Fluid {
    fn new(size: usize, dt: f32) -> Self {
        let total_cells = size * size;
        Fluid {
            size,
            dt,
            density: vec![0.0; total_cells],
            velocity_x: vec![0.0; total_cells],
            velocity_y: vec![0.0; total_cells],
        }
    }

    // Adds density to the fluid at position (x, y)
    fn add_density(&mut self, x: usize, y: usize, amount: f32) {
        let idx = self.index(x, y);
        self.density[idx] += amount;
    }

    // Adds velocity to the fluid at position (x, y)
    fn add_velocity(&mut self, x: usize, y: usize, amount_x: f32, amount_y: f32) {
        let idx = self.index(x, y);
        self.velocity_x[idx] += amount_x;
        self.velocity_y[idx] += amount_y;
    }

    // Update the fluid state (advection, diffusion, pressure solve, etc.)
    fn step(&mut self) {
        // Simple advection and diffusion implementation for illustration
        self.diffuse();
        self.advect();
    }

    // Simple diffusion: spreads velocity over neighboring cells
    fn diffuse(&mut self) {
        let visc = self.dt * VISCOSITY;
        let mut new_velocity_x = vec![0.0; self.size * self.size];
        let mut new_velocity_y = vec![0.0; self.size * self.size];

        for y in 1..self.size - 1 {
            for x in 1..self.size - 1 {
                let idx = self.index(x, y);
                let idx_left = self.index(x - 1, y);
                let idx_right = self.index(x + 1, y);
                let idx_up = self.index(x, y - 1);
                let idx_down = self.index(x, y + 1);

                new_velocity_x[idx] = self.velocity_x[idx]
                    + visc
                        * (self.velocity_x[idx_left]
                            + self.velocity_x[idx_right]
                            + self.velocity_x[idx_up]
                            + self.velocity_x[idx_down]
                            - 4.0 * self.velocity_x[idx]);
                new_velocity_y[idx] = self.velocity_y[idx]
                    + visc
                        * (self.velocity_y[idx_left]
                            + self.velocity_y[idx_right]
                            + self.velocity_y[idx_up]
                            + self.velocity_y[idx_down]
                            - 4.0 * self.velocity_y[idx]);
            }
        }

        self.velocity_x = new_velocity_x;
        self.velocity_y = new_velocity_y;
    }

    // Simple advection: move density and velocity based on the velocity field
    fn advect(&self) {
        let dt0 = self.dt * (self.size - 2) as f32;
        let mut new_density = vec![0.0; self.size * self.size];
        let mut new_velocity_x = vec![0.0; self.size * self.size];
        let mut new_velocity_y = vec![0.0; self.size * self.size];

        for y in 1..self.size - 1 {
            for x in 1..self.size - 1 {
                let idx = self.index(x, y);
                let idx_left = self.index(x - 1, y);
                let idx_right = self.index(x + 1, y);
                let idx_up = self.index(x, y - 1);
                let idx_down = self.index(x, y + 1);

                let mut x = (x as f32) - dt0 * self.velocity_x[idx];
                let mut y = (y as f32) - dt0 * self.velocity_y[idx];

                if x < 0.5 {
                    x = 0.5;
                }
                if x > self.size as f32 + 0.5 {
                    x = self.size as f32 + 0.5;
                }
                let i0 = x as usize;
                let i1 = i0 + 1;

                if y < 0.5 {
                    y = 0.5;
                }
                if y > self.size as f32 + 0.5 {
                    y = self.size as f32 + 0.5;
                }
                let j0 = y as usize;
                let j1 = j0 + 1;

                let s1 = x - i0 as f32;
                let s0 = 1.0 - s1;
                let t1 = y - j0 as f32;
                let t0 = 1.0 - t1;

                let idx00 = self.index(i0, j0);
                let idx01 = self.index(i0, j1);
                let idx10 = self.index(i1, j0);
                let idx11 = self.index(i1, j1);

                new_density[idx] = s0 * (t0 * self.density[idx00] + t1 * self.density[idx01])
                    + s1 * (t0 * self.density[idx10] + t1 * self.density[idx11]);
                new_velocity_x[idx] = s0
                    * (t0 * self.velocity_x[idx00] + t1 * self.velocity_x[idx01])
                    + s1 * (t0 * self.velocity_x[idx10] + t1 * self.velocity_x[idx11]);
                new_velocity_y[idx] = s0
                    * (t0 * self.velocity_y[idx00] + t1 * self.velocity_y[idx01])
                    + s1 * (t0 * self.velocity_y[idx10] + t1 * self.velocity_y[idx11]);
            }
        }
    }

    // Converts 2D grid coordinates to a 1D array index
    fn index(&self, x: usize, y: usize) -> usize {
        x + y * self.size
    }
}
// Main function
fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "Euler Fluid Simulation",
            (WIDTH * SCALE) as u32,
            (HEIGHT * SCALE) as u32,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    let mut event_pump = sdl_context.event_pump()?;

    let mut fluid = Fluid::new(WIDTH, 0.1);

    // Main loop
    'running: loop {
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                // Add density and velocity on mouse click
                Event::MouseButtonDown { x, y, .. } => {
                    let fx = x as usize / SCALE;
                    let fy = y as usize / SCALE;
                    fluid.add_density(fx, fy, 100.0);
                    fluid.add_velocity(fx, fy, 10.0, 0.0);
                }
                _ => {}
            }
        }

        // Step the fluid simulation
        fluid.step();

        // Rendering
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw the density as grayscale
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = fluid.index(x, y);
                let density = (fluid.density[idx] * 255.0) as u8;
                canvas.set_draw_color(Color::RGB(density, density, density));
                canvas.fill_rect(Rect::new(
                    (x * SCALE) as i32,
                    (y * SCALE) as i32,
                    SCALE as u32,
                    SCALE as u32,
                ))?;
            }
        }

        canvas.present();

        // Control the frame rate
        ::std::thread::sleep(Duration::from_millis(16));
    }

    Ok(())
}
