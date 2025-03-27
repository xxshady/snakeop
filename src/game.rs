use std::{
  cell::RefCell,
  time::{Duration, Instant},
};
use crate::{fk::def, CELLS};
use rand::Rng;

thread_local! {
  static STATE: RefCell<State> = RefCell::new(State::new());
}

struct State {
  time: Instant,

  // does it really needs its own struct?
  occupied_cells: OccupiedCells,
}

impl State {
  fn new() -> Self {
    Self {
      time: Instant::now(),
      occupied_cells: def(),
    }
  }
}

pub fn setup() {
  let mut materials: bevy::ecs::system::ResMut<bevy::asset::Assets<bevy::pbr::StandardMaterial>>;
  // the grid
  let grid_material = materials.add(bevy::pbr::StandardMaterial {
    // base_color: RED.into(),
    base_color_texture: Some(images.add(grid_texture())),
    ..def()
  });

  let size = (CELLS + 2) as f32;
  let offset = size / 2.0;

  commands.spawn((
    Mesh3d(meshes.add(Plane3d::default().mesh().size(size, size))),
    // MeshMaterial3d(materials.add(Color::from(SILVER))),
    bevy::pbr::MeshMaterial3d(grid_material),
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
      ..def()
    },
    Transform::from_xyz(8.0, 16.0, 8.0),
  ));

  let snake_material = materials.add(StandardMaterial {
    base_color: Srgba::GREEN.into(),
    ..def()
  });
  let snake_mesh = meshes.add(Cuboid::from_size(Vec3::new(1., 1., 1.)));

  commands.insert_resource(SnakeAssets(snake_material, snake_mesh));

  spawn_snake(&mut commands, &mut occupied_cells, 3);

  let food_sound_asset = asset_server.load("sounds/smb_coin.wav");
  commands.insert_resource(FoodSoundAsset(food_sound_asset));

  let food_material = materials.add(StandardMaterial {
    base_color: RED.into(),
    ..def()
  });
  let food_mesh = meshes.add(Cuboid::from_size(Vec3::new(1., 1., 1.)));

  commands.insert_resource(FoodAssets(food_material, food_mesh));

  spawn_food(&mut commands, &mut occupied_cells);
}

pub fn update() {
  STATE.with_borrow_mut(|state| {
    fixed_update(state);
  });
}

pub fn fixed_update(state: &mut State) {
  let now = Instant::now();
  let since_last_update = now.duration_since(state.time);
  if since_last_update < Duration::from_millis(500) {
    return;
  }

  state.time = now;
  process_snake_movement(state);
  process_snake_food(state);
  animate_food(state);
}

fn animate_food(state: &mut State) {
  for mut food in &mut state.food {
    food.rotate_y(time.delta_secs() * 1.0);
  }
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
  let mut snake_component = Snake {
    parts: vec![],
    direction: Direction::Right,
    next_direction: Direction::Right,
  };

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
        Transform::from_xyz(0.0, 3.0, 0.0).looking_at(Vec3::ZERO, Dir3::Y),
        SpotLight {
          color: Srgba::RED.into(),
          intensity: 250000.,
          range: 5.,
          ..def()
        },
        // PointLight {
        //   intensity: 500_000.,
        //   range: 5.,
        //   color: Srgba::RED.into(),
        //   shadows_enabled: false,
        //   shadow_depth_bias: 0.,
        //   ..def()
        // },
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

  // these states are needed here to prevent player killing themselves
  // by changing direction two times before update (see control_snake)
  direction: Direction,
  next_direction: Direction,
}

struct SnakePart {
  entity: Entity,
  direction: Direction,
  pos: Pos,
}

#[derive(PartialEq, Clone, Copy)]
enum Direction {
  Up,
  Down,
  Left,
  Right,
}

#[derive(Resource, Clone)]
struct SnakeAssets(Handle<StandardMaterial>, Handle<Mesh>);

#[derive(Component, Deref)]
struct Food(Pos);

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

#[derive(Default)]
struct OccupiedCells(Vec<OccupiedCell>);

struct OccupiedCell {
  pos: Pos,
  by_whom: Who,
}

enum Who {
  Snake,
  Food,
}

enum DoWhat {
  Move,
  Spawn,
}

// TEST
// TODO: should only be available for debug?
fn control_snake(keyboard: Res<ButtonInput<KeyCode>>, mut snake: Query<&mut Snake>) {
  let Ok(snake) = &mut snake.get_single_mut() else {
    return;
  };
  // let head = &mut snake.parts[0];

  if keyboard.pressed(KeyCode::ArrowUp) && snake.direction != Direction::Down {
    snake.next_direction = Direction::Up;
  }
  if keyboard.pressed(KeyCode::ArrowDown) && snake.direction != Direction::Up {
    snake.next_direction = Direction::Down;
  }
  if keyboard.pressed(KeyCode::ArrowRight) && snake.direction != Direction::Left {
    snake.next_direction = Direction::Right;
  }
  if keyboard.pressed(KeyCode::ArrowLeft) && snake.direction != Direction::Right {
    snake.next_direction = Direction::Left;
  }
}

fn process_snake_movement(state: &mut State) {
  for (snake_entity, mut snake) in &mut snakes {
    let mut next_positions = vec![];

    {
      let next_direction = snake.next_direction;
      snake.direction = next_direction;
      let head = &mut snake.parts[0];
      head.direction = next_direction;

      let (x, y) = (head.pos.x, head.pos.y);

      let (next_x, next_y) = match next_direction {
        Direction::Up => (x, y - 1),
        Direction::Down => (x, y + 1),
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
          Direction::Up => (x, y - 1),
          Direction::Down => (x, y + 1),
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
          Direction::Up => (x, y - 1),
          Direction::Down => (x, y + 1),
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
          Direction::Up => (x, y + 1),
          Direction::Down => (x, y - 1),
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
