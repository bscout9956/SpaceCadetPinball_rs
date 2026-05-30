use crate::gdrv::GdrvBitmap8;
use crate::zdrv::ZMapHeaderType;
use sdl2::libc::strlen;
use std::cmp::{Ordering, PartialOrd};
use std::ffi::{CStr, c_char};

#[derive(PartialEq, PartialOrd, Copy, Clone)]
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

struct EntryData<'a> {
    entry_type: FieldTypes,
    field_size: i32,
    buffer: Option<&'a [u8]>,
}

impl<'a> EntryData<'a> {
    fn new(entry_type: FieldTypes, buffer: Option<&'a [u8]>) -> Self {
        let mut field_size = 0;
        if let Some(buffer_vec) = buffer {
            field_size = buffer_vec.len();
        }
        Self {
            entry_type,
            field_size: field_size as i32,
            buffer,
        }
    }
}

#[repr(C, packed)]
struct MsgFontChar {
    width: u8,
    data: [u8; 1],
}

#[repr(C, packed)]
struct MsgFont {
    gap_width: i16,
    unknown_1: i16,
    height: i16,
    char_widths: [u8; 128],
    data: [MsgFontChar; 1],
}

pub struct GroupData<'a> {
    group_id: i32,
    group_name: String,
    entries: Vec<EntryData<'a>>,
    bitmaps: [GdrvBitmap8<'a>; 3],
    z_maps: [ZMapHeaderType; 3],
    needs_sort: bool,
}

impl<'a> GroupData<'a> {
    pub fn get_entries(&self) -> &[EntryData<'a>] {
        &self.entries
    }
    pub fn get_zmap(&self, resolution: i32) -> ZMapHeaderType {
        self.z_maps[resolution as usize]
    }

    pub fn get_bitmap(&self, resolution: i32) -> GdrvBitmap8 {
        self.bitmaps[resolution as usize]
    }
}

pub struct DatFile<'a> {
    pub app_name: String,
    pub description: String,
    pub groups: Vec<GroupData<'a>>,
}

impl<'a> DatFile<'a> {
    pub fn field(&self, group_index: i32, target_entry_type: FieldTypes) -> Option<&'a [u8]> {
        assert!(
            target_entry_type != FieldTypes::Bitmap8bit
                && target_entry_type != FieldTypes::Bitmap16bit,
            "partman: Use specific get for bitmaps"
        );

        let group = self.groups.get(group_index as usize)?;

        for entry in group.get_entries() {
            if entry.entry_type == target_entry_type {
                return entry.buffer;
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
    ) -> Option<&'a [u8]> {
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
                    return entry.buffer;
                } else {
                    skip_count += 1;
                }
            }
        }

        None
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

    pub fn get_bitmap(&self, p0: i32) -> _ {
        todo!()
    }

    pub fn get_zmap(&self, p0: i32) -> _ {
        todo!()
    }

    pub fn record_labeled(&self, target_group_name: *const c_char) -> i32 {
        let target_cstr = unsafe { CStr::from_ptr(target_group_name) };
        let target_data = target_cstr.to_bytes();

        for group_index in (0..self.groups.len()).rev() {
            match self.field(group_index as i32, FieldTypes::GroupName) {
                Some(group_name_data) => {
                    let group_name = if group_name_data.last() == Some(&0) {
                        &group_name_data[..group_name_data.len() - 1]
                    } else {
                        group_name_data
                    };

                    if target_data == group_name {
                        return group_index as i32;
                    }
                }
                None => continue,
            }
        }

        -1
    }
}
