
//
// utils
//

/*
    utf-8 encoding:
    #bytes  #bits  code-point  encoding
         1      7      U+0000  0|0000000
                       U+007F  0|1111111
         2     11      U+0080  110|00010 10|000000
                       U+07FF  110|11111 10|111111
         3     16      U+0800  1110|0000 10|100000 10|000000
                       U+D7FF  1110|1101 10|011111 10|111111
                       U+E000  1110|1110 10|000000 10|000000
                       U+FFFF  1110|1111 10|111111 10|111111
         4     21    U+010000  11110|000 10|010000 10|000000 10|000000
                     U+10FFFF  11110|100 10|001111 10|111111 10|111111
*/

#[inline(always)]
pub fn is_boundary(b: u8) -> bool {
    //     b <  0b10000000 = 128 ~= -128
    // ||  b >= 0b11000000 = 196 ~=  -64
    //
    //    (b < -128 || b >= -64)
    // == !(b >= -128 && b < -64)
    // == !(true && b < -64)
    // == b >= -64
    (b as i8) >= -64
}

#[inline(always)]
pub fn is_continuation(b: u8) -> bool {
    !is_boundary(b)
}


#[inline(always)]
pub fn is_ascii(b: u8) -> bool {
    b < 0x80
}



//
// validation
//

pub struct Utf8Error {
    pub valid_until: *const u8,
}

/// check one utf-8 encoded codepoint.
/// - assumes `buffer.len() > 0`.
/// - on success, returns the remaining buffer after the codepoint.
#[inline]
pub fn check_1(buffer: &[u8]) -> Result<&[u8], ()> {
    let b = buffer;
    match b[0] {
        // 0|0000000
        // 0|1111111
        0b0_0000000 ..= 0b0_1111111 => {
            Ok(&b[1..])
        }

        // denormalized ascii.
        0b1_0000000 ..= 0b110_00001 => {
            Err(())
        }

        // 110|00010 10|000000
        // 110|11111 10|111111
        0b110_00010 ..= 0b110_11111 => {
            if b.len() < 2
            || !is_continuation(b[1]) {
                return Err(());
            }
            Ok(&b[2..])
        }

        // 1110|0000 10|100000 10|000000
        // 1110|0000 10|111111 10|111111
        0b1110_0000 => {
            if b.len() < 3
            || !is_continuation(b[1])
            || b[1] < 0b10_100000
            || !is_continuation(b[2]) {
                return Err(());
            }
            Ok(&b[3..])
        }

        // 1110|0001 10|000000 10|000000
        // 1110|1101 10|011111 10|111111
        0b1110_0001 ..= 0b1110_1101 => {
            if b.len() < 3
            || !is_continuation(b[1])
            || b[1] > 0b10_011111
            || !is_continuation(b[2]) {
                return Err(());
            }
            Ok(&b[3..])
        }

        // 1110|1110 10|000000 10|000000
        // 1110|1111 10|111111 10|111111
        0b1110_1110 ..= 0b1110_1111 => {
            if b.len() < 3
            || !is_continuation(b[1])
            || !is_continuation(b[2]) {
                return Err(());
            }
            Ok(&b[3..])
        }

        // 11110|000 10|010000 10|000000 10|000000
        // 11110|000 10|111111 10|111111 10|111111
        0b11110_000 => {
            if b.len() < 4
            || !is_continuation(b[1])
            || b[1] < 0b10_010000
            || !is_continuation(b[2])
            || !is_continuation(b[3]) {
                return Err(());
            }
            Ok(&b[4..])
        }

        // 11110|001 10|000000 10|000000 10|000000
        // 11110|100 10|001111 10|111111 10|111111
        0b11110_001 ..= 0b11110_100 => {
            if b.len() < 4
            || !is_continuation(b[1])
            || b[1] > 0b10_001111
            || !is_continuation(b[2])
            || !is_continuation(b[3]) {
                return Err(());
            }
            Ok(&b[4..])
        }

        // 11110|101 *
        // 11111 111 *
        0b11110_101 ..= 0b11111_111 => {
            Err(())
        }
    }
}


