use std::{
  cell::RefCell,
  time::{Duration, Instant},
};
use crate::{
  fk::{
    def, despawn, key_pressed, load_asset, mut_entity_transform, play_audio, spawn_camera,
    spawn_color_mesh, spawn_empty, spawn_image_mesh, spawn_point_light, AudioAsset, Entity,
    PointLight, Shape,
  },
  grid_texture, CELLS,
};
use bevy::{input::keyboard::KeyCode, math::Vec3, transform::components::Transform};
use rand::Rng;

thread_local! {
  static STATE: RefCell<Option<State>> = RefCell::new(None);
}

struct State {
  last_update: Instant,
  since_last_update: Duration,
  since_last_fixed_update: Duration,
  occupied_cells: OccupiedCells,
  snakes: Vec<Snake>,
  food: Vec<Food>,
  food_sound: AudioAsset,
}

pub fn setup() {
  let size = (CELLS + 2) as f32;
  let offset = size / 2.0;

  // the grid
  spawn_image_mesh(
    Transform::from_xyz(-offset, 0.0, offset),
    Shape::Plane(size, size),
    grid_texture(),
  );

  spawn_camera(
    Transform::from_xyz(-offset + (-offset / 2.), size * 1.5, offset - (offset / 2.))
      .looking_at(Vec3::new(-offset, 0., offset), Vec3::X),
  );

  spawn_point_light(
    Transform::from_xyz(8.0, 16.0, 8.0),
    PointLight {
      shadows_enabled: true,
      intensity: 10_000_000.,
      range: 100.0,
      shadow_depth_bias: 0.2,
      color: (255, 255, 255, 255),
    },
  );

  let (mut snakes, mut occupied_cells, mut food) = def();
  spawn_snake(&mut snakes, &mut occupied_cells, 3);

  let food_sound = load_asset("sounds/smb_coin.wav");

  spawn_food(&mut food, &mut occupied_cells);

  STATE.set(Some(State {
    last_update: Instant::now(),
    since_last_update: Duration::ZERO,
    since_last_fixed_update: Duration::ZERO,
    occupied_cells,
    snakes,
    food,
    food_sound,
  }));
}

pub fn update() {
  STATE.with_borrow_mut(|state| {
    let state = state.as_mut().unwrap();

    control_snake(&mut state.snakes);

    let now = Instant::now();
    let since_last_update = now.duration_since(state.last_update);
    state.last_update = now;
    state.since_last_update = since_last_update;

    state.since_last_fixed_update += since_last_update;
    if state.since_last_fixed_update < Duration::from_millis(300) {
      return;
    }

    fixed_update(state);

    state.since_last_fixed_update = Duration::ZERO;
  });
}

fn fixed_update(state: &mut State) {
  process_snake_movement(state);
  process_snake_food(state);
  animate_food(state);
}

fn animate_food(state: &mut State) {
  for food in &mut state.food {
    let rotate_for = state.since_last_fixed_update.as_millis() as f32 / 1000.0;
    mut_entity_transform(food.entity, |transform| {
      transform.rotate_y(rotate_for);
    });
  }
}

fn spawn_snake_part(
  occupied: &mut OccupiedCells,
  snake: &mut Snake,
  pos: Pos,
  direction: Direction,
) {
  let entity = spawn_color_mesh(
    place_at(pos),
    Shape::Cuboid(Vec3::splat(1.)),
    (0, 255, 0, 255),
  );
  snake.parts.push(SnakePart {
    entity,
    direction,
    pos,
  });
  occupied.push(OccupiedCell {
    pos,
    by_whom: Who::Snake,
  });
}

fn spawn_snake(snakes: &mut Vec<Snake>, occupied: &mut OccupiedCells, len: u8) {
  let mut snake = Snake {
    entity: spawn_empty(),
    parts: vec![],
    direction: Direction::Right,
    next_direction: Direction::Right,
  };

  spawn_snake_part(
    occupied,
    &mut snake,
    Pos {
      x: len as i32 - 1,
      y: 0,
    },
    Direction::Right,
  );

  for idx in (0..=(len - 2)).rev() {
    spawn_snake_part(
      occupied,
      &mut snake,
      Pos {
        x: idx as i32,
        y: 0,
      },
      Direction::Right,
    );
  }

  snakes.push(snake);
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

fn spawn_food(food: &mut Vec<Food>, occupied: &mut OccupiedCells) {
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

  let entity = spawn_color_mesh(
    place_at(pos).with_scale(Vec3::splat(0.5)),
    Shape::Cuboid(Vec3::splat(1.)),
    (255, 0, 0, 255),
  );

  let light = spawn_point_light(
    {
      let mut transform = place_at(pos);
      transform.translation.y += 3.0;
      transform.translation.x -= 0.15;
      transform.translation.z += 0.15;
      transform
    },
    PointLight {
      intensity: 220_000.,
      range: 5.,
      color: (255, 0, 0, 255),
      shadows_enabled: false,
      shadow_depth_bias: 0.,
    },
  );

  food.push(Food { entity, pos, light });
}

fn deoccupy_cell(occupied_cells: &mut OccupiedCells, removed: Pos) {
  let idx = occupied_cells
    .iter()
    .position(|cell| cell.pos == removed)
    .unwrap();
  occupied_cells.swap_remove(idx);
}

struct Snake {
  entity: Entity,
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

struct Food {
  entity: Entity,
  pos: Pos,
  light: Entity,
}

// position is stored in signed integers to avoid
// overflows in movement processing
#[derive(Clone, Copy, PartialEq, Eq)]
struct Pos {
  x: i32,
  y: i32,
}

type OccupiedCells = Vec<OccupiedCell>;

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
fn control_snake(snakes: &mut [Snake]) {
  let Some(snake) = snakes.first_mut() else {
    return;
  };

  if key_pressed(KeyCode::ArrowUp) && snake.direction != Direction::Down {
    snake.next_direction = Direction::Up;
  }
  if key_pressed(KeyCode::ArrowDown) && snake.direction != Direction::Up {
    snake.next_direction = Direction::Down;
  }
  if key_pressed(KeyCode::ArrowRight) && snake.direction != Direction::Left {
    snake.next_direction = Direction::Right;
  }
  if key_pressed(KeyCode::ArrowLeft) && snake.direction != Direction::Right {
    snake.next_direction = Direction::Left;
  }
}

fn process_snake_movement(state: &mut State) {
  let mut snakes_to_remove = vec![];

  for snake in &mut state.snakes {
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
      if !is_it_safe_to_there(DoWhat::Move, next_pos, &state.occupied_cells) {
        despawn(snake.entity);
        for part in &snake.parts {
          despawn(part.entity);
        }
        snakes_to_remove.push(snake.entity);

        break;
      }

      // TODO: better impl
      let part_idx = snake.parts.iter().position(|p| p.entity == entity).unwrap();
      let part = &mut snake.parts[part_idx];

      deoccupy_cell(&mut state.occupied_cells, part.pos);
      state.occupied_cells.push(OccupiedCell {
        pos: next_pos,
        by_whom: Who::Snake,
      });

      part.pos = next_pos;
      if let Some(direction) = direction {
        part.direction = direction;
      }

      mut_entity_transform(entity, |transform| {
        *transform = place_at(next_pos);
      });
    }
  }

  for snake in snakes_to_remove {
    let idx = state
      .snakes
      .iter()
      .position(|snake_| snake_.entity == snake)
      .unwrap();
    state.snakes.swap_remove(idx);
  }
}

fn process_snake_food(state: &mut State) {
  for snake in &mut state.snakes {
    let head = &snake.parts[0];
    let head_pos = head.pos;

    let mut despawn_food = None;
    for food in &mut state.food {
      if food.pos != head_pos {
        continue;
      }
      despawn_food = Some(food);
    }

    if let Some(food) = despawn_food {
      despawn(food.entity);
      despawn(food.light);
      deoccupy_cell(&mut state.occupied_cells, food.pos);

      play_audio(state.food_sound.clone());

      spawn_food(&mut state.food, &mut state.occupied_cells);

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

        spawn_snake_part(&mut state.occupied_cells, snake, new_pos, direction);
      }
    }
  }
}
