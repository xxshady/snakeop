use bevy::{
  audio::PlaybackMode,
  color::palettes::css::RED,
  prelude::*,
  render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
  },
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use rand::Rng;

// how many cells from the bottom to the top and from the left to the right
const CELLS: u32 = 20;

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      WorldInspectorPlugin::default(),
    ))
    .insert_resource(Time::<Fixed>::from_seconds(0.2))
    .insert_resource(OccupiedCells::default())
    .add_systems(Startup, setup)
    .add_systems(Update, (control_snake,))
    .add_systems(
      FixedUpdate,
      (
        process_snake_movement,
        process_snake_food,
        animate_food,
        // debug_render_occupied_cells,
      )
        .chain(),
    )
    .run();
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,
  mut occupied_cells: ResMut<OccupiedCells>,
) {
  // the grid
  let grid_material = materials.add(StandardMaterial {
    // base_color: RED.into(),
    base_color_texture: Some(images.add(grid_texture())),
    ..default()
  });

  let size = (CELLS + 2) as f32;
  let offset = size / 2.0;

  commands.spawn((
    Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
    // MeshMaterial3d(materials.add(Color::from(SILVER))),
    MeshMaterial3d(grid_material),
    Transform::from_xyz(-offset, 0.0, offset),
  ));

  commands.spawn((
    Camera3d::default(),
    Transform::from_xyz(-offset + (-offset / 2.), size * 1.5, offset - (offset / 2.))
      .looking_at(Vec3::new(-offset, 0., offset), Vec3::X),
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

  spawn_snake(&mut commands, &mut occupied_cells, 3);

  let food_sound_asset = asset_server.load("sounds/smb_coin.wav");
  commands.insert_resource(FoodSoundAsset(food_sound_asset));

  let food_material = materials.add(StandardMaterial {
    base_color: RED.into(),
    ..default()
  });
  let food_mesh = meshes.add(Cuboid::from_size(Vec3::new(1., 1., 1.)));

  commands.insert_resource(FoodAssets(food_material, food_mesh));

  spawn_food(&mut commands, &mut occupied_cells);
}

fn process_snake_movement(
  mut snakes: Query<(Entity, &mut Snake)>,
  mut commands: Commands,
  mut part_transform: Query<&mut Transform>,
  mut occupied_cells: ResMut<OccupiedCells>,
) {
  for (snake_entity, mut snake) in &mut snakes {
    let mut next_positions = vec![];

    {
      let head = &snake.parts[0];
      let (x, y) = (head.pos.x, head.pos.y);

      let (next_x, next_y) = match head.direction {
        Direction::Top => (x, y - 1),
        Direction::Bottom => (x, y + 1),
        Direction::Left => (x - 1, y),
        Direction::Right => (x + 1, y),
      };
      let next_pos = Pos {
        x: next_x,
        y: next_y,
      };

      next_positions.push((head.entity, next_pos, None));
    }

    for window in snake.parts.windows(2) {
      let [prev, current] = window else {
        unreachable!();
      };

      let (next_pos, next_direction) = if current.direction == prev.direction {
        let (x, y) = (current.pos.x, current.pos.y);
        let (next_x, next_y) = match current.direction {
          Direction::Top => (x, y - 1),
          Direction::Bottom => (x, y + 1),
          Direction::Left => (x - 1, y),
          Direction::Right => (x + 1, y),
        };

        (
          Pos {
            x: next_x,
            y: next_y,
          },
          None,
        )
      } else {
        let (x, y) = (current.pos.x, current.pos.y);
        let (next_x, next_y) = match current.direction {
          Direction::Top => (x, y - 1),
          Direction::Bottom => (x, y + 1),
          Direction::Left => (x - 1, y),
          Direction::Right => (x + 1, y),
        };

        (
          Pos {
            x: next_x,
            y: next_y,
          },
          Some(prev.direction),
        )
      };

      next_positions.push((current.entity, next_pos, next_direction));
    }

    for (entity, next_pos, direction) in next_positions {
      if !is_it_safe_to_there(DoWhat::Move, next_pos, &occupied_cells) {
        commands.entity(snake_entity).despawn();
        for part in &snake.parts {
          commands.entity(part.entity).despawn();
        }
        break;
      }

      // TODO: better impl
      let part_idx = snake.parts.iter().position(|p| p.entity == entity).unwrap();
      let part = &mut snake.parts[part_idx];

      deoccupy_cell(&mut occupied_cells, part.pos);
      occupied_cells.push(OccupiedCell {
        pos: next_pos,
        by_whom: Who::Snake,
      });

      part.pos = next_pos;
      if let Some(direction) = direction {
        part.direction = direction;
      }

      let mut transform = part_transform.get_mut(entity).unwrap();
      *transform = place_at(next_pos);
    }
  }
}

fn process_snake_food(
  mut commands: Commands,
  mut snakes: Query<(Entity, &mut Snake)>,
  food: Query<(Entity, &Food)>,
  food_sound_asset: Res<FoodSoundAsset>,
  mut occupied_cells: ResMut<OccupiedCells>,
) {
  for (snake_entity, mut snake) in &mut snakes {
    let head = &snake.parts[0];
    let head_pos = head.pos;

    for (food_entity, food) in &food {
      if **food != head_pos {
        continue;
      }

      commands.entity(food_entity).despawn_recursive();
      deoccupy_cell(&mut occupied_cells, **food);
      commands.spawn((
        AudioPlayer(food_sound_asset.clone()),
        PlaybackSettings::DESPAWN,
      ));

      spawn_food(&mut commands, &mut occupied_cells);

      {
        let tail = snake.parts.last().unwrap();
        let SnakePart { pos, direction, .. } = tail;
        let (Pos { x, y }, direction) = (*pos, *direction);

        let (new_x, new_y) = match direction {
          Direction::Top => (x, y + 1),
          Direction::Bottom => (x, y - 1),
          Direction::Left => (x + 1, y),
          Direction::Right => (x - 1, y),
        };
        let new_pos = Pos { x: new_x, y: new_y };

        spawn_snake_part(
          &mut commands,
          &mut occupied_cells,
          (&mut snake, snake_entity),
          new_pos,
          direction,
        );
      }
    }
  }
}

// TEST
// TODO: should only be available for debug
fn control_snake(keyboard: Res<ButtonInput<KeyCode>>, mut snake: Query<&mut Snake>) {
  let Ok(snake) = &mut snake.get_single_mut() else {
    return;
  };
  let head = &mut snake.parts[0];

  if keyboard.pressed(KeyCode::ArrowUp) && head.direction != Direction::Bottom {
    head.direction = Direction::Top;
  }
  if keyboard.pressed(KeyCode::ArrowDown) && head.direction != Direction::Top {
    head.direction = Direction::Bottom;
  }
  if keyboard.pressed(KeyCode::ArrowRight) && head.direction != Direction::Left {
    head.direction = Direction::Right;
  }
  if keyboard.pressed(KeyCode::ArrowLeft) && head.direction != Direction::Right {
    head.direction = Direction::Left;
  }
}

// fn debug_render_occupied_cells(
//   mut commands: Commands,
//   mut materials: ResMut<Assets<StandardMaterial>>,
//   mut meshes: ResMut<Assets<Mesh>>,
//   occupied_cells: Res<OccupiedCells>,
//   entities: Query<Entity, With<OccupiedCellDebug>>,
// ) {
//   for entity in &entities {
//     commands.entity(entity).despawn();
//   }

//   for cell in &**occupied_cells {
//     let material = materials.add(StandardMaterial {
//       base_color: Srgba::rgba_u8(50, 0, 255, 100).into(),
//       alpha_mode: AlphaMode::Blend,
//       ..default()
//     });
//     let mesh = meshes.add(Cuboid::from_size(Vec3::splat(1.3)));

//     commands.spawn((
//       OccupiedCellDebug,
//       MeshMaterial3d(material),
//       Mesh3d(mesh),
//       place_at(cell.pos),
//     ));
//   }
// }

fn animate_food(time: Res<Time>, mut food: Query<&mut Transform, With<Food>>) {
  for mut food in &mut food {
    food.rotate_y(time.delta_secs() * 1.0);
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

fn spawn_snake_part(
  commands: &mut Commands,
  occupied: &mut OccupiedCells,
  (snake, snake_id): (&mut Snake, Entity),
  pos: Pos,
  direction: Direction,
) {
  let part_id = commands.spawn_empty().id();
  snake.parts.push(SnakePart {
    entity: part_id,
    direction,
    pos,
  });

  occupied.push(OccupiedCell {
    pos,
    by_whom: Who::Snake,
  });

  // TODO: this is only was needed to avoid passing material and mesh asset handles explicitly :/
  commands.queue(move |world: &mut World| {
    // TODO: is it really needed?
    if world.get_entity(snake_id).is_err() {
      panic!("failed to spawn snake (id: {snake_id:?}) part");
    }

    let SnakeAssets(material, mesh) = world.resource::<SnakeAssets>().clone();

    let mut part = world.entity_mut(part_id);
    part.insert((Mesh3d(mesh), MeshMaterial3d(material), place_at(pos)));
  });
}

fn spawn_snake(commands: &mut Commands, occupied: &mut OccupiedCells, len: u8) {
  let snake_id = commands.spawn_empty().id();
  let mut snake_component = Snake { parts: vec![] };

  spawn_snake_part(
    commands,
    occupied,
    (&mut snake_component, snake_id),
    Pos {
      x: len as i32 - 1,
      y: 0,
    },
    Direction::Right,
  );

  // TODO: use commands.spawn_batch?
  for idx in (0..=(len - 2)).rev() {
    spawn_snake_part(
      commands,
      occupied,
      (&mut snake_component, snake_id),
      Pos {
        x: idx as i32,
        y: 0,
      },
      Direction::Right,
    );
  }

  let mut snake = commands.entity(snake_id);
  snake.insert(snake_component);
}

fn place_at(pos: Pos) -> Transform {
  Transform::from_xyz(-(pos.y as f32) - 0.5 - 1.0, 0.0, (pos.x as f32) + 0.5 + 1.0)
}

fn is_it_safe_to_there(what: DoWhat, pos: Pos, occupied: &OccupiedCells) -> bool {
  let (x, y) = (pos.x, pos.y);
  let cells = CELLS.try_into().unwrap();
  let safe = x >= 0 && y >= 0 && x < cells && y < cells;
  if !safe {
    return safe;
  }

  for cell in &**occupied {
    if !(cell.pos.x == x && cell.pos.y == y) {
      continue;
    }

    match what {
      DoWhat::Spawn => {
        return false;
      }
      DoWhat::Move => match cell.by_whom {
        Who::Snake => {
          return false;
        }
        Who::Food => {
          return true;
        }
      },
    }
  }

  true
}

fn spawn_food(commands: &mut Commands, occupied: &mut OccupiedCells) {
  let cells = CELLS.try_into().unwrap();

  let pos = loop {
    let pos = Pos {
      x: rand::rng().random_range(0..cells),
      y: rand::rng().random_range(0..cells),
    };

    if is_it_safe_to_there(DoWhat::Spawn, pos, occupied) {
      break pos;
    }
  };

  occupied.push(OccupiedCell {
    pos,
    by_whom: Who::Food,
  });

  // TODO: this is only was needed to avoid passing material and mesh asset handles explicitly :/
  // (see also snake part spawn)
  commands.queue(move |world: &mut World| {
    let FoodAssets(material, mesh) = world.resource::<FoodAssets>().clone();

    world
      .spawn((
        Food(pos),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        place_at(pos).with_scale(Vec3::splat(0.5)),
      ))
      .with_child((
        Transform::from_xyz(0.0, 3.0, 0.0),
        PointLight {
          intensity: 500_000.,
          range: 5.,
          color: Srgba::RED.into(),
          shadows_enabled: false,
          shadow_depth_bias: 0.,
          ..default()
        },
      ));
  });
}

fn deoccupy_cell(occupied_cells: &mut OccupiedCells, removed: Pos) {
  let idx = occupied_cells
    .iter()
    .position(|cell| cell.pos == removed)
    .unwrap();
  occupied_cells.swap_remove(idx);
}

#[derive(Component)]
struct Snake {
  parts: Vec<SnakePart>,
}

struct SnakePart {
  entity: Entity,
  direction: Direction,
  pos: Pos,
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
  Top,
  Bottom,
  Left,
  Right,
}

#[derive(Resource, Clone)]
struct SnakeAssets(Handle<StandardMaterial>, Handle<Mesh>);

#[derive(Component, Deref)]
struct Food(Pos);

#[derive(Component)]
struct FoodSound;

#[derive(Resource, Deref)]
struct FoodSoundAsset(Handle<AudioSource>);

#[derive(Resource, Clone)]
struct FoodAssets(Handle<StandardMaterial>, Handle<Mesh>);

// position is stored in signed integers to avoid
// overflows in movement processing
#[derive(Clone, Copy, PartialEq, Eq)]
struct Pos {
  x: i32,
  y: i32,
}

#[derive(Resource, Default, Deref, DerefMut)]
struct OccupiedCells(Vec<OccupiedCell>);

struct OccupiedCell {
  pos: Pos,
  by_whom: Who,
}

#[derive(Component)]
struct OccupiedCellDebug;

enum Who {
  Snake,
  Food,
}

enum DoWhat {
  Move,
  Spawn,
}
