use fk_core::BevyRawAssetIndex;

pub trait Imports {
  fn despawn(entity: u64);
  fn drop_asset(index: BevyRawAssetIndex);
  fn key_pressed(key_code: u32) -> bool;
}
