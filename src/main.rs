extern crate rand;
extern crate sdl2;

use core::arch::x86_64::_rdtsc;
use rand::Rng;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::mem;
use std::ptr;

const FRAMEBUFFER_WIDTH: u32 = 2560;
const FRAMEBUFFER_HEIGHT: u32 = 1440;
const BYTES_PER_PIXEL: u8 = 4;
const TEXTURE_WIDTH: u8 = 64;
const TEXTURE_HEIGHT: u8 = 64;
const TEXTURE_COUNT: usize = 1000;

struct PixelBuffer {
    buffer: Vec<u8>,
    bytes_per_pixel: u8,
    pitch: u32,
    width_in_pixels: u32,
    height_in_pixels: u32,
}

impl PixelBuffer {
    fn render_by_pixels(&mut self, dest: &mut PixelBuffer, pos_x: u32, pos_y: u32) {
        let mut src_row: *mut u8 = self.buffer.as_mut_ptr();
        let mut dest_row: *mut u8 = dest.buffer.as_mut_ptr();
        unsafe { dest_row = dest_row.offset((pos_y * dest.pitch) as isize) };
        for _y in 0..self.height_in_pixels {
            let mut src_pixel: *mut u32 = src_row as *mut u32;
            let mut dest_pixel: *mut u32 = dest_row as *mut u32;
            unsafe { dest_pixel = dest_pixel.offset(pos_x as isize) };
            for _x in 0..self.width_in_pixels {
                unsafe {
                    *dest_pixel = *src_pixel;
                    src_pixel = src_pixel.offset(1);
                    dest_pixel = dest_pixel.offset(1);
                }
            }
            unsafe {
                src_row = src_row.offset(self.pitch as isize);
                dest_row = dest_row.offset(dest.pitch as isize);
            }
        }
    }

    fn render_by_blocks(&mut self, dest: &mut PixelBuffer, pos_x: u32, pos_y: u32) {
        if self.buffer.len() == dest.buffer.len() && pos_x == 0 && pos_y == 0 {
            unsafe {
                ptr::copy_nonoverlapping(
                    self.buffer.as_ptr(),
                    dest.buffer.as_mut_ptr(),
                    dest.buffer.len(),
                );
            }
        } else {
            let mut src_row: *mut u8 = self.buffer.as_mut_ptr();
            let mut dest_row: *mut u8 = dest.buffer.as_mut_ptr();
            unsafe { dest_row = dest_row.offset((pos_y * dest.pitch) as isize) };
            for _y in 0..self.height_in_pixels {
                let mut dest_pixel: *mut u32 = dest_row as *mut u32;
                unsafe {
                    dest_pixel = dest_pixel.offset(pos_x as isize);
                    ptr::copy_nonoverlapping(src_row, dest_pixel as *mut u8, self.pitch as usize);
                    src_row = src_row.offset(self.pitch as isize);
                    dest_row = dest_row.offset(dest.pitch as isize);
                }
            }
        }
    }
}

fn create_pixel_buffer(width: u32, height: u32, bytes_per_pixel: u8) -> PixelBuffer {
    let pitch = bytes_per_pixel as u32 * width;
    let buffer_size: usize = bytes_per_pixel as usize * (width * height) as usize;
    let buffer = vec![0xFFu8; buffer_size];
    return PixelBuffer {
        buffer: buffer,
        bytes_per_pixel: bytes_per_pixel,
        pitch: pitch,
        width_in_pixels: width,
        height_in_pixels: height,
    };
}

fn render_weird_gradient(b: &mut PixelBuffer) {
    let mut row: *mut u8 = b.buffer.as_mut_ptr();
    for y in 0..FRAMEBUFFER_HEIGHT {
        let mut pixel: *mut u32 = row as *mut u32;
        for x in 0..FRAMEBUFFER_WIDTH {
            let pixel_bytes: [u8; 4] = [y as u8, x as u8, (x * y) as u8, 0xFF];
            unsafe {
                *pixel = mem::transmute::<[u8; 4], u32>(pixel_bytes);
                //*pixel = 0xFF << 24 | 0x00 << 16 | x << 8 | y; <- this produces slightly distorted result
                pixel = pixel.offset(1);
            }
        }
        unsafe {
            row = row.offset(b.pitch as isize);
        }
    }
}

fn render_color(b: &mut PixelBuffer, color: u32) {
    let mut row: *mut u8 = b.buffer.as_mut_ptr();
    for _y in 0..b.height_in_pixels {
        let mut pixel: *mut u32 = row as *mut u32;
        for _x in 0..b.width_in_pixels {
            unsafe {
                *pixel = color;
                pixel = pixel.offset(1);
            }
        }
        unsafe {
            row = row.offset(b.pitch as isize);
        }
    }
}

fn main() -> Result<(), String> {
    let mut rng = rand::thread_rng();
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let time = sdl_context.timer()?;

    let window = video_subsystem
        .window("sdl2-test", 0, 0)
        //.position_centered()
        //.resizable()
        .fullscreen_desktop()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas
        .set_logical_size(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT)
        .unwrap();
    let texture_creator = canvas.texture_creator();

    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::ARGB8888,
            FRAMEBUFFER_WIDTH,
            FRAMEBUFFER_HEIGHT,
        )
        .map_err(|e| e.to_string())?;

    let mut framebuffer =
        create_pixel_buffer(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, BYTES_PER_PIXEL);

    let mut event_pump = sdl_context.event_pump()?;

    let perf_count_frequency = time.performance_frequency();
    let mut last_perf_counter = time.performance_counter();

    let mut background_buffer =
        create_pixel_buffer(FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, BYTES_PER_PIXEL);
    render_weird_gradient(&mut background_buffer);

    let mut test_textures: Vec<PixelBuffer> = Vec::new();
    for x in 0..TEXTURE_COUNT {
        let mut pbuffer =
            create_pixel_buffer(TEXTURE_WIDTH as u32, TEXTURE_HEIGHT as u32, BYTES_PER_PIXEL);
        render_color(&mut pbuffer, x as u32);
        test_textures.push(pbuffer);
    }

    let mut last_cycle_count = unsafe { _rdtsc() };
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        background_buffer.render_by_blocks(&mut framebuffer, 0, 0);

        for t in &mut test_textures {
            let x = rng.gen_range(0, FRAMEBUFFER_WIDTH - TEXTURE_WIDTH as u32);
            let y = rng.gen_range(0, FRAMEBUFFER_HEIGHT - TEXTURE_HEIGHT as u32);
            t.render_by_blocks(&mut framebuffer, x, y);
        }

        texture
            .update(
                None,
                &mut framebuffer.buffer[..],
                framebuffer.pitch as usize,
            )
            .unwrap();

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        let end_cycle_count = unsafe { _rdtsc() };
        let end_perf_counter = time.performance_counter();

        let cycles_elapsed = end_cycle_count - last_cycle_count;
        let counter_elapsed = end_perf_counter - last_perf_counter;
        let ms_per_frame = (1000.0 * counter_elapsed as f32) / perf_count_frequency as f32; // How many milliseconds elapsed
        let fps = perf_count_frequency as f32 / counter_elapsed as f32;
        let mega_cycles_per_frame = cycles_elapsed as f32 / (1000 * 1000) as f32;

        println!(
            "ms_per_frame={}, fps={}, mc/f={}",
            ms_per_frame, fps, mega_cycles_per_frame
        );

        last_cycle_count = end_cycle_count;
        last_perf_counter = end_perf_counter;
    }

    Ok(())
}
