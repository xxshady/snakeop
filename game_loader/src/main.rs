mod imports_impl;

use std::{
  cell::{Cell, RefCell},
  marker::PhantomData,
};

use bevy::{
  prelude::*,
  render::{
    render_asset::RenderAssetUsages,
    render_resource::{Extent3d, TextureDimension, TextureFormat},
  },
  tasks::{TaskPool, TaskPoolBuilder},
  window::{ClosingWindow, WindowCloseRequested},
  winit::WinitPlugin,
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fk_core::def;
use imports_impl::{init_imports, ModuleExports};
use relib_host::{load_module, Module};

fn main() {
  load_game();

  let game_update = |world: &mut World| {
    let setup_called = GAME_INSTANCE.with_borrow_mut(|(_, setup_called)| {
      let called = *setup_called;
      if !called {
        *setup_called = true;
      }
      called
    });

    if !setup_called {
      dbg!();
      fk::clear_world(world);

      let return_world = fk::take_world(world);
      call_game_export(|game| unsafe {
        game.setup().unwrap();
      });
      return_world(world);
    }

    let return_world = fk::take_world(world);
    call_game_export(|game| unsafe {
      game.update().unwrap();
    });
    return_world(world);
  };

  App::new()
    .insert_non_send_resource(SingleThreaded(PhantomData))
    .add_plugins((
      DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
          close_when_requested: false,
          ..def()
        }),
      WorldInspectorPlugin::default(),
    ))
    .add_systems(Update, (game_update, reload_on_window_close).chain())
    .run();
}

// // how many cells from the bottom to the top and from the left to the right
// const CELLS: u32 = 20;
// // TODO: should it also be rewritten?
// pub fn grid_texture() -> Image {
//   const WIDTH: usize = CELLS as usize
//     // left and right borders
//     + 2
//     // // borders between cells
//     // + (CELLS - 1)
//     ;

//   const TEXTURE_SIZE: usize =
//     // width
//     WIDTH
//       // height
//       * WIDTH
//       // rgba
//       * 4;

//   let mut texture_data = [0; TEXTURE_SIZE];

//   const WHITE_CELL: [u8; 4] = [255, 255, 255, 255];

//   let wall: Vec<u8> = WHITE_CELL.repeat(WIDTH);

//   let cells_start = WIDTH * 4;
//   let cells_end = TEXTURE_SIZE - (WIDTH * 4);

//   // top wall
//   texture_data[0..cells_start].copy_from_slice(&wall);
//   // bottom wall
//   texture_data[cells_end..].copy_from_slice(&wall);

//   // left and right walls
//   for idx in (cells_start..(cells_end + 4)).step_by(WIDTH * 4) {
//     texture_data[idx..(idx + 4)].copy_from_slice(&WHITE_CELL);
//     texture_data[(idx - 4)..((idx - 4) + 4)].copy_from_slice(&WHITE_CELL);

//     // --------------- inner walls ---------------
//     //  (enable borders between cells in WIDTH if its needed)
//     // let should_be_a_wall = (idx / (WIDTH * 4)) % 2 == 0;

//     // for rel_idx in (0..WIDTH * 4).step_by(4) {
//     //     if should_be_a_wall || (rel_idx / 4) % 2 == 0 {
//     //         texture_data[(idx + rel_idx)..(idx + rel_idx) + 4].copy_from_slice(&WHITE_CELL);
//     //     }
//     // }
//   }

//   Image::new_fill(
//     Extent3d {
//       width: WIDTH as u32,
//       height: WIDTH as u32,
//       depth_or_array_layers: 1,
//     },
//     TextureDimension::D2,
//     &texture_data,
//     TextureFormat::Rgba8UnormSrgb,
//     RenderAssetUsages::RENDER_WORLD,
//   )
// }

struct SingleThreaded(PhantomData<*const ()>);

fn reload_on_window_close(
  non_send: NonSend<SingleThreaded>,
  mut close_events: EventReader<WindowCloseRequested>,
  mut exit: EventWriter<AppExit>,
) {
  for _ in close_events.read() {
    unload_game();
    load_game();
  }
}

type Game = Module<ModuleExports>;

thread_local! {
  static GAME_INSTANCE: RefCell<(Option<Game>, bool)> = def();
}

fn load_game() {
  GAME_INSTANCE.with_borrow_mut(|(instance, _)| {
    let module = unsafe { load_module("target/debug/game.dll", init_imports) };
    let module: Game = module.unwrap();
    instance.replace(module);
  });
}

fn unload_game() {
  GAME_INSTANCE.with_borrow_mut(|(instance, setup_called)| {
    let game = instance.take().unwrap();
    game.unload().unwrap();
    *setup_called = false;
  });
}

fn call_game_export(call_: impl FnOnce(&ModuleExports)) {
  GAME_INSTANCE.with_borrow(|(game, _)| {
    call_(game.as_ref().unwrap().exports());
  });
}
