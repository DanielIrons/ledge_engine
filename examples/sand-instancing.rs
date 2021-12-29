use ledge_engine::event;
use ledge_engine::input;
use ledge_engine::interface::*;
use ledge_engine::graphics::{self, image};
use ledge_engine::error::GameResult;
use rand::{thread_rng, Rng};

#[derive(Clone)]
struct SandPixel {
    image: image::Image,
    updated: bool,
    draw_info: graphics::DrawInfo,
}

impl SandPixel {
    pub fn new(ctx: &graphics::context::GraphicsContext, size: usize, color: graphics::Color) -> Self {
        let mut draw_info = graphics::DrawInfo::new();
        draw_info.scale(1./(size as f32/2.));
        Self {
            image: image::Image::from_color(ctx, color),
            updated: false,
            draw_info,
        }
    }
}

struct MainState {
    particles: Vec<Vec<Option<SandPixel>>>,
    sprite_batch: graphics::sprite::SpriteBatch,
    particle_count: (usize, usize),
}

impl event::EventHandler for MainState {
    fn update(&mut self, interface: &mut Interface) -> GameResult {
        let mut updated = Vec::new();

        if let Some(button) = interface.mouse_context.current_pressed {
            let x = ((1. + interface.mouse_context.last_position.0) * (1./2.) * self.particle_count.0 as f64) as usize;
            let y = ((1. + interface.mouse_context.last_position.1) * (1./2.) * self.particle_count.1 as f64) as usize;
            
            if !(x > self.particle_count.0-1 || y > self.particle_count.1-1) && self.particles[x][y].is_none(){
                if button == input::mouse::MouseButton::Left {
                    self.particles[x][y] = Some(SandPixel::new(&interface.graphics_context, self.particle_count.0, graphics::Color::rgba(194, 168, 128, 255)));
                }
            }
        }

        let mut rng = thread_rng();
        let n: u32 = rng.gen_range(0..10);

        for i in 0..self.particles.len() {
            if i >= self.particles.len() {
                break;
            }
            for j in 0..self.particles[i].len() {
                if j >= self.particles[i].len()-1 || self.particles[i][j].is_none() || updated.contains(&(i, j)) {
                    continue;
                }

                if self.particles[i][j+1].is_none() {
                    self.particles[i][j+1] = self.particles[i][j].take();        
                    updated.push((i,j+1));
                } else if i < self.particle_count.0-1 && self.particles[i+1][j+1].is_none() && n > 6 {
                    self.particles[i+1][j+1] = self.particles[i][j].take();
                    updated.push((i+1,j+1));
                } else if i > 0 && self.particles[i-1][j+1].is_none() && n > 6  {
                    self.particles[i-1][j+1] = self.particles[i][j].take();
                    updated.push((i-1,j+1));
                }
            }
        }

        Ok(())
    }
    
    fn draw(&mut self, interface: &mut Interface) -> GameResult {
        let x = self.particle_count.0 as f32;
        let y = self.particle_count.1 as f32;

        self.sprite_batch = graphics::sprite::SpriteBatch::new(image::Image::from_color(&interface.graphics_context, graphics::Color::rgba(194, 168, 128, 255)));

        for i in 0..self.particles.len() {
            for j in 0..self.particles[i].len() {
                if let Some(pixel) = &mut self.particles[i][j] {
                    pixel.draw_info.dest((i as f32 - x/2.) / (x/2.), ((j+1) as f32 - y/2.) / (y/2.), 0.);
                    self.sprite_batch.add(pixel.draw_info.clone());
                }
            }
        }

        graphics::draw(&mut interface.graphics_context, &self.sprite_batch, graphics::DrawInfo::default());

        Ok(())
    }
}

impl MainState {
    pub fn new(ctx: &graphics::context::GraphicsContext, x: usize, y: usize) -> Self {
        let mut v = Vec::new();
        for _ in 0..x {
            let mut nv: Vec<Option<SandPixel>> = Vec::new();
            nv.resize(y as usize, None);
            v.push(nv);
        }

        Self{
            particles: v,
            sprite_batch: graphics::sprite::SpriteBatch::new(image::Image::from_color(ctx, graphics::Color::rgba(194, 168, 128, 255))),
            particle_count: (x, y),
        }
    }
}

fn main() {
    let builder = InterfaceBuilder::new("sand", "author");
    let (interface, event_loop) = builder.build().unwrap();
    let state = MainState::new(&interface.graphics_context, 128, 128);
    event::run(interface, event_loop, state);
}