use xcap::image::{RgbImage, Rgb, RgbaImage};

pub fn downscale_frame(image: RgbaImage, size: (u32, u32)) -> RgbImage {
    let frame = image;

    let mut scaled_buffer: RgbImage = RgbImage::new(size.0, size.1);
    //  buffer for averaging pixel values together (count, R, G, B)
    let mut scaled_intermediate_buffer: Vec<Vec<(u32, u32, u32, u32)>> = vec![vec![(0,0,0,0); size.1 as usize]; size.0 as usize];

    for (x, y, pixel) in frame.enumerate_pixels() {
        let destination = downscale_to(x, y, frame.dimensions(), (size.0, size.1));

        let value = &mut scaled_intermediate_buffer[destination.0 as usize][destination.1 as usize];

        value.0 += 1;
        value.1 += pixel.0[0] as u32;
        value.2 += pixel.0[1] as u32;
        value.3 += pixel.0[2] as u32;
    }

    for x in 0..scaled_intermediate_buffer.len() {
        for y in 0..scaled_intermediate_buffer[x].len() {
            scaled_buffer.put_pixel(x as u32, y as u32, 
                Rgb([
                    (scaled_intermediate_buffer[x][y].1 / scaled_intermediate_buffer[x][y].0) as u8,
                    (scaled_intermediate_buffer[x][y].2 / scaled_intermediate_buffer[x][y].0) as u8,
                    (scaled_intermediate_buffer[x][y].3 / scaled_intermediate_buffer[x][y].0) as u8,
                ])
            );
        }
    }

    scaled_buffer
}

fn downscale_to(x: u32, y: u32, source: (u32, u32), destination: (u32, u32)) -> (u32, u32) {
    let ratio = (
        destination.0 as f64 / source.0 as f64,
        destination.1 as f64 / source.1 as f64
    );
    (
        (ratio.0 * x as f64) as u32, 
        (ratio.1 * y as f64) as u32
    )
}