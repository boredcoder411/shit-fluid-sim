use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::Duration;
use std::time::Instant;

const AIR_DENSITY: f64 = 1.225; // kg/m^3
const GAMMA_AIR: f64 = 1.4; // Adiabatic index for air
const ATMOSPHERIC_PRESSURE: f64 = 101325.0; // Pa
const AIR_ENERGY: f64 = 1000.0; // Initial energy (example)

#[derive(Debug, Clone)]
struct Cell {
    density: f64,
    momentum_x: f64,
    momentum_y: f64,
    energy: f64,
}

fn render_grid(canvas: &mut Canvas<Window>, grid: &Vec<Vec<Cell>>, solid: &Vec<Vec<bool>>) {
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if solid[y][x] {
                canvas.set_draw_color(Color::RGB(100, 100, 100)); // Solid objects as gray
            } else {
                let density_color = (cell.density * 25.5).min(255.0) as u8;
                canvas.set_draw_color(Color::RGB(density_color, density_color, 255));
            }
            let rect = Rect::new((x as i32) * 10, (y as i32) * 10, 10, 10);
            let _ = canvas.fill_rect(rect);
        }
    }
}

fn initialize_grid(width: usize, height: usize) -> Vec<Vec<Cell>> {
    let mut grid = vec![
        vec![
            Cell {
                density: AIR_DENSITY,
                momentum_x: 0.0,
                momentum_y: 0.0,
                energy: AIR_ENERGY,
            };
            width
        ];
        height
    ];

    // Set a high-pressure region in the center for initial movement
    let center_x = width / 2;
    let center_y = height / 2;
    grid[center_y][center_x].density = 10.0;
    grid[center_y][center_x].energy = 1000.0; // High energy in the center

    grid
}

fn calculate_pressure(cell: &Cell, gamma: f64) -> f64 {
    let kinetic_energy = (cell.momentum_x.powi(2) + cell.momentum_y.powi(2)) / (2.0 * cell.density);
    let pressure = (gamma - 1.0) * (cell.energy - kinetic_energy);
    pressure
}

fn calculate_fluxes(grid: &Vec<Vec<Cell>>, solid: &Vec<Vec<bool>>, gamma: f64) -> Vec<Vec<Cell>> {
    let mut new_grid = grid.clone();

    for y in 1..grid.len() - 1 {
        for x in 1..grid[0].len() - 1 {
            if solid[y][x] {
                continue;
            }

            // Calculate pressure at current cell and its neighbors
            let pressure = calculate_pressure(&grid[y][x], gamma);
            let pressure_left = calculate_pressure(&grid[y][x - 1], gamma);
            let pressure_right = calculate_pressure(&grid[y][x + 1], gamma);
            let pressure_up = calculate_pressure(&grid[y - 1][x], gamma);
            let pressure_down = calculate_pressure(&grid[y + 1][x], gamma);

            // Calculate pressure gradients
            let pressure_flux_x = (pressure_right - pressure_left) / 2.0;
            let pressure_flux_y = 0.1 * (pressure_down - pressure_up) / 2.0; // Reduce vertical flux

            // Calculate density flux for stability
            let density_flux_x = (grid[y][x + 1].density - grid[y][x - 1].density) / 2.0;
            let density_flux_y = (grid[y + 1][x].density - grid[y - 1][x].density) / 2.0;

            // Apply pressure flux to momentum to maintain movement
            new_grid[y][x].momentum_x -= pressure_flux_x + density_flux_x * pressure / 2.0;
            new_grid[y][x].momentum_y -= pressure_flux_y + density_flux_y * pressure / 2.0;

            // Calculate average density and energy to update the cell
            let avg_density = (grid[y][x].density
                + grid[y - 1][x].density
                + grid[y + 1][x].density
                + grid[y][x - 1].density
                + grid[y][x + 1].density)
                / 5.0;

            let avg_energy = (grid[y][x].energy
                + grid[y - 1][x].energy
                + grid[y + 1][x].energy
                + grid[y][x - 1].energy
                + grid[y][x + 1].energy)
                / 5.0;

            // Update density and energy based on calculated fluxes
            new_grid[y][x].density = avg_density;
            new_grid[y][x].energy = avg_energy;
        }
    }

    new_grid
}

fn update_grid(grid: &mut Vec<Vec<Cell>>, new_grid: Vec<Vec<Cell>>) {
    *grid = new_grid;
}

fn add_fluid_source(
    grid: &mut Vec<Vec<Cell>>,
    width: usize,
    emission_rate: f64,
    fluid_velocity: f64,
) {
    let height = grid.len();
    let center_y = height / 2;

    for y in center_y - width / 2..center_y + width / 2 {
        grid[y][0].density += emission_rate;
        grid[y][0].momentum_x = fluid_velocity;
        grid[y][0].energy += AIR_ENERGY + 100.0;
    }
}

fn create_solid_object(width: usize, height: usize) -> Vec<Vec<bool>> {
    let mut solid_object = vec![vec![false; width]; height];

    for y in 20..40 {
        for x in 30..50 {
            solid_object[y][x] = true;
        }
    }

    solid_object
}

fn apply_boundary_conditions(grid: &mut Vec<Vec<Cell>>) {
    let width = grid[0].len();
    let height = grid.len();

    // Zero out vertical momentum at left and right boundaries
    for y in 0..height {
        grid[y][0].momentum_y = 0.0; // Left boundary
        grid[y][width - 1].momentum_y = 0.0; // Right boundary

        // Optional: Zero out vertical movement at the boundaries
        grid[y][0].momentum_x = grid[y][1].momentum_x; // Maintain horizontal velocity
        grid[y][width - 1].momentum_x = grid[y][width - 2].momentum_x; // Similar velocity at boundary
    }

    // Apply conditions at the top and bottom boundaries
    for x in 0..width {
        grid[0][x].momentum_y = 0.0; // Top boundary
        grid[height - 1][x].momentum_y = 0.0; // Bottom boundary

        // Optional: Match density and energy at the boundaries to prevent pressure imbalances
        grid[0][x].density = grid[1][x].density; // Match density at top
        grid[height - 1][x].density = grid[height - 2][x].density; // Match density at bottom

        grid[0][x].energy = grid[1][x].energy; // Match energy at top
        grid[height - 1][x].energy = grid[height - 2][x].energy; // Match energy at bottom
    }
}

fn create_airfoil(width: usize, height: usize) -> Vec<Vec<bool>> {
    let mut airfoil = vec![vec![false; width]; height];

    // Create a simple tear-drop airfoil
    for y in 0..height {
        for x in 0..width {
            let x_f = x as f64 / width as f64;
            let y_f = y as f64 / height as f64;
            let distance = ((x_f - 0.5).powi(2) + (y_f - 0.5).powi(2)).sqrt();
            if distance < 0.25 {
                airfoil[y][x] = true;
            }
        }
    }

    airfoil
}

fn main() {
    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Euler Fluid Simulation with Airfoil", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    // Increase the grid size
    let grid_width = 80; // Increased width
    let grid_height = 60; // Increased height
    let mut grid = initialize_grid(grid_width, grid_height);
    //let solid_object = create_airfoil(grid_width, grid_height); // simple oval cause idk how to make a tear drop/foil
    let solid_object = create_solid_object(grid_width, grid_height); // just a block to prove my sim works

    let mut event_pump = sdl_context.event_pump().unwrap();

    // FPS tracking variables
    let mut frame_count = 0;
    let mut last_fps_update = Instant::now();
    let mut last_frame_time = Instant::now();
    let mut fps = 0.0;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        // Clear the screen
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Add a fluid source on the left
        add_fluid_source(&mut grid, 20, 20000.0, 65000.0); // Fluid source

        // Update the fluid grid using the new flux calculations
        let mut new_grid = calculate_fluxes(&grid, &solid_object, GAMMA_AIR);
        apply_boundary_conditions(&mut new_grid);
        update_grid(&mut grid, new_grid);

        // Render the grid and airfoil object
        render_grid(&mut canvas, &grid, &solid_object);

        // Calculate FPS
        let now = Instant::now();
        let frame_time = now.duration_since(last_frame_time);
        last_frame_time = now;

        frame_count += 1;
        if now.duration_since(last_fps_update) >= Duration::from_secs(1) {
            fps = frame_count as f64 / now.duration_since(last_fps_update).as_secs_f64();
            println!("FPS: {:.2}", fps);
            frame_count = 0;
            last_fps_update = now;
        }

        // Present the canvas
        canvas.present();

        // Control frame rate
        std::thread::sleep(Duration::from_millis(1)); // Approx 60 FPS
    }
}
