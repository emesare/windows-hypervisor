use std::{fs::File, io::Seek, os::windows::io::AsRawHandle};

use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::{
        CreateFileMappingA, MapViewOfFile, UnmapViewOfFile, VirtualAlloc, VirtualFree,
        FILE_MAP_ALL_ACCESS, MEMORY_MAPPED_VIEW_ADDRESS, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE,
        PAGE_READWRITE,
    },
};

use crate::{flags::MapGpaRangeFlags, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionBacking {
    /// Not backed by anything other than the allocation itself.
    Volatile,
    /// Backed by a file on the filesystem.
    File,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MemoryRegion {
    // TODO: We should have a different type for `address`, doesn't convey the correct meaning.
    pub address: usize,
    pub guest_address: usize,
    pub flags: MapGpaRangeFlags,
    pub size: usize,
    pub backing: RegionBacking,
}

// TODO: Add a way to detect dirty pages (probably requires file backing?).
// TODO: Add a way to search the region and manipulate it.
// TODO: Showcase how to map from a file and not bloat the memory... (i.e. not paged in until a page is accessed)
impl MemoryRegion {
    pub fn from_bytes(guest_address: usize, flags: MapGpaRangeFlags, bytes: &[u8]) -> Self {
        // TODO: Should we use MEM_RESERVE?
        let address =
            unsafe { VirtualAlloc(None, bytes.len(), MEM_RESERVE | MEM_COMMIT, PAGE_READWRITE) };

        unsafe {
            address.copy_from(bytes.as_ptr() as *const _, bytes.len());
        }

        Self {
            address: address.addr(),
            guest_address,
            flags,
            size: bytes.len(),
            backing: RegionBacking::Volatile,
        }
    }

    pub fn from_file(
        guest_address: usize,
        flags: MapGpaRangeFlags,
        mut file: File,
    ) -> Result<Self> {
        // TODO: Error handling.
        let file_len = file.stream_len().unwrap();

        let raw_file_handle = file.as_raw_handle();
        // TODO: Check make sure its valid handle.

        let mapping_handle = unsafe {
            CreateFileMappingA(
                HANDLE(raw_file_handle as _),
                None,
                PAGE_READWRITE,
                0,
                0,
                None,
            )?
        };

        // TODO: Make sure this is a valid address.
        let address = unsafe { MapViewOfFile(mapping_handle, FILE_MAP_ALL_ACCESS, 0, 0, 0) };

        unsafe { CloseHandle(mapping_handle)? };

        Ok(Self {
            address: address.Value.addr(),
            guest_address,
            flags,
            size: file_len.try_into()?,
            backing: RegionBacking::File,
        })
    }
}

impl Drop for MemoryRegion {
    fn drop(&mut self) {
        match self.backing {
            RegionBacking::Volatile => unsafe {
                VirtualFree(self.address as *mut _, 0, MEM_RELEASE).unwrap()
            },
            RegionBacking::File => unsafe {
                UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                    Value: self.address as _,
                })
                .unwrap()
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use tempfile::tempfile;

    use crate::flags::MapGpaRangeFlags;

    use super::MemoryRegion;

    #[test]
    fn map_bytes() {
        let mr = MemoryRegion::from_bytes(
            0xF0000,
            MapGpaRangeFlags::Read | MapGpaRangeFlags::Execute,
            &[0, 1, 2, 3, 4, 5],
        );
        assert_ne!(mr.address, 0);
    }

    #[test]
    fn map_file() {
        let mut file = tempfile().unwrap();
        writeln!(file, "Hello world").unwrap();

        let mr = MemoryRegion::from_file(
            0xF0000,
            MapGpaRangeFlags::Read | MapGpaRangeFlags::Execute,
            file,
        )
        .unwrap();
        assert_eq!(mr.size, 12);
    }
}
