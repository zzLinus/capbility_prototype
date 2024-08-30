use alloc::{string::String, vec::Vec};
use core::{fmt::Display, mem::size_of};

pub(super) const ARCHIVE_MAGIC: [u8; 8] = [0x21, 0x3c, 0x61, 0x72, 0x63, 0x68, 0x3E, 0x0A];

pub struct ArchiveSymtab<'a> {
    count: u32,
    offsets: &'a [u8],
    data: &'a [u8],
}

impl<'a> ArchiveSymtab<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if data.len() < size_of::<Self>() {
            return None;
        }
        let count = u32::from_be_bytes(data[0..4].try_into().unwrap());
        let offsets = &data[4..];
        Some(Self {
            count,
            offsets,
            data,
        })
    }
}

pub struct ArchiveMemberHeader<'a> {
    name: &'a [u8],
    date: &'a [u8],
    uid: &'a [u8],
    gid: &'a [u8],
    mode: &'a [u8],
    size: &'a [u8],
    fmag: &'a [u8],
    data_offset: usize,
}

impl<'a> Display for ArchiveMemberHeader<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "name: {:?}\n,date: {:?}\n,uid: {:?}\n,gid: {:?}\n,mode: {:?}\n,size: {:?}\n,fmag: {:?}\n",
            self.name, self.date, self.uid, self.gid, self.mode, self.size, self.fmag
        )
    }
}

impl<'a> ArchiveMemberHeader<'a> {
    pub const fn length() -> usize {
        60
    }

    pub fn name(&self) -> String {
        let mut name = String::new();
        for i in 0..self.name.len() {
            if self.name[i] == 0x20 {
                break;
            }
            name.push(self.name[i] as char);
        }
        name
    }

    fn lfn_offset(&self) -> usize {
        let mut index = 0;
        for i in 1..self.name.len() {
            if self.name[i] == 0x20 {
                break;
            }
            index = index * 10 + (self.name[i] - b'0') as usize;
        }
        index
    }

    pub fn long_name(&self, data: &'a [u8]) -> String {
        let offset = self.lfn_offset();
        let mut name = String::new();
        for i in offset..data.len() {
            if data[i] == 0x0A {
                break;
            }
            name.push(data[i] as char);
        }
        name
    }

    pub fn data_offset(&self) -> usize {
        self.data_offset
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        for i in 0..self.size.len() {
            if self.size[i] == 0x20 {
                break;
            }
            size *= 10;
            size += (self.size[i] - b'0') as usize;
        }
        size
    }

    pub fn mode(&self) -> u32 {
        let mut mode = 0;
        for i in 0..self.mode.len() {
            if self.mode[i] == 0x20 {
                break;
            }
            mode *= 8;
            mode += (self.mode[i] - b'0') as u32;
        }
        mode
    }

    pub fn is_symtab(&self) -> bool {
        self.name[0] == b'/' && self.name[1] == b' '
    }

    pub fn is_lfntab(&self) -> bool {
        self.name[0] == b'/' && self.name[1] == b'/'
    }

    pub fn is_long_name(&self) -> bool {
        self.name[0] == b'/' && self.name[1] != b' '
    }

    pub fn from_bytes(data: &'a [u8], offset: usize) -> Option<Self> {
        if data.len() < size_of::<Self>() {
            return None;
        }
        Some(Self {
            name: &data[0..16],
            date: &data[16..28],
            uid: &data[28..34],
            gid: &data[34..40],
            mode: &data[40..48],
            size: &data[48..58],
            fmag: &data[58..60],
            data_offset: offset,
        })
    }
}

pub fn parse_archive(data: &[u8]) -> Option<Vec<ArchiveMemberHeader>> {
    let header_length = ArchiveMemberHeader::length();
    let magic_len = ARCHIVE_MAGIC.len();
    if data.len() < magic_len || data[0..magic_len] != ARCHIVE_MAGIC {
        return None;
    }
    let mut members = Vec::new();
    let mut offset = magic_len;
    while offset < data.len() {
        if let Some(header) =
            ArchiveMemberHeader::from_bytes(&data[offset..], offset + ArchiveMemberHeader::length())
        {
            offset += header_length + header.size();
            members.push(header);
            offset += offset % 2;
        } else {
            break;
        }
    }
    Some(members)
}

pub(super) fn get_file_name_from_archive(
    header: &ArchiveMemberHeader,
    lfntab: Option<&[u8]>,
) -> Result<String, &'static str> {
    if header.is_long_name() {
        match lfntab {
            Some(lfntab) => Ok(header.long_name(lfntab)),
            None => Err("failed to parse long file name table."),
        }
    } else {
        Ok(header.name())
    }
}
