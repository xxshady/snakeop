relib_interface::include_exports!();
relib_interface::include_imports!();

pub use fk_core::*;

pub fn despawn(entity: Entity) {
  unsafe { gen_imports::despawn(entity.0) }
}
