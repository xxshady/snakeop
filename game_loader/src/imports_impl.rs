use fk_core::Entity;
use gen_imports::ModuleImportsImpl;
use shared::imports::Imports;

relib_interface::include_exports!();
relib_interface::include_imports!();

impl Imports for ModuleImportsImpl {
  fn despawn(entity: u64) {
    fk::despawn(Entity(entity));
  }
}
