use ledge_engine::interface::*;
use ledge_engine::asset::*;
use ledge_engine::physics::*;
use ledge_engine::scene::level::*;
use ledge_engine::graphics::sprite::SpriteBatch;
use ledge_engine::graphics::image::Image;
use ledge_engine::graphics::BlendMode;
use ledge_engine::graphics::DrawInfo;
use ledge_engine::graphics::Transform;
use ledge_engine::graphics::Rect;
use ledge_engine::game::GameState;
use ledge_engine::ecs::World;
use ledge_engine::event;

use cgmath::Matrix4;

fn main() {
    let mut world = World::new();
    let asset_storage = storage::AssetStorage::<types::Texture>::new();
    world.insert(asset_storage);

    let (mut interface, event_loop) = InterfaceBuilder::new("test", "Dan").build().unwrap();

    // ECS //
    world.register::<SpriteBatch>();
    world.register::<Visible>();
    world.register::<Position>();
    world.register::<DynamicObject>();
    world.register::<RigidBody>();

    // Texture Creation //
    // let sweater_texture_handle;
    let rock_texture_handle;
    {
        // let texture_sweater = types::Texture::from_file_vulkano(include_bytes!("images/SweaterGuy.png"), &interface.graphics_context);
        let texture_rock = types::Texture::from_file_vulkano(include_bytes!("images/rock.png"), &interface.graphics_context);

        let mut texture_assets = world.fetch_mut::<storage::AssetStorage<types::Texture>>();
        // sweater_texture_handle = texture_assets.insert(texture_sweater);
        rock_texture_handle = texture_assets.insert(texture_rock);
    }
    // let sweat_sprite = SpriteBatch::new(sweater_texture_handle.clone());
    let rock_image = Image::new(rock_texture_handle.clone(), 
                                interface.graphics_context.sampler.clone(), 
                                BlendMode::Default,
                                512,
                                512,
                            );
    let mut rock_sprite = SpriteBatch::new(rock_image);
    rock_sprite.load_asset(&world, &mut interface.graphics_context);

    let draw_info = DrawInfo {
        texture_rect: Rect { x: 0.33, y: 0.33, w: 0.33, h: 0.33 },
        color: [0.0, 0.0, 0.0, 1.0],
        transform: Transform::Matrix(Matrix4::new(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)),
    };

    rock_sprite.add(
        draw_info
    );

    // Entity Creation //
    // let rock = world.create_entity().with::<SpriteBatch>(rock_sprite)
    //                                    .is::<Visible>()
    //                                    .with::<Position>(Position { previous_position: (-1.0,-1.0), current_position: (-1.0, -1.0) }).build();
    let player = world.create_entity().with::<SpriteBatch>(rock_sprite.clone())
                                        .is::<Visible>()
                                        .is::<DynamicObject>()
                                        .with::<RigidBody>(RigidBody { 
                                            velocity: (0.0, 0.0), 
                                            previous_velocity: (0.0, 0.0), 
                                            desired_velocity: (0.0, 0.0), 
                                            transition_speed: (20.0, 20.0)
                                        })
                                        .with::<Position>(Position { 
                                            previous_position: (0.0, 0.0), 
                                            current_position: (0.0, 0.0) 
                                        }).build();
                                        
    for i in 0..interface.graphics_context.surface.window.inner_size() / 50 {

    }
    // Level Builder //
    let level_space = LevelSpaceBuilder::new().with_entity(player).build();

    // Game Creation and Running //
    let mut game = GameState::new();

    game.add_space(Box::new(level_space));

    event::run(interface, world, event_loop, game);
}