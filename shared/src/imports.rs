use fk_core::{BevyRawAssetIndex, KeyCode, PointLight, RawEntity, Rgba, Shape, StableTransform};
use crate::abi_stable_types::Str;

pub trait Imports {
  fn despawn(entity: RawEntity);
  fn key_pressed(key_code: KeyCode) -> bool;
  fn drop_asset(index: BevyRawAssetIndex);
  fn load_audio_asset(path: Str) -> BevyRawAssetIndex;
  fn begin_mut_entity_transform(entity: RawEntity) -> StableTransform;
  fn finish_mut_entity_transform(entity: RawEntity, mutated: &StableTransform);
  fn play_audio(asset: BevyRawAssetIndex) -> RawEntity;
  fn spawn_camera(transform: &StableTransform) -> RawEntity;
  fn spawn_color_mesh(transform: &StableTransform, shape: &Shape, color: Rgba) -> RawEntity;
  fn spawn_empty() -> RawEntity;
  fn spawn_point_light(transform: &StableTransform, light: &PointLight) -> RawEntity;
}
