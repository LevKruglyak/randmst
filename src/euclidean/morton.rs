fn space_bits_2(x: u32) -> u32 {
    let (b_1, b_2, b_4, b_8) = (
        0b01010101_01010101_01010101_01010101,
        0b00110011_00110011_00110011_00110011,
        0b00001111_00001111_00001111_00001111,
        0b00000000_11111111_00000000_11111111,
    );
    let x = (x | x << 8) & b_8;
    let x = (x | x << 4) & b_4;
    let x = (x | x << 2) & b_2;
    let x = (x | x << 1) & b_1;
    x
}

fn space_bits_3(x: u32) -> u32 {
    let (b_2, b_4, b_8) = (
        0b00000000_001001_001001_001001_001001,
        0b00000000_000011_000011_000011_000011,
        0b00000000_000000_001111_000000_001111,
    );
    let x = (x | x << 8) & b_8;
    let x = (x | x << 4) & b_4;
    let x = (x | x << 2) & b_2;
    x
}

fn space_bits_4(x: u32) -> u32 {
    let (b_3, b_6, b_12) = (
        0b00010001_00010001_00010001_00010001,
        0b00000011_00000011_00000011_00000011,
        0b00000000_00001111_00000000_00001111,
    );
    let x = (x | x << 12) & b_12;
    let x = (x | x << 6) & b_6;
    let x = (x | x << 3) & b_3;
    x
}

pub fn morton_encode_2(point: [u8; 2]) -> u32 {
    space_bits_2(point[0] as u32) | space_bits_2(point[1] as u32) << 1
}

pub fn morton_encode_3(point: [u8; 3]) -> u32 {
    space_bits_3(point[0] as u32)
        | space_bits_3(point[1] as u32) << 1
        | space_bits_3(point[2] as u32) << 2
}

pub fn morton_encode_4(point: [u8; 4]) -> u32 {
    space_bits_4(point[0] as u32)
        | space_bits_4(point[1] as u32) << 1
        | space_bits_4(point[2] as u32) << 2
        | space_bits_4(point[3] as u32) << 3
}

pub trait Morton {
    fn morton_encode(&self) -> u32;
}

#[cfg(test)]
mod tests {}
