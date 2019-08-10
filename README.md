# Efficient 2D software rendering in Rust

This repo is a workbench fr my personal studies in efficient 2d software rendering techniques.

Currently it contains a test program, which renders a background (with some psychedelic gradient)
+ n test textures that get rendered at a random position every frame.

The rendering is all done in software and the final framebuffer is blitted using SDL (this last part is hardware accelerated thanks to SDL2,
so blits are fast)).

## Optimizations done to improve rendering speed

### Manually managing memory for pixel buffers

I use `libc::malloc` to allocate n bytes of memory for the framebuffer and other pixel buffers.
As a result I get a raw pointer, which I traverse using pointer arithmetic when rendering.
Switching from using a &[u8] to raw pointers for pixel buffers resulted in about 30% better performance
in pixel by pixel rendering.

### Rendering textures in blocks instead of pixel by pixel

Another convenient result whe using pointers is that rendering can now be done using system level
memory copying functions. I decided to use `memcpy` from `libc`, because it's straight forward.
Instead rendering textures to framebuffer by copying pixel by pixel, I know memcpy each pixel row instead.
This leads to about 30x performance compared to pixel by pixel rendering.

## Some very non-scientfic performance results

On my personal computer, which has an AMD Ryzen 5 1600 running at 3.2 GhZ, rendering the background (which is the same size as the framebuffer)
+ 1000 test textures of size 64x64 each, results in the following FPS for each framebuffer size using release build:

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

[Handmade Hero](https://handmadehero.org/watch)
[Paper: Efficient 2D software rendering](https://www.researchgate.net/publication/271769119_Efficient_2D_software_rendering)
[SDL2 Migration Guide](https://wiki.libsdl.org/MigrationGuide)