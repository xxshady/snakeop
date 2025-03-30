use bevy::{
  prelude::*,
  render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
  },
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

// how many cells from the bottom to the top and from the left to the right
const CELLS: u32 = 20;

mod fk;
mod game;

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      WorldInspectorPlugin::default(),
    ))
    .add_systems(Startup, |world: &mut World| {
      let return_world = fk::take_world(world);
      game::setup();
      return_world(world);
    })
    .add_systems(Update, |world: &mut World| {
      let return_world = fk::take_world(world);
      game::update();
      return_world(world);
    })
    .run();
}

// TODO: should it also be rewritten?
pub fn grid_texture() -> Image {
  const WIDTH: usize = CELLS as usize
    // left and right borders
    + 2
    // // borders between cells 
    // + (CELLS - 1)
    ;

  const TEXTURE_SIZE: usize =
    // width
    WIDTH
      // height
      * WIDTH
      // rgba
      * 4;

  let mut texture_data = [0; TEXTURE_SIZE];

  const WHITE_CELL: [u8; 4] = [255, 255, 255, 255];

  let wall: Vec<u8> = WHITE_CELL.repeat(WIDTH);

  let cells_start = WIDTH * 4;
  let cells_end = TEXTURE_SIZE - (WIDTH * 4);

  // top wall
  texture_data[0..cells_start].copy_from_slice(&wall);
  // bottom wall
  texture_data[cells_end..].copy_from_slice(&wall);

  // left and right walls
  for idx in (cells_start..(cells_end + 4)).step_by(WIDTH * 4) {
    texture_data[idx..(idx + 4)].copy_from_slice(&WHITE_CELL);
    texture_data[(idx - 4)..((idx - 4) + 4)].copy_from_slice(&WHITE_CELL);

    // --------------- inner walls ---------------
    //  (enable borders between cells in WIDTH if its needed)
    // let should_be_a_wall = (idx / (WIDTH * 4)) % 2 == 0;

    // for rel_idx in (0..WIDTH * 4).step_by(4) {
    //     if should_be_a_wall || (rel_idx / 4) % 2 == 0 {
    //         texture_data[(idx + rel_idx)..(idx + rel_idx) + 4].copy_from_slice(&WHITE_CELL);
    //     }
    // }
  }

  Image::new_fill(
    Extent3d {
      width: WIDTH as u32,
      height: WIDTH as u32,
      depth_or_array_layers: 1,
    },
    TextureDimension::D2,
    &texture_data,
    TextureFormat::Rgba8UnormSrgb,
    RenderAssetUsages::RENDER_WORLD,
  )
}
