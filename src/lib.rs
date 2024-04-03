#![feature(strict_provenance)]
#![feature(seek_stream_len)]

use std::fmt::Debug;

use partition::PartitionProperty;
use thiserror::Error;
use windows::Win32::System::Hypervisor::{
    WHvGetCapability, WHV_CAPABILITY, WHV_CAPABILITY_CODE, WHV_PROCESSOR_VENDOR,
};

use flags::{CapabilityFeatures, ExtendedVmExits, ProcessorFeatures, ProcessorXsaveFeatures};

pub mod fields;
pub mod flags;
pub mod memory;
pub mod partition;
pub mod processor;

// TODO: Require windows target.
// TODO: Move architecture specific stuff behind flags? I.e. `WHV_X64_*`.

// TODO: Rename all these errors to more idiomatic names.
#[derive(Error, Debug)]
pub enum Error {
    #[error("a windows function returned: {0}")]
    Windows(#[from] windows::core::Error),
    #[error("failed int conversion: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
    #[error(
        "incompatible partition property availability, property ({0:?}) unable to be set after setup"
    )]
    IncompatiblePropertyAvailibility(PartitionProperty),
    #[error("index {0} is greater than the partition's processor count ({1})")]
    InvalidVpIndex(u32, u32),
}

/// A specialized [`Result`] type that provides Windows Hypervisor error information.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum ProcessorVendor {
    Amd = 0x0,
    Intel = 0x1,
    Hygon = 0x2,
}

impl From<ProcessorVendor> for WHV_PROCESSOR_VENDOR {
    fn from(value: ProcessorVendor) -> Self {
        Self(value as i32)
    }
}

impl From<WHV_PROCESSOR_VENDOR> for ProcessorVendor {
    fn from(value: WHV_PROCESSOR_VENDOR) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0x00000000 => Self::Amd,
            0x00000001 => Self::Intel,
            0x00000002 => Self::Hygon,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(i32)]
pub enum CapabilityCode {
    HypervisorPresent = 0x0,
    Features = 0x1,
    ExtendedVmExits = 0x2,
    ProcessorVendor = 0x1000,
    ProcessorFeatures = 0x1001,
    ProcessorClFlushSize = 0x1002,
    ProcessorXsaveFeatures = 0x1003,
}

impl From<CapabilityCode> for WHV_CAPABILITY_CODE {
    fn from(value: CapabilityCode) -> Self {
        Self(value as i32)
    }
}

impl From<WHV_CAPABILITY_CODE> for CapabilityCode {
    fn from(value: WHV_CAPABILITY_CODE) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0x00000000 => Self::HypervisorPresent,
            0x00000001 => Self::Features,
            0x00000002 => Self::ExtendedVmExits,
            0x00001000 => Self::ProcessorVendor,
            0x00001001 => Self::ProcessorFeatures,
            0x00001002 => Self::ProcessorClFlushSize,
            0x00001003 => Self::ProcessorXsaveFeatures,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    HypervisorPresent(bool),
    Features(CapabilityFeatures),
    ExtendedVmExits(ExtendedVmExits),
    ProcessorVendor(ProcessorVendor),
    ProcessorFeatures(ProcessorFeatures),
    ProcessorXsaveFeatures(ProcessorXsaveFeatures),
    ProcessorClFlushSize(u8),
}

impl Capability {
    fn from_union(code: CapabilityCode, cap: WHV_CAPABILITY) -> Self {
        // SAFETY: The code corresponds to the union variant.
        unsafe {
            match code {
                CapabilityCode::HypervisorPresent => {
                    Capability::HypervisorPresent(cap.HypervisorPresent.as_bool())
                }
                CapabilityCode::Features => Capability::Features(cap.Features.into()),
                CapabilityCode::ExtendedVmExits => {
                    Capability::ExtendedVmExits(cap.ExtendedVmExits.into())
                }
                CapabilityCode::ProcessorVendor => {
                    Capability::ProcessorVendor(cap.ProcessorVendor.into())
                }
                CapabilityCode::ProcessorFeatures => {
                    Capability::ProcessorFeatures(cap.ProcessorFeatures.into())
                }
                CapabilityCode::ProcessorClFlushSize => {
                    Capability::ProcessorClFlushSize(cap.ProcessorClFlushSize)
                }
                CapabilityCode::ProcessorXsaveFeatures => {
                    Capability::ProcessorXsaveFeatures(cap.ProcessorXsaveFeatures.into())
                }
            }
        }
    }
}

// TODO: Move this to [CapabilityCode]?
pub fn query_capability(code: CapabilityCode) -> Result<Capability> {
    let mut cap: WHV_CAPABILITY = Default::default();
    unsafe {
        WHvGetCapability(
            code.into(),
            std::mem::transmute(&mut cap),
            std::mem::size_of::<WHV_CAPABILITY>().try_into()?,
            None,
        )?;
    };
    Ok(Capability::from_union(code, cap))
}
