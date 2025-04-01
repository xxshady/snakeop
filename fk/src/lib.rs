use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  mem,
  sync::Arc,
};

use fk_core::*;

use bevy::{
  asset::{AssetId, DirectAssetAccessExt, Handle, StrongHandle},
  audio::{AudioPlayer, AudioSource, PlaybackSettings},
  color::Srgba,
  core_pipeline::core_3d::Camera3d,
  ecs::{entity::Entity as BevyEntity, world::World},
  image::Image,
  input::{
    keyboard::{KeyCode, NativeKeyCode},
    ButtonInput,
  },
  math::{
    primitives::{Cuboid, Plane3d},
    Vec3,
  },
  pbr::{MeshMaterial3d, StandardMaterial},
  render::mesh::{Mesh, Mesh3d, Meshable},
  transform::components::Transform,
};

pub fn entity_to_bevy(entity: Entity) -> BevyEntity {
  BevyEntity::from_bits(entity.0)
}

pub fn bevy_to_entity(bevy: BevyEntity) -> Entity {
  Entity(bevy.to_bits())
}

thread_local! {
  static CURRENT_WORLD: RefCell<World> = def();
  static EMPTY_WORLD: RefCell<Option<World>> = RefCell::new(Some(World::new()));
  static ASSET_HANDLES: RefCell<HashMap<BevyRawAssetIndex, Arc<StrongHandle>>> = def();
  static ENTITIES: RefCell<HashSet<BevyEntity>> = def();
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
pub fn spawn_color_mesh(transform: Transform, shape: &Shape, color: Rgba) -> Entity {
  use_world(|world| {
    let mesh: Mesh = match shape {
      Shape::Cuboid(size) => Cuboid::from_size(*size).into(),
      Shape::Plane(width, height) => Plane3d::default().mesh().size(*width, *height).into(),
    };
    let mesh: Handle<Mesh> = world.add_asset(mesh);
    let mesh = Mesh3d(mesh);
    let material: Handle<StandardMaterial> = world.add_asset(StandardMaterial {
      base_color: Srgba::rgba_u8(color.0, color.1, color.2, color.3).into(),
      ..def()
    });
    let material = MeshMaterial3d(material);

    let entity = world.spawn((transform, mesh, material)).id();
    ENTITIES.with_borrow_mut(|entities| entities.insert(entity));

    let entity = bevy_to_entity(entity);
    entity
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

    let entity = world.spawn((transform, mesh, material)).id();

    ENTITIES.with_borrow_mut(|entities| entities.insert(entity));

    let entity = bevy_to_entity(entity);
    entity
  })
}

pub fn spawn_point_light(transform: Transform, light: &PointLight) -> Entity {
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
      .id();

    ENTITIES.with_borrow_mut(|entities| entities.insert(entity));

    let entity = bevy_to_entity(entity);
    entity
  })
}

pub fn spawn_camera(transform: Transform) -> Entity {
  use_world(|world| {
    let entity = world.spawn((Camera3d::default(), transform)).id();
    ENTITIES.with_borrow_mut(|entities| entities.insert(entity));
    let entity = bevy_to_entity(entity);
    entity
  })
}

pub fn key_pressed(key_code: u32) -> bool {
  assert_ne!(
    key_code,
    key_code_enum_discriminant(&KeyCode::Unidentified(NativeKeyCode::Unidentified))
  );
  // max key code (variant_count is unstable)
  assert!(key_code <= key_code_enum_discriminant(&KeyCode::F35));

  // SAFETY: KeyCode enum is marked as repr(u32) so first u32 is always discriminant
  // (checked in miri and it doesnt complain)
  let key_code = unsafe { std::mem::transmute::<[u32; 3], KeyCode>([key_code, 0, 0]) };

  use_world(|world| {
    let input = world.resource::<ButtonInput<KeyCode>>();
    input.pressed(key_code)
  })
}

pub fn spawn_empty() -> Entity {
  use_world(|world| {
    let entity = world.spawn_empty().id();
    ENTITIES.with_borrow_mut(|entities| entities.insert(entity));
    let entity = bevy_to_entity(entity);
    entity
  })
}

pub fn despawn(entity: Entity) {
  use_world(|world| {
    world.despawn(entity_to_bevy(entity));
  })
}

pub fn begin_mut_entity_transform(entity: Entity) -> StableTransform {
  use_world(|world| {
    world
      .get::<Transform>(entity_to_bevy(entity))
      .cloned()
      .unwrap()
      .into()
  })
}

pub fn finish_mut_entity_transform(entity: Entity, mutated: &StableTransform) {
  use_world(|world| {
    let mut transform = world.get_mut::<Transform>(entity_to_bevy(entity)).unwrap();
    transform.translation = mutated.translation;
    transform.rotation = mutated.rotation;
    transform.scale = mutated.scale;
  })
}

pub fn load_audio_asset(path: &str) -> AudioAsset {
  use_world(|world| {
    let handle: Handle<AudioSource> = world.load_asset(path);

    let AssetId::Index {
      index: handle_index,
      marker: _,
    } = handle.id()
    else {
      unreachable!();
    };
    let handle_index = handle_index.to_bits();

    ASSET_HANDLES.with_borrow_mut({
      // let handle = handle.clone();
      let Handle::Strong(handle) = handle else {
        unreachable!();
      };
      |handles| {
        handles.insert(handle_index, handle);
      }
    });

    AudioAsset(handle_index)
  })
}

pub fn play_audio(asset: AudioAsset) -> Entity {
  use_world(|world| {
    let handle = ASSET_HANDLES.with_borrow(|handles| handles.get(&asset.0).cloned().unwrap());

    let entity = world
      .spawn((
        AudioPlayer(Handle::<AudioSource>::Strong(handle)),
        PlaybackSettings::DESPAWN,
      ))
      .id();

    bevy_to_entity(entity)
  })
}

pub fn drop_asset(index: BevyRawAssetIndex) {
  ASSET_HANDLES.with_borrow_mut(|handles| {
    handles.remove(&index).unwrap();
  });
}

pub fn clear_world(world: &mut World) {
  ASSET_HANDLES.with_borrow_mut(|handles| {
    handles.clear();
  });
  ENTITIES.with_borrow_mut(|entities| {
    for entity in entities.iter() {
      world.despawn(*entity);
    }
  });
}
