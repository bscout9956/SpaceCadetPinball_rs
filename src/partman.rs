use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryData, FieldTypes, GroupData};
use crate::pb::FULL_TILT_MODE;
use crate::utils;
use num_traits::{FromPrimitive, ToPrimitive};
use sdl2::log::Category::Assert;
use std::ffi::{CStr, c_char};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::{BitAnd, BitOr};
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{LazyLock, Mutex};
use utils::LRead;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Bmp8Flags(u8);

impl Bmp8Flags {
    pub const RAW_BMP_UNALIGNED: Self = Self(1 << 0);
    pub const DIB_BITMAP: Self = Self(1 << 1);
    pub const SPLICED: Self = Self(1 << 2);
}

impl BitOr for Bmp8Flags {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for Bmp8Flags {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(packed)]
pub struct DatFileHeader {
    pub file_signature: [u8; 21],
    pub app_name: [u8; 50],
    pub description: [u8; 100],
    pub file_size: i32,
    pub number_of_groups: u16,
    pub size_of_body: i32,
    pub unknown: u16,
}

impl Default for DatFileHeader {
    fn default() -> Self {
        Self {
            file_signature: [0; 21],
            app_name: [0; 50],
            description: [0; 100],
            file_size: 0,
            number_of_groups: 0,
            size_of_body: 0,
            unknown: 0,
        }
    }
}

unsafe impl bytemuck::Zeroable for DatFileHeader {}
unsafe impl bytemuck::Pod for DatFileHeader {}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
pub struct Dat8BitBmpHeader {
    pub resolution: u8,
    pub width: i16,
    pub height: i16,
    pub x_position: i16,
    pub y_position: i16,
    pub size: i32,
    flags: Bmp8Flags,
}

unsafe impl bytemuck::Zeroable for Dat8BitBmpHeader {}
unsafe impl bytemuck::Pod for Dat8BitBmpHeader {}

impl Dat8BitBmpHeader {
    pub fn is_flag_set(&self, flag: Bmp8Flags) -> bool {
        (self.flags.0 & flag.0) != 0
    }
}

#[derive(Copy, Clone, Debug, Default)]
#[repr(packed)]
struct Dat16BitBmpHeader {
    width: i16,
    height: i16,
    stride: i16,
    unknown_0: i32,
    unknown_1_0: i16,
    unknown_1_1: i16,
}

unsafe impl bytemuck::Zeroable for Dat16BitBmpHeader {}
unsafe impl bytemuck::Pod for Dat16BitBmpHeader {}

const _: () = {
    const {
        assert!(
            size_of::<Dat8BitBmpHeader>() == 14,
            "Wrong size of dat8bitbmpheader"
        );
        assert!(
            size_of::<DatFileHeader>() == 183,
            "Wrong size of datfileheader"
        );
        assert!(
            size_of::<Dat16BitBmpHeader>() == 14,
            "Wrong size of dat16bitbmpheader"
        )
    }
};

pub static FIELD_SIZE: LazyLock<Mutex<[i16; 14]>> =
    LazyLock::new(|| Mutex::new([2, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0]));

pub fn load_records(file_name: String, full_tilt_mode: bool) -> Option<DatFile<'static>> {
    let mut header: DatFileHeader = Default::default();
    let mut bmp_header: Dat8BitBmpHeader = Default::default();
    let mut zmap_header: Dat16BitBmpHeader = Default::default();

    match File::open(&file_name) {
        Ok(mut file) => {
            let mut reader = BufReader::new(file);
            match reader.read_exact(bytemuck::bytes_of_mut(&mut header)) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error reading bytes for file: {}", e);
                }
            }

            if header.file_signature != *b"PARTOUT(4.0)RESOURCE\0" {
                println!("File signature does not match: {:?}", header.file_signature);
                return None;
            }

            let mut dat_file = DatFile::new();

            match CStr::from_bytes_until_nul(&header.app_name) {
                Ok(app_name) => {
                    dat_file.app_name = app_name.to_string_lossy().into_owned();
                }
                Err(e) => {
                    println!("Error parsing app_name: {}", e);
                    return None;
                }
            }
            match CStr::from_bytes_until_nul(&header.description) {
                Ok(description) => {
                    dat_file.description = description.to_string_lossy().into_owned();
                }
                Err(e) => {
                    println!("Error parsing description: {}", e);
                    return None;
                }
            }

            if header.unknown > 0 {
                let unknown_size: usize = header.unknown as usize;
                let mut unknown_buffer: Vec<c_char> = vec![0; unknown_size];

                if let Err(e) = reader.seek(SeekFrom::Current(header.unknown as i64)) {
                    println!("Error reading past bytes for unknown data: {}", e);
                    return None;
                }
            }

            dat_file.groups.reserve(header.number_of_groups as usize);
            let abort = false;
            for group_index in 0..header.number_of_groups {
                if abort {
                    break;
                }

                match u8::lread(&mut reader) {
                    Ok(entry_count) => {
                        let mut group_data = GroupData::new(group_index as i32);
                        group_data.reserve_entries(entry_count as usize);

                        for entry_index in 0..entry_count {
                            let mut entry_data = EntryData::default();
                            match u8::lread(&mut reader) {
                                Ok(entry_type) => unsafe {
                                    let field_type = FieldTypes::from_u8(entry_type)?;
                                    entry_data.entry_type = field_type;
                                    let fixed_size =
                                        FIELD_SIZE.lock().unwrap()[entry_index as usize];
                                    let mut field_size = if fixed_size >= 0 {
                                        fixed_size
                                    } else {
                                        u32::lread(&mut reader).unwrap().to_i16()?
                                    };
                                    entry_data.field_size = field_size as i32;

                                    if field_type == FieldTypes::Bitmap8bit {
                                        if let Err(e) = reader
                                            .read_exact(bytemuck::bytes_of_mut(&mut bmp_header))
                                        {
                                            println!("Error reading bmp header: {}", e);
                                        }
                                        assert_eq!(
                                            bmp_header.size as usize
                                                + size_of::<Dat8BitBmpHeader>(),
                                            field_size as usize,
                                            "partman: Wrong bitmap field size"
                                        );
                                        assert!(
                                            bmp_header.resolution <= 2,
                                            "partman: bitmap resolution out of bounds"
                                        );

                                        let mut bmp = GdrvBitmap8::new(&bmp_header);
                                        // This is ugly, very ugly
                                        entry_data.buffer = Some(std::slice::from_raw_parts(
                                            &bmp as *const _ as *const u8,
                                            std::mem::size_of::<GdrvBitmap8>(),
                                        ));
                                        let mut indexed_bmp_data_buffer =
                                            vec![0u8; bmp_header.size as usize];
                                        if let Err(e) =
                                            reader.read_exact(&mut indexed_bmp_data_buffer)
                                        {
                                            println!("Error reading bmp data buffer: {}", e);
                                        }
                                        bmp.indexed_bmp_ptr =
                                            &raw mut indexed_bmp_data_buffer as *const c_char;
                                    } else if field_type == FieldTypes::Bitmap16bit {
                                        let mut z_map_resolution = 0u8;
                                        if FULL_TILT_MODE.load(Relaxed) {
                                            /*Full tilt has extra byte(@0:resolution) in zMap*/
                                            z_map_resolution = u8::lread(&mut reader).unwrap();
                                            field_size -= 1;

                                            // -1 means universal resolution, maybe. FT demo .006 is the only known user.
                                            if z_map_resolution == 0xff {
                                                z_map_resolution = 0;
                                            }
                                            assert!(
                                                z_map_resolution <= 2,
                                                "partman: zMap resolution out of bounds"
                                            );
                                        }

                                        if let Err(e) = reader
                                            .read_exact(bytemuck::bytes_of_mut(&mut zmap_header))
                                        {
                                            println!("Failed to read zmap header: {}", e);
                                        }

                                        let length =
                                            field_size as usize - size_of::<Dat16BitBmpHeader>();
                                        
                                        // TODO: Continue, line 100 of partman.cpp
                                    }
                                },
                                Err(e) => {
                                    println!("Error reading entry type: {}", e);
                                    return None;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error reading group index {}, {}", group_index, e);
                        return None;
                    }
                }
            }
        }
        Err(e) => {
            println!("Error opening file {}: {}", file_name, e);
            return None;
        }
    }

    None
}
