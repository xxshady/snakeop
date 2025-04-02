use bevy_transform::components::Transform;
pub use fk_core::*;

relib_interface::include_exports!();
pub use gen_exports::ModuleExportsImpl;
relib_interface::include_imports!(gen_imports);

pub fn despawn(entity: Entity) {
  unsafe { gen_imports::despawn(entity.0) }
}

pub fn key_pressed(key: KeyCode) -> bool {
  unsafe { gen_imports::key_pressed(key) }
}

pub fn load_audio_asset(path: &str) -> AudioAsset {
  let index = unsafe { gen_imports::load_audio_asset(path.into()) };
  AudioAsset(index)
}

pub fn mut_entity_transform<R>(entity: Entity, mutate: impl FnOnce(&mut Transform) -> R) -> R {
  let mut mutated = begin_mut_entity_transform(entity);
  let returned = mutate(&mut mutated);
  finish_mut_entity_transform(entity, mutated);
  returned
}

fn begin_mut_entity_transform(entity: Entity) -> Transform {
  unsafe { gen_imports::begin_mut_entity_transform(entity.0) }.into()
}

fn finish_mut_entity_transform(entity: Entity, mutated: Transform) {
  unsafe { gen_imports::finish_mut_entity_transform(entity.0, &mutated.into()) }
}

pub struct AssetHandle(BevyRawAssetIndex);

impl Drop for AssetHandle {
  fn drop(&mut self) {
    unsafe { gen_imports::drop_asset(self.0) }
  }
}

pub fn play_audio(audio: AudioAsset) -> Entity {
  let entity = unsafe { gen_imports::play_audio(audio.0) };
  Entity(entity)
}

pub fn spawn_camera(transform: Transform) -> Entity {
  let entity = unsafe { gen_imports::spawn_camera(&transform.into()) };
  Entity(entity)
}

pub fn spawn_color_mesh(transform: Transform, shape: &Shape, color: Rgba) -> Entity {
  let entity = unsafe { gen_imports::spawn_color_mesh(&transform.into(), shape, color) };
  Entity(entity)
}

pub fn spawn_empty() -> Entity {
  let entity = unsafe { gen_imports::spawn_empty() };
  Entity(entity)
}

pub fn spawn_point_light(transform: Transform, light: &PointLight) -> Entity {
  let entity = unsafe { gen_imports::spawn_point_light(&transform.into(), light) };
  Entity(entity)
}
