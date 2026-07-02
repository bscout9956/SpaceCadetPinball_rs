use crate::errors::RecordLoadError;
use crate::gdrv::GdrvBitmap8;
use crate::group_data::{DatFile, EntryBuffer, EntryData, FieldTypes, GroupData};
use crate::state::fullscrn_state::FullscrnState;
use crate::utils;
use crate::zdrv::ZMapHeaderType;
use anyhow::{Result, bail};
use num_traits::FromPrimitive;
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::ops::{BitAnd, BitOr};
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
#[repr(packed, C)]
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
#[repr(packed, C)]
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
#[repr(packed, C)]
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

pub const FIELD_SIZE: [i16; 14] = [2, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0];

pub fn validate_bmp_8_header(
    bmp_header: &Dat8BitBmpHeader,
    field_size: u32,
) -> Result<(), RecordLoadError> {
    if bmp_header.size as usize + size_of::<Dat8BitBmpHeader>() != field_size as usize {
        return Err(RecordLoadError::BitmapFieldSize);
    }
    if bmp_header.resolution > 2 {
        return Err(RecordLoadError::BitmapResolutionOob);
    }
    Ok(())
}

pub fn load_records(
    file_name: String,
    full_tilt_mode: bool,
    fullscrn_state: &mut FullscrnState,
) -> Result<DatFile> {
    let mut header: DatFileHeader = Default::default();
    let mut bmp_header: Dat8BitBmpHeader = Default::default();
    let mut zmap_header: Dat16BitBmpHeader = Default::default();

    let file = File::open(&file_name)?;
    let mut reader = BufReader::new(file);
    reader.read_exact(bytemuck::bytes_of_mut(&mut header))?;

    if header.file_signature != *b"PARTOUT(4.0)RESOURCE\0" {
        bail!(RecordLoadError::IncorrectFileSignature);
    }

    let mut dat_file = DatFile::new();

    let app_name = CStr::from_bytes_until_nul(&header.app_name)?;
    dat_file.app_name = app_name.to_string_lossy().into_owned();

    let description = CStr::from_bytes_until_nul(&header.description)?;
    dat_file.description = description.to_string_lossy().into_owned();

    if header.unknown > 0 {
        reader.seek(SeekFrom::Current(header.unknown as i64))?;
    }

    dat_file.groups.reserve(header.number_of_groups as usize);
    let mut abort = false;
    for group_index in 0..header.number_of_groups {
        if abort {
            break;
        }

        let entry_count = u8::lread(&mut reader)?;
        let mut group_data = GroupData::new(group_index as i32);
        group_data.reserve_entries(entry_count as usize);

        for _ in 0..entry_count {
            let mut entry_data = EntryData::default();
            let entry_type_u8 = u8::lread(&mut reader)?;

            let field_type =
                FieldTypes::from_u8(entry_type_u8).ok_or(RecordLoadError::InvalidFieldType)?;
            entry_data.entry_type = field_type;

            let fixed_size = FIELD_SIZE[entry_type_u8 as usize];
            let mut field_size = if fixed_size >= 0 {
                fixed_size as u32
            } else {
                u32::lread(&mut reader)?
            };

            entry_data.field_size = field_size as i32;

            let buff_enum = if field_type == FieldTypes::Bitmap8bit {
                reader.read_exact(bytemuck::bytes_of_mut(&mut bmp_header))?;
                validate_bmp_8_header(&bmp_header, field_size)?;

                let mut bmp = GdrvBitmap8::new(&bmp_header);
                let mut indexed_bmp_data_buffer = vec![0; bmp_header.size as usize];
                reader.read_exact(&mut indexed_bmp_data_buffer)?;

                bmp.indexed_bmp_data = indexed_bmp_data_buffer;

                EntryBuffer::Bitmap8(bmp)
            } else if field_type == FieldTypes::Bitmap16bit {
                let mut z_map_resolution = 0u8;
                if full_tilt_mode {
                    /*Full tilt has extra byte(@0:resolution) in zMap*/
                    z_map_resolution = u8::lread(&mut reader)?;
                    field_size -= 1;

                    // -1 means universal resolution, maybe. FT demo .006 is the only known user.
                    if z_map_resolution == 0xff {
                        z_map_resolution = 0;
                    }
                    assert!(
                        z_map_resolution > 2,
                        "partman: zMap resolution out of bounds"
                    );
                }

                reader.read_exact(bytemuck::bytes_of_mut(&mut zmap_header))?;
                let length = field_size as usize - size_of::<Dat16BitBmpHeader>();

                let mut zmap = ZMapHeaderType::new(
                    zmap_header.width.into(),
                    zmap_header.height.into(),
                    zmap_header.stride.into(),
                );

                if (zmap_header.stride as usize * zmap_header.height as usize * 2) == length {
                    zmap.z_map_data = vec![0u16; length / 2];
                    zmap.resolution = z_map_resolution as u32;
                    reader.read_exact(bytemuck::cast_slice_mut(&mut zmap.z_map_data))?;
                } else {
                    // 3DPB .dat has zeroed zMap headers, in groups 497 and 498, skip them.
                    reader.seek(SeekFrom::Current(length as i64))?;
                    zmap = ZMapHeaderType::new(0, 0, 0);
                }

                EntryBuffer::Bitmap16(zmap)
            } else {
                let mut raw_buffer = Vec::new();
                if raw_buffer.try_reserve_exact(field_size as usize).is_err() {
                    abort = true;
                    break;
                }

                raw_buffer.resize(field_size as usize, 0);
                reader.read_exact(&mut raw_buffer)?;

                EntryBuffer::Raw(raw_buffer) // Return Enum variant
            };

            let entry_data = EntryData::new(field_type, field_size as i32, buff_enum);

            group_data.add_entry(entry_data, fullscrn_state)?;
        }
        dat_file.groups.push(group_data);
    }

    if dat_file.groups.len() == header.number_of_groups as usize {
        dat_file.finalize(full_tilt_mode, fullscrn_state)?;
        return Ok(dat_file);
    }
    bail!(RecordLoadError::Unknown)
}
