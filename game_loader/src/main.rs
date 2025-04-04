mod imports_impl;
mod live_reload;

use std::{
  cell::RefCell,
  sync::mpsc::{channel, Receiver},
};

use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use fk_core::def;
use imports_impl::{init_imports, ModuleExports};
use relib_host::{load_module, Module};
use live_reload::LiveReloadMessage;

fn main() {
  load_game();

  let (sender, receiver) = channel();
  std::thread::spawn(|| {
    live_reload::run_loop(sender);
  });

  thread_local! {
    static LIVE_RELOAD_RECEIVER: RefCell<Option<Receiver<LiveReloadMessage>>> = def();
  }

  LIVE_RELOAD_RECEIVER.replace(Some(receiver));

  let game_update = |world: &mut World| {
    let msg = LIVE_RELOAD_RECEIVER.with_borrow(|receiver| receiver.as_ref().unwrap().try_recv());
    if let Ok(msg) = msg {
      match msg {
        LiveReloadMessage::Success => {
          unload_game();
          load_game();
        }
        LiveReloadMessage::BuildFailure => {
          unload_game();
        }
      }
    }

    let game_is_loaded = GAME_INSTANCE.with_borrow(|(instance, _)| instance.is_some());
    if !game_is_loaded {
      return;
    }

    let setup_called = GAME_INSTANCE.with_borrow_mut(|(_, setup_called)| {
      let called = *setup_called;
      if !called {
        *setup_called = true;
      }
      called
    });

    if !setup_called {
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
    .add_plugins((
      DefaultPlugins.set(ImagePlugin::default_nearest()),
      WorldInspectorPlugin::default(),
    ))
    .add_systems(Update, game_update)
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
    let Some(game) = instance.take() else {
      return;
    };
    game.unload().unwrap();
    *setup_called = false;
  });
}

fn call_game_export(call_: impl FnOnce(&ModuleExports)) {
  GAME_INSTANCE.with_borrow(|(game, _)| {
    call_(game.as_ref().unwrap().exports());
  });
}
