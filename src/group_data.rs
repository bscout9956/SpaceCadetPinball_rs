use crate::embedded_data::PB_MSGFT_BIN_COMPRESSED_DATA_BASE85;
use crate::errors::GroupDataError;
use crate::gdrv::{BitmapTypes, GdrvBitmap8};
use crate::zdrv::ZMapHeaderType;
use crate::{fullscrn, pb, zdrv};
use base85::Error;
use num_derive::FromPrimitive;
use std::array;
use std::cmp::PartialOrd;
use std::sync::atomic::Ordering::Relaxed;
use thiserror::Error;

#[derive(PartialEq, PartialOrd, Copy, Clone, FromPrimitive)]
#[repr(i16)]
pub enum FieldTypes {
    // One 16-bit signed integer
    ShortValue = 0,
    // Sprite bitmap, 8bpp, indexed color
    Bitmap8bit = 1,
    Unknown2 = 2,
    // Group name, char[]. Not all groups have names.
    GroupName = 3,
    Unknown4 = 4,
    // Palette, contains 256 RBGA 4-byte colors.
    Palette = 5,
    Unknown6 = 6,
    Unknown7 = 7,
    Unknown8 = 8,
    // String, char[]
    String = 9,
    // Array of 16-bit signed integers
    ShortArray = 10,
    // Array of 32-bit floats
    FloatArray = 11,
    // Sprite depth map, 16bpp, unsigned
    Bitmap16bit = 12,
}

#[derive(Clone)]
pub enum EntryBuffer {
    Bitmap8(GdrvBitmap8),
    Bitmap16(ZMapHeaderType),
    Raw(Vec<u8>),
}

#[derive(Clone)]
pub struct EntryData {
    pub entry_type: FieldTypes,
    pub field_size: i32,
    pub buffer: EntryBuffer,
}

impl EntryData {
    pub fn new(entry_type: FieldTypes, field_size: i32, buffer: EntryBuffer) -> Self {
        Self {
            entry_type,
            field_size,
            buffer,
        }
    }

    pub fn default() -> Self {
        Self {
            entry_type: FieldTypes::Unknown8,
            field_size: 0,
            buffer: EntryBuffer::Raw(vec![]),
        }
    }
}

struct MsgFontChar {
    width: u8,
    data: Vec<u8>,
}

struct MsgFont {
    gap_width: i16,
    unknown_1: i16,
    height: i16,
    char_widths: [u8; 128],
    data: Vec<MsgFontChar>,
}

pub struct GroupData {
    group_id: i32,
    group_name: String,
    entries: Vec<EntryData>,
    bitmaps: [GdrvBitmap8; 3],
    z_maps: [ZMapHeaderType; 3],
    needs_sort: bool,
}

impl GroupData {
    pub fn get_entries(&self) -> &[EntryData] {
        &self.entries
    }
    pub fn get_zmap(&self, resolution: i32) -> &ZMapHeaderType {
        &self.z_maps[resolution as usize]
    }

    pub fn get_bitmap(&self, resolution: i32) -> &GdrvBitmap8 {
        &self.bitmaps[resolution as usize]
    }

    pub fn finalize_group(&mut self) {
        if self.needs_sort {
            self.needs_sort = false;
            self.entries
                .sort_by(|a, b| a.entry_type.partial_cmp(&b.entry_type).unwrap());
        }
    }

    pub fn new(group_id: i32) -> Self {
        Self {
            group_id,
            group_name: "".to_string(),
            entries: vec![],
            bitmaps: array::from_fn(|_| GdrvBitmap8::default()),
            z_maps: array::from_fn(|_| ZMapHeaderType::default()),
            needs_sort: true,
        }
    }

    pub fn reserve_entries(&mut self, count: usize) {
        self.entries.reserve(count);
    }
    pub fn add_entry(&mut self, entry: EntryData) {
        match entry.entry_type {
            FieldTypes::Bitmap8bit => {
                if let EntryBuffer::Bitmap8(src_bmp) = &entry.buffer {
                    if src_bmp.bitmap_type == BitmapTypes::Spliced {
                        // Get rid of spliced bitmap early on, to simplify render pipeline
                        let mut bmp =
                            GdrvBitmap8::new_dims_indexed(src_bmp.width, src_bmp.height, true);
                        let mut zmap =
                            ZMapHeaderType::new(src_bmp.width, src_bmp.height, src_bmp.width);

                        let _ = split_sliced_bitmap(src_bmp, &mut bmp, &mut zmap);

                        self.needs_sort = true;
                        self.add_entry(EntryData::new(
                            FieldTypes::Bitmap8bit,
                            -1,
                            EntryBuffer::Bitmap8(bmp),
                        ));
                        self.add_entry(EntryData::new(
                            FieldTypes::Bitmap16bit,
                            -1,
                            EntryBuffer::Bitmap16(zmap),
                        ));

                        return;
                    } else {
                        self.set_bitmap(src_bmp.clone());
                    }
                }
            }
            FieldTypes::GroupName => {
                if let EntryBuffer::Raw(data) = &entry.buffer {
                    self.group_name = String::from_utf8(data.clone()).unwrap();
                } else {
                    panic!("Unrecognized data type...");
                }
            }
            FieldTypes::Bitmap16bit => {
                if let EntryBuffer::Bitmap16(src_data) = &entry.buffer {
                    self.set_zmap(src_data.clone());
                }
            }
            _ => {}
        }

        self.entries.push(entry);
    }

    pub fn set_bitmap(&mut self, bmp: GdrvBitmap8) {
        let bmp_res = bmp.resolution as usize;
        let bmp_height = bmp.height as usize;
        let bmp_width = bmp.width as usize;

        assert_eq!(self.bitmaps[bmp_res].width, 0, "GroupData: bitmap override");
        self.bitmaps[bmp_res] = bmp;

        let zmap = &self.z_maps[bmp_res];

        if zmap.width > 0 && zmap.height > 0 {
            assert!(
                bmp_width == zmap.width as usize && bmp_height == zmap.height as usize,
                "GroupData: Mismatched bitmap/zmap dimensions"
            );
        }
    }

    pub fn set_zmap(&mut self, mut zmap: ZMapHeaderType) {
        zdrv::flip_zmap_horizontally(&mut zmap);
        let zmap_res = zmap.resolution as usize;
        let zmap_width = zmap.width as usize;
        let zmap_height = zmap.height as usize;

        assert_eq!(self.z_maps[zmap_res].width, 0, "GroupData: zMap override");

        self.z_maps[zmap_res] = zmap;

        let bmp = &self.bitmaps[zmap_res];

        if bmp.width > 0 && bmp.height > 0 {
            assert!(
                bmp.width as usize == zmap_width && bmp.height as usize == zmap_height,
                "GroupData: Mismatched bitmap/zmap dimensions"
            );
        }
    }
}

pub fn split_sliced_bitmap(
    src_bmp: &GdrvBitmap8,
    bmp: &mut GdrvBitmap8,
    zmap: &mut ZMapHeaderType,
) -> Result<(), GroupDataError> {
    assert_eq!(
        src_bmp.bitmap_type,
        BitmapTypes::Spliced,
        "GroupData: wrong bitmap type"
    );

    bmp.indexed_bmp_data = vec![0xFF; (bmp.stride * bmp.height) as usize];
    bmp.x_position = src_bmp.x_position;
    bmp.y_position = src_bmp.y_position;
    bmp.resolution = src_bmp.resolution;

    crate::zdrv::fill(zmap, zmap.width, zmap.height, 0, 0, 0xFFFF);
    zmap.resolution = src_bmp.resolution;

    let res_array = fullscrn::RESOLUTION_ARRAY.lock()?;
    let table_width = (*res_array)[src_bmp.resolution as usize].table_width;
    let src = &src_bmp.indexed_bmp_data;
    let src_char = &src;

    let mut src_idx = 0;
    let mut dst_idx = 0;

    // This was translated by an LLM,
    // go look at the original code
    // if you think you can make sense of it...
    // TODO: Rewrite this by hand like I did flip_zmap_horizontally?
    loop {
        if src_idx + 2 > src.len() {
            break;
        }
        let stride = i16::from_le_bytes([src[src_idx], src[src_idx + 1]]);
        src_idx += 2;

        if stride < 0 {
            break;
        }

        let mut stride = stride as i32;

        // Stride is in terms of dst stride, hardcoded to match vScreen width in current resolution
        if stride > bmp.width {
            stride += bmp.width - table_width as i32;
            assert!(stride >= 0, "Spliced bitmap: negative computed stride");
        }

        dst_idx += stride as usize;

        if src_idx + 2 > src.len() {
            break;
        }

        let mut count = u16::from_le_bytes([src[src_idx], src[src_idx + 1]]);
        src_idx += 2;

        // PS: Equivalent to the original for loop with auto count
        while count > 0 {
            if src_idx + 2 > src.len() {
                break;
            }
            let depth = u16::from_le_bytes([src[src_idx], src[src_idx + 1]]);
            src_idx += 2;

            if src_idx >= src.len() {
                break;
            }
            let color = src[src_idx];
            src_idx += 1;

            bmp.indexed_bmp_data[dst_idx] = color;
            zmap.z_map_data[dst_idx] = depth;

            dst_idx += 1;
            count -= 1;
        }
    }

    Ok(())
}

pub struct DatFile {
    pub app_name: String,
    pub description: String,
    pub groups: Vec<GroupData>,
}

#[derive(Error, Debug)]
pub enum DatFileError {
    #[error("Could not parse pinball font file")]
    DecodeError(#[from] Error),
}

unsafe impl Send for DatFile {}
unsafe impl Sync for DatFile {}

impl DatFile {
    pub fn new() -> Self {
        Self {
            app_name: "".to_string(),
            description: "".to_string(),
            groups: vec![],
        }
    }

    pub fn field(&self, group_index: i32, target_entry_type: FieldTypes) -> Option<&EntryBuffer> {
        assert!(
            target_entry_type != FieldTypes::Bitmap8bit
                && target_entry_type != FieldTypes::Bitmap16bit,
            "partman: Use specific get for bitmaps"
        );

        let group = self.groups.get(group_index as usize)?;

        for entry in group.get_entries() {
            if entry.entry_type == target_entry_type {
                return Some(&entry.buffer);
            }
            if entry.entry_type > target_entry_type {
                break;
            }
        }
        None
    }

    pub fn field_nth(
        &self,
        group_index: i32,
        target_entry_type: FieldTypes,
        skip_first_n: i32,
    ) -> Option<&EntryBuffer> {
        assert!(
            target_entry_type != FieldTypes::Bitmap8bit
                && target_entry_type != FieldTypes::Bitmap16bit,
            "partman: Use specific get for bitmaps"
        );

        let group = self.groups.get(group_index as usize)?;
        let mut skip_count = 0;
        for entry in group.get_entries() {
            if entry.entry_type > target_entry_type {
                break;
            }
            if entry.entry_type == target_entry_type {
                if skip_count == skip_first_n {
                    skip_count += 1;
                    return Some(&entry.buffer);
                } else {
                    skip_count += 1;
                }
            }
        }

        None
    }

    pub fn field_labeled(&self, name: &str, field_type: FieldTypes) -> Option<&EntryBuffer> {
        let group_index = self.record_labeled(name);
        if group_index < 0 {
            None
        } else {
            self.field(group_index, field_type)
        }
    }

    pub fn finalize(&mut self) -> Result<(), DatFileError> {
        let is_full_tilt = pb::FULL_TILT_MODE.load(Relaxed);

        if !is_full_tilt {
            let group_index = self.record_labeled("pbmsg_ft");
            assert!(group_index < 0, "DatFile: pbmsg_ft is already in .dat");
        }

        let rc_data = base85::decode(PB_MSGFT_BIN_COMPRESSED_DATA_BASE85)?; //TODO: use result yadda yadda

        self.add_msg_font(&rc_data, "pbmsg_ft");

        for group in &mut self.groups {
            group.finalize_group();
        }

        Ok(())
    }

    fn add_msg_font(&mut self, font_data: &[u8], font_name: &str) -> Result<(), GroupDataError> {
        if font_data.len() < 134 {
            return Err(GroupDataError::InvalidBufferLength);
        }

        let gap_width = i16::from_le_bytes(font_data[0..2].try_into().unwrap());
        let height = i16::from_le_bytes(font_data[4..6].try_into().unwrap()) as usize;

        let mut char_widths = [0u8; 128];
        char_widths.copy_from_slice(&font_data[6..134]); // 128

        let mut cursor = &font_data[134..];
        let mut group_id = self.groups.last().map(|g| g.group_id).unwrap_or(0) + 1;

        for char_index in 32..128 {
            if cursor.is_empty() {
                return Err(GroupDataError::InvalidBufferLength);
            }

            let width = cursor[0] as usize;
            if width != char_widths[char_index] as usize {
                return Err(GroupDataError::FontWidthMismatch);
            }

            let total_chunk_size = 1 + (width * height);
            if cursor.len() < total_chunk_size {
                return Err(GroupDataError::InvalidBufferLength);
            }

            let char_pixel_data = &cursor[1..total_chunk_size];

            let mut bmp = GdrvBitmap8::new_dims(width as i32, height as i32);
            let byte_count = (bmp.height * bmp.stride) as usize;
            bmp.indexed_bmp_data.resize(byte_count, 0);

            for y in 0..height {
                let src_start = y * width;
                let src_end = src_start + width;

                let dst_row = height - 1 - y;
                let dst_start = dst_row * (bmp.stride as usize);
                let dst_end = dst_start + width;

                bmp.indexed_bmp_data[dst_start..dst_end]
                    .copy_from_slice(&char_pixel_data[src_start..src_end]);
            }

            let mut group_data = GroupData::new(group_id);
            let bmp_field_size = byte_count as i32;
            let bmp_entry = EntryData::new(
                FieldTypes::Bitmap8bit,
                bmp_field_size,
                EntryBuffer::Bitmap8(bmp),
            );
            group_data.add_entry(bmp_entry);

            if char_index == 32 {
                let mut name_bytes = font_name.as_bytes().to_vec();
                name_bytes.push(0);

                let name_entry = EntryData::new(
                    FieldTypes::GroupName,
                    name_bytes.len() as i32,
                    EntryBuffer::Raw(name_bytes),
                );

                group_data.add_entry(name_entry);

                let gap_bytes = gap_width.to_le_bytes().to_vec();
                let gap_entry =
                    EntryData::new(FieldTypes::ShortArray, 2, EntryBuffer::Raw(gap_bytes));
                group_data.add_entry(gap_entry);
            } else {
                let group_name = format!("char {}='{}'\0", char_index, char_index as u8 as char);
                let name_bytes = group_name.into_bytes();

                let name_entry = EntryData::new(
                    FieldTypes::GroupName,
                    name_bytes.len() as i32,
                    EntryBuffer::Raw(name_bytes),
                );
                group_data.add_entry(name_entry);
            }
            self.groups.push(group_data);
            group_id += 1;
        }

        Ok(())
    }

    pub fn field_size_nth(
        &self,
        group_index: i32,
        target_entry_type: FieldTypes,
        skip_first_n: i32,
    ) -> i32 {
        assert!(
            target_entry_type != FieldTypes::Bitmap8bit
                && target_entry_type != FieldTypes::Bitmap16bit,
            "partman: Use specific get for bitmaps"
        );

        let group = self.groups.get(group_index as usize).unwrap();
        let mut skip_count = 0;
        for entry in group.get_entries() {
            if entry.entry_type > target_entry_type {
                return 0;
            }
            if entry.entry_type == target_entry_type {
                if skip_count == skip_first_n {
                    skip_count += 1;
                    return entry.field_size;
                } else {
                    skip_count += 1;
                }
            }
        }

        0
    }

    pub fn field_size(&self, group_index: i32, target_entry_type: FieldTypes) -> i32 {
        self.field_size_nth(group_index, target_entry_type, 0)
    }

    pub fn get_bitmap(&self, group_index: i32) -> &GdrvBitmap8 {
        let group = self.groups.get(group_index as usize).unwrap();
        group.get_bitmap(fullscrn::get_resolution())
    }

    pub fn get_zmap(&self, group_index: i32) -> &ZMapHeaderType {
        let group = self.groups.get(group_index as usize).unwrap();
        group.get_zmap(fullscrn::get_resolution())
    }

    pub fn record_labeled(&self, target_group_name: &str) -> i32 {
        let target_data = target_group_name.as_bytes();

        for group_index in (0..self.groups.len()).rev() {
            match self.field(group_index as i32, FieldTypes::GroupName) {
                Some(EntryBuffer::Raw(group_name_data)) => {
                    let group_name = if group_name_data.last() == Some(&0) {
                        &group_name_data[..group_name_data.len() - 1]
                    } else {
                        group_name_data.as_slice()
                    };

                    if target_data == group_name {
                        return group_index as i32;
                    }
                }
                // None or Bitmap
                _ => continue,
            }
        }

        -1
    }
}
