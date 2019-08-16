# Efficient 2D software rendering in Rust

This repo is a workbench for my personal studies in efficient 2d software rendering techniques.

Currently it contains a test program, which renders a background (with some psychedelic gradient)
and n test textures that get rendered at a random position every frame.

The rendering is all done in software and the final framebuffer is blitted using SDL (this last part is hardware accelerated thanks to SDL2,
so blits are fast)).

## Optimizations done to improve rendering speed

### Rendering textures in blocks instead of pixel by pixel

Block copying is implemented by using raw pointers and `std::ptr::copy_nonoverlapping` function, which semantically same as C's `memcpy` function.

## Some very non-scientfic performance results

On my personal computer, which has an AMD Ryzen 5 1600 running at 3.2 GhZ, rendering the background (which is the same size as the framebuffer)
and 1000 test textures of size 64x64 each, results in the following FPS for each framebuffer size using release build:

| Framebuffer size (resolution) | FPS       |
| ----------------------------- | ---------:|
| 3840 x 2160                   | 75        |
| 2560 x 1440                   | 112       |
| 1920 x 1080                   | 170       |
| 1280 x 720                    | 380       |

Things to consider: these are solid textures (no empty pixels or holes),
so there's yet no need for texture preprocessing and a little bit more complicated rendering pipeline.
In other words, rendering solid textures is pretty straight forward.

## TODO

- rendering of non-solid textures (with empty pixels and holes)
- alpha blending
- texture preprocessing to ensure that rendering of non-solid and non-opaque textures is efficient
- multithreaded rendering
- a little bit more scientific results :)

## Resources

- [Handmade Hero](https://handmadehero.org/watch)
- [Paper: Efficient 2D software rendering](https://www.researchgate.net/publication/271769119_Efficient_2D_software_rendering)
- [SDL2 Migration Guide](https://wiki.libsdl.org/MigrationGuide)