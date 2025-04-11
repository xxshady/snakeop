live reload, bevy engine

https://github.com/user-attachments/assets/4370d70b-5dff-4bc0-be06-1d8e6f734f30

it's just a proof of concept, implementation is pretty clunky

how to run:
```txt
cargo build
cargo run --package game_loader
```

code changed in `game/src` will be automatically compiled and reloaded









if you see directx error spam (INVALID_SUBRESOURCE_STATE) use vulkan as backend instead: `WGPU_BACKEND=vulkan`
https://github.com/bevyengine/bevy/issues/14936#issuecomment-2508938673
