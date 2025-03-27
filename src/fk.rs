use std::{
  cell::{Cell, RefCell},
  mem,
};

use bevy::{
  app::{App, AppExit},
  asset::Handle,
  ecs::world::World,
  pbr::{MeshMaterial3d, StandardMaterial},
};

pub fn def<T: Default>() -> T {
  Default::default()
}

struct ColorMesh(Handle<StandardMaterial>);
struct ImageMesh(Handle<StandardMaterial>);

type Rgba = (u8, u8, u8, u8);

struct Vec3 {
  x: f32,
  y: f32,
  z: f32,
}

enum Shape {
  Cuboid(Vec3),
}

thread_local! {
  static CURRENT_WORLD: RefCell<World> = Default::default();
  static EMPTY_WORLD: RefCell<Option<World>> = Default::default();
}

pub fn take_world(world: &mut World) -> impl FnOnce(&mut App) + use<> {
  let world = mem::replace(world, EMPTY_WORLD.take().unwrap());
  CURRENT_WORLD.set(world);

  |app| {
    let world = CURRENT_WORLD.take();
    let empty_world = std::mem::replace(app.world_mut(), world);
    EMPTY_WORLD.set(Some(empty_world));
  }
}

fn use_world<R>(use_: impl FnOnce(&mut World) -> R) -> R {
  CURRENT_WORLD.with_borrow_mut(|world| {
    EMPTY_WORLD.with_borrow(|empty_world| {
      assert_ne!(world.id(), empty_world.as_ref().unwrap().id());
    });
    use_(world)
  })
}

pub fn app_runner(mut app: App) -> AppExit {
  loop {
    let return_world = take_world(app.world_mut());
    app.update();
    return_world(&mut app);

    if let Some(exit) = app.should_exit() {
      return exit;
    }
  }
}

pub fn new_color_mesh(shape: Shape, color: Rgba) -> ColorMesh {
  use_world(|world| {
    // TODO:
    todo!()
  })
}

// TODO:
// pub fn new_image_mesh() -> ImageMesh {}
