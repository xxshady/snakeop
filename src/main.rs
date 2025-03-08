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
    .insert_resource(GameTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
    .add_systems(Startup, setup)
    .add_systems(Update, update)
    .run();
}

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

  spawn_snake(&mut commands, 4);
}

#[derive(Resource)]
struct GameTimer(Timer);

fn update(
  time: Res<Time>,
  mut timer: ResMut<GameTimer>,
  mut snakes: Query<&mut Snake>,
  mut commands: Commands,
  mut part_transform: Query<&mut Transform>,
) {
  if timer.0.tick(time.delta()).just_finished() {
    for mut snake in &mut snakes {
      // TODO: other directions and change direction of parts depending on previous ones
      for (entity, direction, x, y) in &mut snake.parts {
        match direction {
          Direction::Right => {
            *x += 1;
          }
          _ => {
            todo!();
          }
        }

        let mut transform = part_transform.get_mut(*entity).unwrap();
        *transform = place_at(*x, *y);
      }
    }
  }
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

fn spawn_snake_part(
  commands: &mut Commands,
  (snake, snake_id): (&mut Snake, Entity),
  x: u32,
  y: u32,
  direction: Direction,
) {
  let part_id = commands.spawn_empty().id();
  snake.parts.push((part_id, direction, x, y));

  commands.queue(move |world: &mut World| {
    // TODO: is it really needed?
    if world.get_entity(snake_id).is_err() {
      error!("failed to spawn snake (id: {snake_id:?}) part");
      world.despawn(part_id);
      return;
    }

    let SnakeAssets(material, mesh) = world.resource::<SnakeAssets>().clone();

    let mut part = world.entity_mut(part_id);
    part.insert((Mesh3d(mesh), MeshMaterial3d(material), place_at(x, y)));
  });
}

fn spawn_snake(commands: &mut Commands, len: u32) {
  let snake_id = commands.spawn_empty().id();
  let mut snake_component = Snake { parts: vec![] };

  spawn_snake_part(
    commands,
    (&mut snake_component, snake_id),
    len - 1,
    0,
    Direction::Right,
  );

  // TODO: use commands.spawn_batch?
  for idx in (0..=(len - 2)).rev() {
    spawn_snake_part(
      commands,
      (&mut snake_component, snake_id),
      idx,
      0,
      Direction::Right,
    );
  }

  let mut snake = commands.entity(snake_id);
  snake.insert(snake_component);
}

fn place_at(x: u32, y: u32) -> Transform {
  // TODO: de-hardcode
  Transform::from_xyz(-(y as f32) - 0.5 - 1.0, 0.0, (x as f32) + 0.5 + 1.0)
}

#[derive(Component)]
struct Snake {
  parts: Vec<(Entity, Direction, u32, u32)>,
}

enum Direction {
  Top,
  Bottom,
  Left,
  Right,
}

#[derive(Resource, Clone)]
struct SnakeAssets(Handle<StandardMaterial>, Handle<Mesh>);
