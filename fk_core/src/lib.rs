use bevy_input::keyboard::KeyCode;
use bevy_math::Vec3;

pub fn def<T: Default>() -> T {
  Default::default()
}

#[derive(Clone, Copy, PartialEq)]
pub struct Entity(pub u64);

/// See <https://docs.rs/bevy_asset/0.15.3/bevy_asset/struct.AssetIndex.html>
pub type BevyRawAssetIndex = u64;

#[derive(Clone)]
pub struct AudioAsset(pub BevyRawAssetIndex);

pub type Rgba = (u8, u8, u8, u8);

#[repr(C)]
pub enum Shape {
  Cuboid(Vec3),
  Plane(f32, f32),
}

#[repr(C)]
pub struct PointLight {
  pub intensity: f32,
  pub range: f32,
  pub shadows_enabled: bool,
  pub shadow_depth_bias: f32,
  pub color: Rgba,
}

/// See [`core::mem::discriminant`]
pub fn key_code_enum_discriminant(key_code: &KeyCode) -> u32 {
  // SAFETY: Because `KeyCode` is marked `repr(u32)`, its layout is a `repr(C)` `union`
  // between `repr(C)` structs, each of which has the `u32` discriminant as its first
  // field, so we can read the discriminant without offsetting the pointer.
  unsafe { *<*const _>::from(key_code).cast::<u32>() }
}
