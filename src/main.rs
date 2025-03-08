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

fn main() {
  App::new()
    .add_plugins((
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      WorldInspectorPlugin::default(),
    ))
    .insert_resource(Time::<Fixed>::from_seconds(0.5))
    .add_systems(Startup, setup)
    .add_systems(
      FixedUpdate,
      (
        control_snake,
        process_snake_movement,
        process_snake_food,
        test_sound,
      )
        .chain(),
    )
    .run();
}

// TEST
fn test_sound(sound: Query<&AudioSink, With<FoodSound>>) {
  let Ok(sound) = sound.get_single() else {
    return;
  };

  sound.play();
}

fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut images: ResMut<Assets<Image>>,
  mut materials: ResMut<Assets<StandardMaterial>>,
  asset_server: Res<AssetServer>,
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
    Transform::from_xyz(-offset, size * 1.5, offset)
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

  spawn_snake(&mut commands, 8);

  let food_sound = AudioPlayer::new(asset_server.load("sounds/smb_coin.wav"));
  commands.spawn((food_sound, FoodSound));
}

fn process_snake_movement(
  mut snakes: Query<(Entity, &mut Snake)>,
  mut commands: Commands,
  mut part_transform: Query<&mut Transform>,
) {
  for (snake_entity, mut snake) in &mut snakes {
    let mut next_positions = vec![];

    {
      let head = &snake.parts[0];
      let (x, y) = (head.x, head.y);

      let (next_x, next_y) = match head.direction {
        Direction::Top => (x, y - 1),
        Direction::Bottom => (x, y + 1),
        Direction::Left => (x - 1, y),
        Direction::Right => (x + 1, y),
      };

      next_positions.push((head.entity, next_x, next_y, None));
    }

    for window in snake.parts.windows(2) {
      let [prev, current] = window else {
        unreachable!();
      };

      let (next_x, next_y, next_direction) = if current.direction == prev.direction {
        let (x, y) = (current.x, current.y);
        let (next_x, next_y) = match current.direction {
          Direction::Top => (x, y - 1),
          Direction::Bottom => (x, y + 1),
          Direction::Left => (x - 1, y),
          Direction::Right => (x + 1, y),
        };

        (next_x, next_y, None)
      } else {
        let (x, y) = (current.x, current.y);
        let (next_x, next_y) = match current.direction {
          Direction::Top => (x, y - 1),
          Direction::Bottom => (x, y + 1),
          Direction::Left => (x - 1, y),
          Direction::Right => (x + 1, y),
        };

        (next_x, next_y, Some(prev.direction))
      };

      next_positions.push((current.entity, next_x, next_y, next_direction));
    }

    for (entity, x, y, direction) in next_positions {
      if !is_it_safe_to_move_there(x, y, &snake.parts) {
        commands.entity(snake_entity).despawn();
        for part in &snake.parts {
          commands.entity(part.entity).despawn();
        }
        break;
      }

      // TODO: better impl
      let part_idx = snake.parts.iter().position(|p| p.entity == entity).unwrap();
      let part = &mut snake.parts[part_idx];
      part.x = x;
      part.y = y;
      if let Some(direction) = direction {
        part.direction = direction;
      }

      let mut transform = part_transform.get_mut(entity).unwrap();
      *transform = place_at(x, y);
    }
  }
}

fn process_snake_food(
  mut commands: Commands,
  mut snakes: Query<&mut Snake>,
  food: Query<(Entity, &Food)>,
) {
  for snake in &mut snakes {
    let head = &snake.parts[0];
    for (food_entity, food) in &food {
      if food.x == head.x && food.y == head.y {
        commands.entity(food_entity).despawn();
        // TODO: play sound
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
  (snake, snake_id): (&mut Snake, Entity),
  x: i32,
  y: i32,
  direction: Direction,
) {
  let part_id = commands.spawn_empty().id();
  snake.parts.push(SnakePart {
    entity: part_id,
    direction,
    x,
    y,
  });

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

fn spawn_snake(commands: &mut Commands, len: u8) {
  let snake_id = commands.spawn_empty().id();
  let mut snake_component = Snake { parts: vec![] };

  spawn_snake_part(
    commands,
    (&mut snake_component, snake_id),
    len as i32 - 1,
    0,
    Direction::Right,
  );

  // TODO: use commands.spawn_batch?
  for idx in (0..=(len - 2)).rev() {
    spawn_snake_part(
      commands,
      (&mut snake_component, snake_id),
      idx as i32,
      0,
      Direction::Right,
    );
  }

  let mut snake = commands.entity(snake_id);
  snake.insert(snake_component);
}

fn place_at(x: i32, y: i32) -> Transform {
  Transform::from_xyz(-(y as f32) - 0.5 - 1.0, 0.0, (x as f32) + 0.5 + 1.0)
}

fn is_it_safe_to_move_there(x: i32, y: i32, snake_parts: &[SnakePart]) -> bool {
  let cells = CELLS.try_into().unwrap();
  let safe = x >= 0 && y >= 0 && x < cells && y < cells;
  if !safe {
    return safe;
  }

  for part in snake_parts {
    if part.x == x && part.y == y {
      return false;
    }
  }

  true
}

#[derive(Component)]
struct Snake {
  parts: Vec<SnakePart>,
}

struct SnakePart {
  entity: Entity,
  direction: Direction,

  // position is stored in signed integers to avoid
  // overflows in movement processing
  x: i32,
  y: i32,
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

#[derive(Component)]
struct Food {
  x: i32,
  y: i32,
}

#[derive(Component)]
struct FoodSound;
