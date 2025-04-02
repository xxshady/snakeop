use fk_core::{
  BevyRawAssetIndex, Entity, KeyCode, PointLight, RawEntity, Rgba, Shape,
  StableTransform,
};
use shared::{abi_stable_types::Str, imports::Imports};

relib_interface::include_exports!();
pub use gen_exports::ModuleExports;
relib_interface::include_imports!();
use gen_imports::ModuleImportsImpl;
pub use gen_imports::init_imports;

impl Imports for ModuleImportsImpl {
  fn despawn(entity: RawEntity) {
    fk::despawn(Entity(entity))
  }

  fn key_pressed(key_code: KeyCode) -> bool {
    fk::key_pressed(key_code)
  }

  fn drop_asset(index: BevyRawAssetIndex) {
    fk::drop_asset(index)
  }

  fn load_audio_asset(path: Str) -> BevyRawAssetIndex {
    fk::load_audio_asset(unsafe { path.into_str() })
  }

  fn begin_mut_entity_transform(entity: RawEntity) -> StableTransform {
    fk::begin_mut_entity_transform(Entity(entity))
  }

  fn finish_mut_entity_transform(entity: RawEntity, mutated: &StableTransform) {
    fk::finish_mut_entity_transform(Entity(entity), mutated)
  }

  fn play_audio(asset: BevyRawAssetIndex) -> RawEntity {
    fk::play_audio(asset).0
  }

  fn spawn_camera(transform: &StableTransform) -> RawEntity {
    fk::spawn_camera(transform.clone().into()).0
  }

  fn spawn_color_mesh(transform: &StableTransform, shape: &Shape, color: Rgba) -> RawEntity {
    fk::spawn_color_mesh(transform.clone().into(), shape, color).0
  }

  fn spawn_empty() -> RawEntity {
    fk::spawn_empty().0
  }

  fn spawn_point_light(transform: &StableTransform, light: &PointLight) -> RawEntity {
    fk::spawn_point_light(transform.clone().into(), light).0
  }
}
