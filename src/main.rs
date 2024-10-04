use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::Duration;

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
            let pressure_flux_y = (pressure_down - pressure_up) / 2.0;

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
        grid[y][0].energy += AIR_ENERGY;
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

fn main() {
    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Euler Fluid Simulation with Solid Objects", 800, 600)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut grid = initialize_grid(80, 60);
    let solid_object = create_solid_object(80, 60);

    let mut event_pump = sdl_context.event_pump().unwrap();

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
        add_fluid_source(&mut grid, 20, 1000.0, 650.0); // Reduced velocity for stability

        // Update the fluid grid using the new flux calculations
        let new_grid = calculate_fluxes(&grid, &solid_object, GAMMA_AIR);
        update_grid(&mut grid, new_grid);

        // Render the grid and solid object
        render_grid(&mut canvas, &grid, &solid_object);

        // Present the canvas
        canvas.present();

        // Control frame rate
        std::thread::sleep(Duration::from_millis(1)); // Approx 60 FPS
    }
}
