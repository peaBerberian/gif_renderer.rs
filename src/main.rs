// use lzw::{Decoder,LsbReader};

const HEADER_SIZE : usize = 13;

const IMAGE_DESCRIPTOR_BLOCK_ID : u8 = 0x2C;
const TRAILER_BLOCK_ID : u8 = 0x3B;

// struct LEReader {
//     buf : Vec<u8>,
//     idx : usize,
// }

// struct GifReader<'a> {
//     buf : &'a[u8],
//     pos : usize,
// }
// struct GifReader<'a> {
//     buf : &'a[u8],
// }

// struct GifReader<T : AsRef<[T]>> {
//     buf : T,
//     pos : usize,
// }
struct GifReader {
    buf : Vec<u8>,
    pos : usize,
}
impl GifReader {
    fn new(buf : Vec<u8>) -> GifReader {
        GifReader {
            buf,
            pos: 0,
        }
    }

    fn read_str(&mut self, nb_bytes : usize) -> Result<&str, std::str::Utf8Error> {
        use std::str;
        let end = self.pos + nb_bytes;
        let data = &self.buf[self.pos..end];
        self.pos += nb_bytes;
        str::from_utf8(data)
    }

    fn read_u16(&mut self) -> u16 {
        let val = self.buf[self.pos] as u16 |
            ((self.buf[self.pos + 1] as u16) << 8);
        self.pos += 2;
        val
    }

    fn read_u8(&mut self) -> u8 {
        let val = self.buf[self.pos];
        self.pos += 1;
        val
    }

    fn read_slice(&mut self, nb_bytes : usize) -> &[u8] {
        let end = self.pos + nb_bytes;
        let val = &self.buf[self.pos..end];
        self.pos += nb_bytes;
        val
    }

    // fn skip(&mut self, nb_bytes : usize) {
    //     self.pos += nb_bytes;
    // }

    fn get_pos(&self) -> usize {
        self.pos
    }

    fn bytes_left(&self) -> usize {
        self.buf.len() - self.pos
    }

    // fn read_u16(&mut self) -> u16 {
    //     let val = self.buf[self.idx] as u16 |
    //         ((self.buf[self.idx + 1] as u16) << 8);
    //     self.idx += 2;
    //     val
    // }

    // fn read_u8(&mut self) -> u8 {
    //     let val = self.buf[self.idx];
    //     self.idx += 1;

    // }
}
// impl<'a> GifReader<'a> {
//     fn new(vec : Vec<u8>) -> GifReader<'a> {
//         let buf = &vec;
//         GifReader {
//             buf,
//         }
//     }

//     fn read_str(&mut self, nb_bytes : usize) -> Result<&str, std::str::Utf8Error> {
//         use std::str;
//         let data = &self.buf[0..nb_bytes];
//         self.buf = &self.buf[nb_bytes..];
//         str::from_utf8(data)
//     }

//     fn read_u16(&mut self) -> u16 {
//         let val = self.buf[0] as u16 |
//             ((self.buf[0 + 1] as u16) << 8);
//         self.buf = &self.buf[2..];
//         val
//     }

//     fn read_u8(&mut self) -> u8 {
//         let val = self.buf[0];
//         self.buf = &self.buf[1..];
//         val
//     }

//     fn skip(&mut self, nb_bytes : usize) {
//         self.buf = &self.buf[nb_bytes..];
//     }

//     fn bytes_left(&self) -> usize {
//         self.buf.len()
//     }

//     // fn read_u16(&mut self) -> u16 {
//     //     let val = self.buf[self.idx] as u16 |
//     //         ((self.buf[self.idx + 1] as u16) << 8);
//     //     self.idx += 2;
//     //     val
//     // }

//     // fn read_u8(&mut self) -> u8 {
//     //     let val = self.buf[self.idx];
//     //     self.idx += 1;

//     // }
// }

fn main() {
    let file_data = std::fs::read("./z.gif").unwrap();
    if file_data.len() < HEADER_SIZE {
        panic!("Invalid GIF file: too short");
    }
    let mut rdr = GifReader::new(file_data);
    let header = parse_header(&mut rdr);
    println!("{}x{} {:?}", header.height, header.width, header.global_color_table);

    while rdr.bytes_left() > 0 {
        match rdr.read_u8() {
            IMAGE_DESCRIPTOR_BLOCK_ID => {
                println!("IT'S A BLOCK {}", rdr.get_pos());
                parse_image_descriptor(&mut rdr);
                return;
            }
            TRAILER_BLOCK_ID => {
                println!("IT'S A TRAILER {}", rdr.get_pos());
                // return ();
            }
            _ => {
                println!("KEZAKO {}", rdr.get_pos());
            }
        }
    }
}

fn parse_image_descriptor(rdr : &mut GifReader) {
    let image_left_position = rdr.read_u16();
    let image_top_position = rdr.read_u16();
    let image_width = rdr.read_u16();
    let image_height = rdr.read_u16();
    let field = rdr.read_u8();
    println!("{}, {}, {}, {}",
        image_left_position, image_top_position, image_width, image_height);

    let has_local_color_table = field & 0x80 != 0;
    let has_interlacing = field & 0x40 != 0;
    let is_sorted = field & 0x20 != 0;
    let _reserved_1 = field & 0x10;
    let _reserved_2 = field & 0x08;
    let nb_entries : usize = 1 << ((field & 0x07) + 1);

    let lct = if has_local_color_table {
        Some(parse_color_table(rdr, nb_entries))
    } else { None };

    println!("lt{}, i{}, s{}, {} {:?}",
        has_local_color_table, has_interlacing, is_sorted, nb_entries, lct);

    let initial_code_size = rdr.read_u8();
    println!("aaa {}", initial_code_size);

    // TODO Remove
    let mut whole_data : Vec<u8> = vec![];
    loop {
        if rdr.bytes_left() <= 0 {
            panic!("Invalid GIF File: Image Descriptor Truncated");
        }

        let sub_block_size = rdr.read_u8() as usize;
        if sub_block_size == 0 {
            println!("Aended {}", whole_data.len());
            println!("{:?}", whole_data);
            let mut decoder = Decoder::new(initial_code_size);
            let mut compressed = &whole_data[..];
            let mut data2 = vec![];
            while compressed.len() > 0 {
                let (start, bytes) = decoder.decode_bytes(&whole_data).unwrap();
                compressed = &compressed[start..];
                println!("OKX{:?}", bytes);
                data2.extend(bytes.iter().map(|&i| i));
            }
            println!("{}x{} {:?} - {:?}",
                image_height, image_width, data2, data2.len());
            return ;
        }
        if rdr.bytes_left() <= sub_block_size {
            panic!("Invalid GIF File: Image Descriptor Truncated");
        }
        whole_data.extend(rdr.read_slice(sub_block_size));
    }
}
// fn parse_image_descriptor(buf : &Vec<u8>, idx : usize) -> usize {
//     let image_left_position = le_buf_to_u16(buf, idx + 1);
//     let image_top_position = le_buf_to_u16(buf, idx + 3);
//     let image_width = le_buf_to_u16(buf, idx + 5);
//     let image_height = le_buf_to_u16(buf, idx + 7);
//     let field = buf[idx + 9];
//     println!("{}, {}, {}, {}",
//         image_left_position, image_top_position, image_width, image_height);

//     let has_local_color_table = field & 0x80 != 0;
//     let has_interlacing = field & 0x40 != 0;
//     let is_sorted = field & 0x20 != 0;
//     let _reserved_1 = field & 0x10;
//     let _reserved_2 = field & 0x08;
//     let nb_entries : usize = 1 << ((field & 0x07) + 1);

//     let (code_size_idx, lct) = if has_local_color_table {
//         (
//             idx + 10 + (nb_entries * 3),
//             Some(parse_color_table(buf, nb_entries))
//         )
//     } else { (idx + 10, None) };

//     println!("lt{}, i{}, s{}, {} {:?} {} ",
//         has_local_color_table, has_interlacing, is_sorted, nb_entries, lct, code_size_idx);

//     let initial_code_size = buf[code_size_idx];
//     println!("aaa {} {}", initial_code_size, code_size_idx + 1);
//     let mut curr_index = code_size_idx + 1;

//     // TODO Remove
//     let mut whole_data : Vec<u8> = vec![];
//     loop {
//         if buf.len() <= curr_index {
//             panic!("Invalid GIF File: Image Descriptor Truncated");
//         }

//         let sub_block_size = buf[curr_index] as usize;
//         if sub_block_size == 0 {
//             println!("Aended {}", whole_data.len());
//             println!("{:?}", whole_data);
//             let mut decoder = Decoder::new(initial_code_size);
//             let mut compressed = &whole_data[..];
//             let mut data2 = vec![];
//             while compressed.len() > 0 {
//                 let (start, bytes) = decoder.decode_bytes(&whole_data).unwrap();
//                 compressed = &compressed[start..];
//                 println!("OKX{:?}", bytes);
//                 data2.extend(bytes.iter().map(|&i| i));
//             }
//             println!("{}x{} {:?} - {:?}",
//                 image_height, image_width, image_height, data2.len());
//             return curr_index + 1;
//         }
//         if buf.len() <= curr_index + sub_block_size {
//             panic!("Invalid GIF File: Image Descriptor Truncated");
//         }
//         let sub_block_start : usize = curr_index + 1;
//         let sub_block_end : usize = sub_block_start + sub_block_size;
//         whole_data.extend(&buf[sub_block_start..sub_block_end]);
//         curr_index = sub_block_end;
//     }
// }

#[derive(Debug)]
struct GifHeader {
    width : u16,
    height : u16,
    nb_color_resolution_bits : u8,
    is_table_sorted : bool,
    background_color_index : u8,
    pixel_aspect_ratio : u8,
    global_color_table : Option<Vec<RGB>>,
}
// #[derive(Debug)]
// struct GifHeader {
//     width : u16,
//     height : u16,
//     nb_color_resolution_bits : u8,
//     is_table_sorted : bool,
//     background_color_index : u8,
//     pixel_aspect_ratio : u8,
//     global_color_table : Option<Vec<RGB>>,
//     first_block_index : usize,
// }

#[derive(Debug, Clone)]
struct RGB {
    r : u8,
    g : u8,
    b : u8,
}

// TODO use C repr to parse it more rapidly?
// fn parse_color_table(buf : &Vec<u8>, nb_entries : usize) -> Vec<RGB> {
//     let ct_size : usize = nb_entries * 3;
//     if buf.len() < HEADER_SIZE + ct_size  {
//         panic!("Invalid GIF file: truncated color table");
//     }
//     let mut ct : Vec<RGB> = vec![RGB { r: 0, g: 0, b: 0}; nb_entries as usize];
//     let mut curr_index = HEADER_SIZE;
//     for curr_elt_idx in 0..(nb_entries) {
//         ct[curr_elt_idx as usize] = RGB {
//             r: buf[curr_index],
//             g: buf[curr_index + 1],
//             b: buf[curr_index + 2],
//         };
//         curr_index += 3;
//     }
//     ct
// }
fn parse_color_table(rdr : &mut GifReader, nb_entries : usize) -> Vec<RGB> {
    let ct_size : usize = nb_entries * 3;
    if rdr.bytes_left() < ct_size  {
        panic!("Invalid GIF file: truncated color table");
    }
    let mut ct : Vec<RGB> = vec![RGB { r: 0, g: 0, b: 0}; nb_entries as usize];
    for curr_elt_idx in 0..(nb_entries) {
        ct[curr_elt_idx as usize] = RGB {
            r: rdr.read_u8(),
            g: rdr.read_u8(),
            b: rdr.read_u8(),
        };
    }
    ct
}

fn parse_header(rdr : &mut GifReader) -> GifHeader {
    match rdr.read_str(3) {
        Err(e) => panic!("Invalid GIF file:
            Impossible to read the header, obtained: {}.", e),
        Ok(x) if x != "GIF" => panic!("Invalid GIF file: Missing GIF header."),
        _ => {}
    }

    match rdr.read_str(3) {
        Err(x) => panic!("Impossible to parse the version: {}.", x),
        Ok(v) if v != "89a" && v != "87a" => panic!("Unmanaged version: {}", v),
        _ => {}
    }

    let width = rdr.read_u16();
    let height = rdr.read_u16();

    let field = rdr.read_u8();
    let has_global_color_table = field & 0x80 != 0;
    let nb_color_resolution_bits = ((field & 0x70) >> 4) + 1;
    let is_table_sorted = field & 0x08 != 0;
    println!("nba {}", field & 0x07);
    let nb_entries : usize = 1 << ((field & 0x07) + 1);

    let background_color_index = rdr.read_u8();
    let pixel_aspect_ratio = rdr.read_u8();

    let gct = if has_global_color_table {
        Some(parse_color_table(rdr, nb_entries))
    } else {
        None
    };

    GifHeader {
        width,
        height,
        nb_color_resolution_bits,
        is_table_sorted,
        background_color_index,
        pixel_aspect_ratio,
        global_color_table: gct,
    }
}
// fn parse_header(buf : &Vec<u8>) -> GifHeader {
//     use std::str;

//     let id_tag = &buf[0..3];
//     match str::from_utf8(id_tag) {
//         Err(e) => panic!("Invalid GIF file:
//             Impossible to read the header, obtained: {}.", e),
//         Ok(x) if x != "GIF" => panic!("Invalid GIF file: Missing GIF header."),
//         _ => {}
//     }

//     let version = &buf[3..6];
//     match str::from_utf8(version) {
//         Err(x) => panic!("Impossible to parse the version: {}.", x),
//         Ok(v) if v != "89a" && v != "87a" => panic!("Unmanaged version: {}", v),
//         _ => {}
//     }

//     let width = le_buf_to_u16(buf, 6);
//     let height = le_buf_to_u16(buf, 8);
//     let field = buf[10];
//     let has_global_color_table = field & 0x80 != 0;
//     let nb_color_resolution_bits = ((field & 0x70) >> 4) + 1;
//     let is_table_sorted = field & 0x08 != 0;
//     println!("nba {}", field & 0x07);
//     let nb_entries : usize = 1 << ((field & 0x07) + 1);
//     let background_color_index = buf[11];
//     let pixel_aspect_ratio = buf[12];
//     println!("nb {}", nb_entries);
//     let (first_block_index, gct) = if has_global_color_table {
//         (
//             HEADER_SIZE + (nb_entries * 3),
//             Some(parse_color_table(buf, nb_entries))
//         )
//     } else { (HEADER_SIZE, None) };

//     GifHeader {
//         width,
//         height,
//         nb_color_resolution_bits,
//         is_table_sorted,
//         background_color_index,
//         pixel_aspect_ratio,
//         global_color_table: gct,
//         first_block_index,
//     }
// }

// use std::collections::HashMap;

// #[derive(Debug)]
// struct DecodingDict {
//     table: HashMap<u16, Vec<u8>>,
//     buffer: Vec<u8>,
// }

// fn get_dict(initial_code_size : u16) {
//     let mut dict : Vec = HashMap.new();
//     let last_code = 1u16 << initial_code_size;
//     for i in 0..initial_code_size {
//         dict
//     }
// }

/// Convert two bytes from a buffer in little-endian to an u16
/// /!\ Perform no bound checking. Will panic if the length of the vector minus
/// the index given equals less than two.
// fn le_buf_to_u16(buf : &Vec<u8>,
//                  idx : usize) -> u16 {
//     buf[idx] as u16 + ((buf[idx + 1] as u16) << 8)
// }

// fn _le_buf_to_u32(buf : &Vec<u8>,
//                  idx : usize) -> u32 {
//     buf[idx] as u32 +
//         ((buf[idx + 1] as u32) << 8) +
//         ((buf[idx + 1] as u32) << 16) +
//         ((buf[idx + 1] as u32) << 24)
// }

/// Alias for a LZW code point
type Code = u16;
const MAX_CODESIZE: u8 = 12;
const MAX_ENTRIES: usize = 1 << MAX_CODESIZE as usize;

#[derive(Debug)]
struct DecodingDict {
    min_size: u8,
    table: Vec<(Option<Code>, u8)>,
    buffer: Vec<u8>,
}

use std::io;

impl DecodingDict {
    /// Creates a new dict
    fn new(min_size: u8) -> DecodingDict {
        DecodingDict {
            min_size: min_size,
            table: Vec::with_capacity(512),
            buffer: Vec::with_capacity((1 << MAX_CODESIZE as usize) - 1)
        }
    }

    /// Resets the dictionary
    fn reset(&mut self) {
        self.table.clear();
        for i in 0..(1u16 << self.min_size as usize) {
            self.table.push((None, i as u8));
        }
    }

    /// Inserts a value into the dict
    #[inline(always)]
    fn push(&mut self, key: Option<Code>, value: u8) {
        self.table.push((key, value))
    }

    /// Reconstructs the data for the corresponding code
    fn reconstruct(&mut self, code: Option<Code>) -> io::Result<&[u8]> {
        self.buffer.clear();
        let mut code = code;
        let mut cha;
        // Check the first access more thoroughly since a bad code
        // could occur if the data is malformed
        if let Some(k) = code {
            match self.table.get(k as usize) {
                Some(&(code_, cha_)) => {
                    code = code_;
                    cha = cha_;
                }
                None => return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    &*format!("Invalid code {:X}, expected code <= {:X}", k, self.table.len())
                ))
            }
            self.buffer.push(cha);
        }
        while let Some(k) = code {
            if self.buffer.len() >= MAX_ENTRIES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Invalid code sequence. Cycle in decoding table."
                ))
            }
            //(code, cha) = self.table[k as usize];
            // Note: This could possibly be replaced with an unchecked array access if
            //  - value is asserted to be < self.next_code() in push
            //  - min_size is asserted to be < MAX_CODESIZE
            let entry = self.table[k as usize]; code = entry.0; cha = entry.1;
            self.buffer.push(cha);
        }
        self.buffer.reverse();
        Ok(&self.buffer)
    }

    /// Returns the buffer constructed by the last reconstruction
    #[inline(always)]
    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    /// Number of entries in the dictionary
    #[inline(always)]
    fn next_code(&self) -> u16 {
        self.table.len() as u16
    }
}

#[derive(Debug)]
struct BitReader {
    bits: u8,
    acc: u32,
}

/// Containes either the consumed bytes and reconstructed bits or
/// only the consumed bytes if the supplied buffer was not bit enough
pub enum Bits {
    /// Consumed bytes, reconstructed bits
    Some(usize, u16),
    /// Consumed bytes
    None(usize),
}

impl BitReader {
    fn new() -> BitReader {
        BitReader {
            bits: 0,
            acc: 0,
        }
    }
    fn read_bits(&mut self, mut buf: &[u8], n: u8) -> Bits {
        println!("IN RB {:?} {} {} {}", buf, n, buf[0], buf[1]);
        if n > 16 {
            // This is a logic error the program should have prevented this
            // Ideally we would used bounded a integer value instead of u8
            panic!("Cannot read more than 16 bits")
        }
        let mut consumed = 0;
        while self.bits < n {
            let byte = if buf.len() > 0 {
                let byte = buf[0];
                buf = &buf[1..];
                byte
            } else {
                return Bits::None(consumed)
            };
            println!("v{}", byte);
            self.acc |= (byte as u32) << self.bits;
            println!("r{}", self.acc);
            self.bits += 8;
            consumed += 1;
        }
        println!("acc{} {}", self.acc, ((1 << n) - 1));
        let res = self.acc & ((1 << n) - 1);
        println!("res{}", res);
        self.acc >>= n;
        self.bits -= n;
        println!("acc{}", self.acc);
        println!("bits{}", self.bits);
        Bits::Some(consumed, res as u16)
    }
}

#[derive(Debug)]
struct Decoder {
    r: BitReader,
    prev: Option<Code>,
    table: DecodingDict,
    buf: [u8; 1],
    code_size: u8,
    min_code_size: u8,
    clear_code: Code,
    end_code: Code,
}

impl Decoder {
    /// Creates a new LZW decoder.
    pub fn new(min_code_size: u8) -> Decoder {
        Decoder {
            r: BitReader::new(),
            prev: None,
            table: DecodingDict::new(min_code_size),
            buf: [0; 1],
            code_size: min_code_size + 1,
            min_code_size: min_code_size,
            clear_code: 1 << min_code_size,
            end_code: (1 << min_code_size) + 1,
        }
    }

    /// Tries to obtain and decode a code word from `bytes`.
    ///
    /// Returns the number of bytes that have been consumed from `bytes`. An empty
    /// slice does not indicate `EOF`.
    pub fn decode_bytes(&mut self, bytes: &[u8]) -> io::Result<(usize, &[u8])> {
        println!("{}", self.clear_code);
        Ok(match self.r.read_bits(bytes, self.code_size) {
            Bits::Some(consumed, code) => {
                (consumed, if code == self.clear_code {
                    println!("CCC");
                    self.table.reset();
                    self.table.push(None, 0); // clear code
                    self.table.push(None, 0); // end code
                    self.code_size = self.min_code_size + 1;
                    self.prev = None;
                    &[]
                } else if code == self.end_code {
                    println!("DDD");
                    &[]
                } else {
                    println!("EEE");
                    let next_code = self.table.next_code();
                    if code > next_code {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            &*format!("Invalid code {:X}, expected code <= {:X}",
                                      code,
                                      next_code
                            )
                        ))
                    }
                    let prev = self.prev;
                    let result = if prev.is_none() {
                        println!("HERE {}", code);
                        self.buf = [code as u8];
                        &self.buf[..]
                    } else {
                        let data = if code == next_code {
                            println!("HERE2");
                            let cha = self.table.reconstruct(prev)?[0];
                            self.table.push(prev, cha);
                            self.table.reconstruct(Some(code))?
                        } else if code < next_code {
                            println!("HERE3 {}", code);
                            let cha = self.table.reconstruct(Some(code))?[0];
                            self.table.push(prev, cha);
                            self.table.buffer()
                        } else {
                            // code > next_code is already tested a few lines earlier
                            unreachable!()
                        };
                        data
                    };
                    if next_code == (1 << self.code_size as usize) - 1 - 0 // XXX TODO $offset
                       && self.code_size < MAX_CODESIZE {
                        self.code_size += 1;
                    }
                    self.prev = Some(code);
                    result
                })
            },
            Bits::None(consumed) => {
                println!("FFF");
                (consumed, &[])
            }
        })
    }
}

/// Convert two bytes from a buffer in little-endian to an u16
fn _le_buf_to_u16(buf : &[u8]) -> u16 {
    buf[0] as u16 | ((buf[1] as u16) << 8)
}

fn __le_buf_to_u32(buf : &[u8]) -> u32 {
    buf[0] as u32 |
        ((buf[1] as u32) << 8) |
        ((buf[2] as u32) << 16) |
        ((buf[3] as u32) << 24)
}

// /// Convert two bytes from a buffer in little-endian to an u16
// fn _le_buf_to_u16(buf : &[u8]) -> u16 {
//      match buf.len() {
//         0 => 0,
//         1 => buf[0] as u16,
//         _ => buf[0] as u16 + ((buf[1] as u16) << 8),
//      }
// }

// fn _le_buf_to_u32(buf : &[u8]) -> u32 {
//      match buf.len() {
//         0..=2 => le_buf_to_u16(buf) as u32,
//         3 => buf[0] as u32 +
//              ((buf[1] as u32) << 8) +
//              ((buf[2] as u32) << 16),
//         _ => buf[0] as u32 +
//              ((buf[1] as u32) << 8) +
//              ((buf[2] as u32) << 16) +
//              ((buf[3] as u32) << 24),
//      }
// }
