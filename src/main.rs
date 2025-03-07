use bevy::{
  color::palettes::{basic::SILVER, css::RED},
  prelude::*,
  render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
  },
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      WorldInspectorPlugin::default(),
    ))
    .add_systems(Startup, setup)
    // .add_systems(Update, (rotate,))
    .run();
}

/// A marker component for our shapes so we can query them separately from the ground plane
#[derive(Component)]
struct Shape;

const SHAPES_X_EXTENT: f32 = 14.0;
const EXTRUSION_X_EXTENT: f32 = 16.0;
const Z_EXTENT: f32 = 5.0;

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
) {
  // the grid
  {
    let grid_material = materials.add(StandardMaterial {
      // base_color: RED.into(),
      base_color_texture: Some(images.add(grid_texture())),
      ..default()
    });

    // TODO: de-hardcode
    let size = 12.0;
    let offset = size / 2.0;

    commands.spawn((
      Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
      // MeshMaterial3d(materials.add(Color::from(SILVER))),
      MeshMaterial3d(grid_material),
      Transform::from_xyz(-offset, 0.0, offset),
    ));
  }

  // // cube
  // {
  //   fn place_at(x: u32, y: u32) -> Transform {
  //     // TODO: de-hardcode
  //     Transform::from_xyz(-(y as f32) - 0.5 - 1.0, 0.0, (x as f32) + 0.5 + 1.0)
  //   }

  //   let debug_material = materials.add(StandardMaterial {
  //     base_color_texture: Some(images.add(uv_debug_texture())),
  //     ..default()
  //   });

  //   let mesh = meshes.add(Cuboid::from_size(Vec3::new(1., 1., 1.)));
  //   commands.spawn((
  //     Mesh3d(mesh),
  //     MeshMaterial3d(debug_material.clone()),
  //     place_at(8, 1),
  //     Shape,
  //   ));
  // }

  commands.spawn((
    Camera3d::default(),
    // TODO: de-hardcode (should look at the center of the grid)
    Transform::from_xyz(-6.0, 20., 6.0).looking_at(Vec3::new(-6.0, 0., 6.0), Vec3::X),
  ));

  commands.spawn((
    PointLight {
      shadows_enabled: true,
      intensity: 10_000_000.,
      range: 100.0,
      shadow_depth_bias: 0.2,
      ..default()
    },
    Transform::from_xyz(8.0, 16.0, 8.0),
  ));

  let snake_material = materials.add(StandardMaterial {
    base_color_texture: Some(images.add(uv_debug_texture())),
    ..default()
  });
  let snake_mesh = meshes.add(Cuboid::from_size(Vec3::new(1., 1., 1.)));

  commands.insert_resource(SnakeAssets(snake_material, snake_mesh));

  spawn_snake(&mut commands, 0, 4);
}

fn uv_debug_texture() -> Image {
  const TEXTURE_SIZE: usize = 4;

  #[rustfmt::skip]
    let texture_data: [u8; 64] = [
        255, 255, 255, 255,
        255, 255, 255, 255,
        255, 255, 255, 255,
        255, 255, 255, 255,
        255, 255, 255, 255,
        0, 0, 0, 0,
        0, 0, 0, 0,
        255, 255, 255, 255,
        255, 255, 255, 255,
        0, 0, 0, 0,
        0, 0, 0, 0,
        255, 255, 255, 255,
        255, 255, 255, 255,
        255, 255, 255, 255,
        255, 255, 255, 255,
        255, 255, 255, 255,
    ];

  // for y in 0..TEXTURE_SIZE {
  //     let offset = TEXTURE_SIZE * y * 4;
  //     texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
  //     palette.rotate_right(4);
  // }

  Image::new_fill(
    Extent3d {
      width: TEXTURE_SIZE as u32,
      height: TEXTURE_SIZE as u32,
      depth_or_array_layers: 1,
    },
    TextureDimension::D2,
    &texture_data,
    TextureFormat::Rgba8UnormSrgb,
    RenderAssetUsages::RENDER_WORLD,
  )
}

fn grid_texture() -> Image {
  // how many cells from the bottom to the top and from the left to the right
  const CELLS: usize = 10; // TODO: de-hardcode

  const WIDTH: usize = CELLS
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

fn spawn_snake_part(commands: &mut Commands, snake: &mut Snake, x: u32, y: u32) {
  fn place_at(x: u32, y: u32) -> Transform {
    // TODO: de-hardcode
    Transform::from_xyz(-(y as f32) - 0.5 - 1.0, 0.0, (x as f32) + 0.5 + 1.0)
  }

  commands.queue(move |world: &mut World| {
    let SnakeAssets(material, mesh) = world.resource::<SnakeAssets>().clone();

    let entity = world.spawn((
      Mesh3d(mesh),
      MeshMaterial3d(material),
      place_at(x, y),
      Shape,
    ));
    snake.parts.push(entity.id());
  });
}

fn spawn_snake(commands: &mut Commands, id: u8, len: u32) {
  let mut snake = Snake {
    id,
    parts: vec![],
  };

  spawn_snake_part(commands, &mut snake, len - 1, 0);

  // TODO: use commands.spawn_batch
  for idx in 0..=(len - 2) {
    spawn_snake_part(commands, &mut snake, idx, 0);
  }

  commands.spawn(snake);
}

#[derive(Component)]
struct Snake {
  id: u8,
  parts: Vec<Entity>,
}

// #[derive(Component)]
// struct SnakePart(Snake);

// #[derive(Component)]
// struct SnakeHead(Snake);

#[derive(Resource, Clone)]
struct SnakeAssets(Handle<StandardMaterial>, Handle<Mesh>);
