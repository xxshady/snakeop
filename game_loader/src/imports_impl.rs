use fk_core::{BevyRawAssetIndex, Entity};
use shared::imports::Imports;

relib_interface::include_exports!();
pub use gen_exports::ModuleExports;
relib_interface::include_imports!();
use gen_imports::ModuleImportsImpl;
pub use gen_imports::init_imports;

impl Imports for ModuleImportsImpl {
  fn despawn(entity: u64) {
    fk::despawn(Entity(entity))
  }

  fn drop_asset(index: BevyRawAssetIndex) {
    fk::drop_asset(index)
  }

  fn key_pressed(key_code: u32) -> bool {
    fk::key_pressed(key_code)
  }
}
