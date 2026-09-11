pub fn downscale_frame(image: Vec<u8>, source_size: (u32, u32), dest_size: (u32, u32)) -> Vec<u8> {
    let frame = image;

    //  buffer for averaging pixel values together (count, R, G, B)
    let mut scaled_intermediate_buffer: Vec<Vec<(u32, u32, u32, u32)>> = vec![vec![(0,0,0,0); dest_size.1 as usize]; dest_size.0 as usize];

    //  source is raw RGBA, row major
    for y in 0..source_size.1 {
        for x in 0..source_size.0 {
            let pixel = ((y as usize * source_size.0 as usize) + x as usize) * 4;

            let destination = downscale_to(x, y, source_size, dest_size);

            let value = &mut scaled_intermediate_buffer[destination.0 as usize][destination.1 as usize];

            value.0 += 1;
            value.1 += frame[pixel] as u32;
            value.2 += frame[pixel + 1] as u32;
            value.3 += frame[pixel + 2] as u32;
        }
    }

    //  flattened RGBA bytes, row major
    let mut scaled_buffer: Vec<u8> = Vec::with_capacity(dest_size.0 as usize * dest_size.1 as usize * 4);

    for y in 0..dest_size.1 as usize {
        for x in 0..dest_size.0 as usize {
            let value = scaled_intermediate_buffer[x][y];

            scaled_buffer.push((value.1 / value.0) as u8);
            scaled_buffer.push((value.2 / value.0) as u8);
            scaled_buffer.push((value.3 / value.0) as u8);
            scaled_buffer.push(255);
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
