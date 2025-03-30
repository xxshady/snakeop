use std::{cell::RefCell, mem};

use bevy::prelude::*;

pub fn def<T: Default>() -> T {
  Default::default()
}

#[derive(Clone, Copy, PartialEq)]
pub struct Entity(u64);

impl From<bevy::ecs::entity::Entity> for Entity {
  fn from(entity: bevy::ecs::entity::Entity) -> Self {
    Self(entity.to_bits())
  }
}

impl From<Entity> for bevy::ecs::entity::Entity {
  fn from(entity: Entity) -> Self {
    Self::from_bits(entity.0)
  }
}

type Rgba = (u8, u8, u8, u8);

pub enum Shape {
  Cuboid(Vec3),
  Plane(f32, f32),
}

pub struct PointLight {
  pub intensity: f32,
  pub range: f32,
  pub shadows_enabled: bool,
  pub shadow_depth_bias: f32,
  pub color: Rgba,
}

thread_local! {
  static CURRENT_WORLD: RefCell<World> = Default::default();
  static EMPTY_WORLD: RefCell<Option<World>> = RefCell::new(Some(World::new()));
}

pub fn take_world(world: &mut World) -> impl FnOnce(&mut World) + use<> {
  let world = mem::replace(world, EMPTY_WORLD.take().unwrap());
  CURRENT_WORLD.set(world);

  |world_| {
    let world = CURRENT_WORLD.take();
    let empty_world = mem::replace(world_, world);
    EMPTY_WORLD.set(Some(empty_world));
  }
}

fn use_world<R>(use_: impl FnOnce(&mut World) -> R) -> R {
  CURRENT_WORLD.with_borrow_mut(|world| {
    EMPTY_WORLD.with_borrow(|empty_world| {
      assert!(empty_world.is_none());
      // assert_ne!(world.id(), empty_world.as_ref().unwrap().id());
    });
    use_(world)
  })
}

// TODO: cache assets
pub fn spawn_color_mesh(transform: Transform, shape: Shape, color: Rgba) -> Entity {
  use_world(|world| {
    let mesh: Mesh = match shape {
      Shape::Cuboid(size) => Cuboid::from_size(size).into(),
      Shape::Plane(width, height) => Plane3d::default().mesh().size(width, height).into(),
    };
    let mesh: Handle<Mesh> = world.add_asset(mesh);
    let mesh = Mesh3d(mesh);
    let material: Handle<StandardMaterial> = world.add_asset(StandardMaterial {
      base_color: Srgba::rgba_u8(color.0, color.1, color.2, color.3).into(),
      ..def()
    });
    let material = MeshMaterial3d(material);

    world.spawn((transform, mesh, material)).id().into()
  })
}

// TODO: abstract away from bevy Image
// TODO: cache assets
pub fn spawn_image_mesh(transform: Transform, shape: Shape, image: Image) -> Entity {
  use_world(|world| {
    let mesh: Mesh = match shape {
      Shape::Cuboid(size) => Cuboid::from_size(size).into(),
      Shape::Plane(width, height) => Plane3d::default().mesh().size(width, height).into(),
    };
    let mesh: Handle<Mesh> = world.add_asset(mesh);
    let mesh = Mesh3d(mesh);

    let texture: Handle<Image> = world.add_asset(image);
    let material: Handle<StandardMaterial> = world.add_asset(StandardMaterial {
      base_color_texture: Some(texture),
      ..def()
    });
    let material = MeshMaterial3d(material);

    world.spawn((transform, mesh, material)).id().into()
  })
}

pub fn spawn_point_light(transform: Transform, light: PointLight) -> Entity {
  use_world(|world| {
    let entity = world
      .spawn((
        transform,
        bevy::pbr::PointLight {
          intensity: light.intensity,
          range: light.range,
          shadows_enabled: light.shadows_enabled,
          shadow_depth_bias: light.shadow_depth_bias,
          color: Srgba::rgba_u8(light.color.0, light.color.1, light.color.2, light.color.3).into(),
          ..def()
        },
      ))
      .id()
      .into();
    entity
  })
}

pub fn spawn_camera(transform: Transform) -> Entity {
  use_world(|world| {
    let entity = world.spawn((Camera3d::default(), transform)).id().into();
    entity
  })
}

pub fn key_pressed(key_code: KeyCode) -> bool {
  use_world(|world| {
    let input = world.resource::<ButtonInput<KeyCode>>();
    input.pressed(key_code)
  })
}

pub fn spawn_empty() -> Entity {
  use_world(|world| world.spawn_empty().id().into())
}

pub fn despawn(entity: Entity) {
  use_world(|world| {
    world.despawn(entity.into());
  })
}

pub fn mut_entity_transform<R>(
  entity: Entity,
  mutate: impl FnOnce(&mut Transform) -> R,
) -> Option<R> {
  use_world(|world| {
    let transform = world.get_mut::<Transform>(entity.into());
    if let Some(mut transform) = transform {
      return Some(mutate(&mut transform));
    }
    None
  })
}

#[derive(Clone)]
pub struct AudioAsset(Handle<AudioSource>);

pub fn load_asset(path: &str) -> AudioAsset {
  use_world(|world| AudioAsset(world.load_asset(path)))
}

pub fn play_audio(asset: AudioAsset) -> Entity {
  use_world(|world| {
    world
      .spawn((AudioPlayer(asset.0), PlaybackSettings::DESPAWN))
      .id()
      .into()
  })
}
