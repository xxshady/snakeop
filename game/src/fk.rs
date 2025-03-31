use bevy_input::keyboard::KeyCode;
pub use fk_core::*;
use shared::imports::BevyRawAssetIndex;

relib_interface::include_exports!();
pub use gen_exports::ModuleExportsImpl;
relib_interface::include_imports!();

pub fn despawn(entity: Entity) {
  unsafe { gen_imports::despawn(entity.0) }
}

pub fn key_pressed(key: KeyCode) -> bool {
  unsafe { gen_imports::key_pressed(key_code_enum_discriminant(&key)) }
}

pub struct AssetHandle(BevyRawAssetIndex);

impl Drop for AssetHandle {
  fn drop(&mut self) {
    unsafe { gen_imports::drop_asset(self.0) }
  }
}
