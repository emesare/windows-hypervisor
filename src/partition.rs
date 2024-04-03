use std::sync::Arc;

use windows::Win32::System::Hypervisor::{
    WHvCreatePartition, WHvCreateVirtualProcessor, WHvDeletePartition, WHvGetPartitionProperty,
    WHvMapGpaRange, WHvSetPartitionProperty, WHvSetupPartition, WHV_CPUID_OUTPUT, WHV_MSR_ACTION,
    WHV_MSR_ACTION_ENTRY, WHV_PARTITION_HANDLE, WHV_PARTITION_PROPERTY,
    WHV_PARTITION_PROPERTY_CODE, WHV_PROCESSOR_FEATURES_BANKS, WHV_PROCESSOR_FEATURES_BANKS_0,
    WHV_PROCESSOR_FEATURES_BANKS_0_0, WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS,
    WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS_0, WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS_0_0,
    WHV_X64_CPUID_RESULT, WHV_X64_CPUID_RESULT2, WHV_X64_LOCAL_APIC_EMULATION_MODE,
};

use crate::{
    flags::{
        ExtendedVmExits, ProcessorFeatures, ProcessorFeatures1, ProcessorPerfmonFeatures,
        ProcessorXsaveFeatures, SyntheticProcessorFeatures, X64CpuidResult2Flags, X64MsrExitBitmap,
    },
    memory::MemoryRegion,
    processor::VirtualProcessor,
    Error, Result,
};

#[derive(Debug, Clone, Copy)]
pub enum PartitionPropertyAvailability {
    BeforeSetup,
    AfterSetup,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum PartitionPropertyCode {
    ExtendedVmExits = 0x1,
    ExceptionExitBitmap = 0x2,
    SeparateSecurityDomain = 0x3,
    NestedVirtualization = 0x4,
    X64MsrExitBitmap = 0x5,
    PrimaryNumaNode = 0x6,
    CpuReserve = 0x7,
    CpuCap = 0x8,
    CpuWeight = 0x9,
    CpuGroupId = 0xA,
    ProcessorFrequencyCap = 0xB,
    AllowDeviceAssignment = 0xC,
    DisableSmt = 0xD,
    ProcessorFeatures = 0x1001,
    ProcessorClFlushSize = 0x1002,
    CpuidExitList = 0x1003,
    CpuidResultList = 0x1004,
    LocalApicEmulationMode = 0x1005,
    ProcessorXsaveFeatures = 0x1006,
    ProcessorClockFrequency = 0x1007,
    InterruptClockFrequency = 0x1008,
    ApicRemoteRead = 0x1009,
    ProcessorFeaturesBanks = 0x100A,
    ReferenceTime = 0x100B,
    SyntheticProcessorFeaturesBanks = 0x100C,
    CpuidResultList2 = 0x100D,
    ProcessorPerfmonFeatures = 0x100E,
    MsrActionList = 0x100F,
    UnimplementedMsrAction = 0x1010,
    ProcessorCount = 0x1fff,
}

impl From<PartitionPropertyCode> for WHV_PARTITION_PROPERTY_CODE {
    fn from(value: PartitionPropertyCode) -> Self {
        // TODO: Why cant the repr(i32) expose the implicit conversion?
        Self(value as i32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SyntheticProcessorFeaturesBanks {
    pub banks_count: u32,
    pub bank_0: SyntheticProcessorFeatures,
}

impl From<WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS> for SyntheticProcessorFeaturesBanks {
    fn from(value: WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS) -> Self {
        Self {
            banks_count: value.BanksCount,
            bank_0: unsafe { value.Anonymous.Anonymous.Bank0.into() },
        }
    }
}

impl From<SyntheticProcessorFeaturesBanks> for WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS {
    fn from(value: SyntheticProcessorFeaturesBanks) -> Self {
        Self {
            BanksCount: value.banks_count,
            Reserved0: 0,
            Anonymous: WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS_0 {
                Anonymous: WHV_SYNTHETIC_PROCESSOR_FEATURES_BANKS_0_0 {
                    Bank0: value.bank_0.into(),
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct X64CpuidResult {
    pub function: u32,
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

impl From<WHV_X64_CPUID_RESULT> for X64CpuidResult {
    fn from(value: WHV_X64_CPUID_RESULT) -> Self {
        Self {
            function: value.Function,
            eax: value.Eax,
            ebx: value.Ebx,
            ecx: value.Ecx,
            edx: value.Edx,
        }
    }
}

impl From<X64CpuidResult> for WHV_X64_CPUID_RESULT {
    fn from(value: X64CpuidResult) -> Self {
        Self {
            Function: value.function,
            Reserved: Default::default(),
            Eax: value.eax,
            Ebx: value.ebx,
            Ecx: value.ecx,
            Edx: value.edx,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct X64CpuidResult2 {
    pub function: u32,
    pub index: u32,
    pub vp_index: u32,
    pub flags: X64CpuidResult2Flags,
    pub output: CpuidOutput,
    pub mask: CpuidOutput,
}

impl From<WHV_X64_CPUID_RESULT2> for X64CpuidResult2 {
    fn from(value: WHV_X64_CPUID_RESULT2) -> Self {
        Self {
            function: value.Function,
            index: value.Index,
            vp_index: value.VpIndex,
            flags: value.Flags.into(),
            output: value.Output.into(),
            mask: value.Mask.into(),
        }
    }
}

impl From<X64CpuidResult2> for WHV_X64_CPUID_RESULT2 {
    fn from(value: X64CpuidResult2) -> Self {
        Self {
            Function: value.function,
            Index: value.index,
            VpIndex: value.vp_index,
            Flags: value.flags.into(),
            Output: value.output.into(),
            Mask: value.mask.into(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CpuidOutput {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

impl From<WHV_CPUID_OUTPUT> for CpuidOutput {
    fn from(value: WHV_CPUID_OUTPUT) -> Self {
        Self {
            eax: value.Eax,
            ebx: value.Ebx,
            ecx: value.Ecx,
            edx: value.Edx,
        }
    }
}

impl From<CpuidOutput> for WHV_CPUID_OUTPUT {
    fn from(value: CpuidOutput) -> Self {
        Self {
            Eax: value.eax,
            Ebx: value.ebx,
            Ecx: value.ecx,
            Edx: value.edx,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MsrActionEntry {
    pub index: u32,
    pub read_action: u8,
    pub write_action: u8,
}

impl From<WHV_MSR_ACTION_ENTRY> for MsrActionEntry {
    fn from(value: WHV_MSR_ACTION_ENTRY) -> Self {
        Self {
            index: value.Index,
            read_action: value.ReadAction,
            write_action: value.WriteAction,
        }
    }
}

impl From<MsrActionEntry> for WHV_MSR_ACTION_ENTRY {
    fn from(value: MsrActionEntry) -> Self {
        Self {
            Index: value.index,
            ReadAction: value.read_action,
            WriteAction: value.write_action,
            Reserved: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum MsrAction {
    ArchitectureDefault = 0,
    IgnoreWriteReadZero = 1,
    Exit = 2,
}

impl From<WHV_MSR_ACTION> for MsrAction {
    fn from(value: WHV_MSR_ACTION) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0x00000000 => Self::ArchitectureDefault,
            0x00000001 => Self::IgnoreWriteReadZero,
            0x00000002 => Self::Exit,
            _ => unreachable!(),
        }
    }
}

impl From<MsrAction> for WHV_MSR_ACTION {
    fn from(value: MsrAction) -> Self {
        // TODO: Why cant the repr(i32) expose the implicit conversion?
        Self(value as i32)
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum X64LocalApicEmulationMode {
    None,
    XApic,
    X2Apic,
}

impl From<WHV_X64_LOCAL_APIC_EMULATION_MODE> for X64LocalApicEmulationMode {
    fn from(value: WHV_X64_LOCAL_APIC_EMULATION_MODE) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0x00000000 => Self::None,
            0x00000001 => Self::XApic,
            0x00000002 => Self::X2Apic,
            _ => unreachable!(),
        }
    }
}

impl From<X64LocalApicEmulationMode> for WHV_X64_LOCAL_APIC_EMULATION_MODE {
    fn from(value: X64LocalApicEmulationMode) -> Self {
        // TODO: Why cant the repr(i32) expose the implicit conversion?
        Self(value as i32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProcessorFeaturesBanks {
    pub banks_count: u32,
    pub bank_0: ProcessorFeatures,
    pub bank_1: ProcessorFeatures1,
}

impl From<WHV_PROCESSOR_FEATURES_BANKS> for ProcessorFeaturesBanks {
    fn from(value: WHV_PROCESSOR_FEATURES_BANKS) -> Self {
        Self {
            banks_count: value.BanksCount,
            bank_0: unsafe { value.Anonymous.Anonymous.Bank0.into() },
            bank_1: unsafe { value.Anonymous.Anonymous.Bank1.into() },
        }
    }
}

impl From<ProcessorFeaturesBanks> for WHV_PROCESSOR_FEATURES_BANKS {
    fn from(value: ProcessorFeaturesBanks) -> Self {
        Self {
            BanksCount: value.banks_count,
            Reserved0: 0,
            Anonymous: WHV_PROCESSOR_FEATURES_BANKS_0 {
                Anonymous: WHV_PROCESSOR_FEATURES_BANKS_0_0 {
                    Bank0: value.bank_0.into(),
                    Bank1: value.bank_1.into(),
                },
            },
        }
    }
}

#[derive(Debug)]
pub enum PartitionProperty {
    ExtendedVmExits(ExtendedVmExits),
    ProcessorFeatures(ProcessorFeatures),
    SyntheticProcessorFeaturesBanks(SyntheticProcessorFeaturesBanks),
    ProcessorXsaveFeatures(ProcessorXsaveFeatures),
    ProcessorClFlushSize(u8),
    ProcessorCount(u32),
    CpuidExitList([u32; 1]),                // TODO: Wrap
    CpuidResultList([X64CpuidResult; 1]),   // TODO: Is this always 1? or is that placeholder...
    CpuidResultList2([X64CpuidResult2; 1]), // TODO: Is this always 1? or is that placeholder..
    MsrActionList([MsrActionEntry; 1]),     // TODO: Is this always 1? or is that placeholder..
    UnimplementedMsrAction(MsrAction),
    ExceptionExitBitmap(u64), // TODO: This is a bitflag?
    LocalApicEmulationMode(X64LocalApicEmulationMode),
    SeparateSecurityDomain(bool),
    NestedVirtualization(bool),
    X64MsrExitBitmap(X64MsrExitBitmap),
    ProcessorClockFrequency(u64),
    InterruptClockFrequency(u64),
    ApicRemoteRead(bool),
    ProcessorFeaturesBanks(ProcessorFeaturesBanks),
    ReferenceTime(u64),
    PrimaryNumaNode(u16),
    CpuReserve(u32),
    CpuCap(u32),
    CpuWeight(u32),
    CpuGroupId(u64),
    ProcessorFrequencyCap(u32),
    AllowDeviceAssignment(bool),
    ProcessorPerfmonFeatures(ProcessorPerfmonFeatures),
    DisableSmt(bool),
}

impl PartitionProperty {
    pub fn from_union(code: PartitionPropertyCode, raw_val: WHV_PARTITION_PROPERTY) -> Self {
        // SAFETY: The code corresponds to the union variant.
        unsafe {
            match code {
                PartitionPropertyCode::ExtendedVmExits => {
                    Self::ExtendedVmExits(raw_val.ExtendedVmExits.into())
                }
                PartitionPropertyCode::ExceptionExitBitmap => {
                    Self::ExceptionExitBitmap(raw_val.ExtendedVmExits.AsUINT64)
                }
                PartitionPropertyCode::SeparateSecurityDomain => {
                    Self::SeparateSecurityDomain(raw_val.SeparateSecurityDomain.as_bool())
                }
                PartitionPropertyCode::NestedVirtualization => {
                    Self::NestedVirtualization(raw_val.NestedVirtualization.as_bool())
                }
                PartitionPropertyCode::X64MsrExitBitmap => {
                    Self::X64MsrExitBitmap(raw_val.X64MsrExitBitmap.into())
                }
                PartitionPropertyCode::PrimaryNumaNode => {
                    Self::PrimaryNumaNode(raw_val.PrimaryNumaNode)
                }
                PartitionPropertyCode::CpuReserve => Self::CpuReserve(raw_val.CpuReserve),
                PartitionPropertyCode::CpuCap => Self::CpuCap(raw_val.CpuCap),
                PartitionPropertyCode::CpuWeight => Self::CpuWeight(raw_val.CpuWeight),
                PartitionPropertyCode::CpuGroupId => Self::CpuGroupId(raw_val.CpuGroupId),
                PartitionPropertyCode::ProcessorFrequencyCap => {
                    Self::ProcessorFrequencyCap(raw_val.ProcessorFrequencyCap)
                }
                PartitionPropertyCode::AllowDeviceAssignment => {
                    Self::AllowDeviceAssignment(raw_val.AllowDeviceAssignment.as_bool())
                }
                PartitionPropertyCode::DisableSmt => Self::DisableSmt(raw_val.DisableSmt.as_bool()),
                PartitionPropertyCode::ProcessorFeatures => {
                    Self::ProcessorFeatures(raw_val.ProcessorFeatures.into())
                }
                PartitionPropertyCode::ProcessorClFlushSize => {
                    Self::ProcessorClFlushSize(raw_val.ProcessorClFlushSize)
                }
                PartitionPropertyCode::CpuidExitList => Self::CpuidExitList(raw_val.CpuidExitList),
                PartitionPropertyCode::CpuidResultList => {
                    Self::CpuidResultList(raw_val.CpuidResultList.map(|v| v.into()))
                }
                PartitionPropertyCode::LocalApicEmulationMode => {
                    Self::LocalApicEmulationMode(raw_val.LocalApicEmulationMode.into())
                }
                PartitionPropertyCode::ProcessorXsaveFeatures => {
                    Self::ProcessorXsaveFeatures(raw_val.ProcessorXsaveFeatures.into())
                }
                PartitionPropertyCode::ProcessorClockFrequency => {
                    Self::ProcessorClockFrequency(raw_val.ProcessorClockFrequency)
                }
                PartitionPropertyCode::InterruptClockFrequency => {
                    Self::InterruptClockFrequency(raw_val.InterruptClockFrequency)
                }
                PartitionPropertyCode::ApicRemoteRead => {
                    Self::ApicRemoteRead(raw_val.ApicRemoteRead.as_bool())
                }
                PartitionPropertyCode::ProcessorFeaturesBanks => {
                    Self::ProcessorFeaturesBanks(raw_val.ProcessorFeaturesBanks.into())
                }
                PartitionPropertyCode::ReferenceTime => Self::ReferenceTime(raw_val.ReferenceTime),
                PartitionPropertyCode::SyntheticProcessorFeaturesBanks => {
                    Self::SyntheticProcessorFeaturesBanks(
                        raw_val.SyntheticProcessorFeaturesBanks.into(),
                    )
                }
                PartitionPropertyCode::CpuidResultList2 => {
                    Self::CpuidResultList2(raw_val.CpuidResultList2.map(|v| v.into()))
                }
                PartitionPropertyCode::ProcessorPerfmonFeatures => {
                    Self::ProcessorPerfmonFeatures(raw_val.ProcessorPerfmonFeatures.into())
                }
                PartitionPropertyCode::MsrActionList => {
                    Self::MsrActionList(raw_val.MsrActionList.map(|v| v.into()))
                }
                PartitionPropertyCode::UnimplementedMsrAction => {
                    Self::UnimplementedMsrAction(raw_val.UnimplementedMsrAction.into())
                }
                PartitionPropertyCode::ProcessorCount => {
                    Self::ProcessorCount(raw_val.ProcessorCount)
                }
            }
        }
    }

    // TODO: Mark availibility, if it can be used after setup.
    pub const fn availibility(&self) -> PartitionPropertyAvailability {
        match self {
            PartitionProperty::ExtendedVmExits(_) => todo!(),
            PartitionProperty::ProcessorFeatures(_) => todo!(),
            PartitionProperty::SyntheticProcessorFeaturesBanks(_) => todo!(),
            PartitionProperty::ProcessorXsaveFeatures(_) => todo!(),
            PartitionProperty::ProcessorClFlushSize(_) => todo!(),
            PartitionProperty::ProcessorCount(_) => todo!(),
            PartitionProperty::CpuidExitList(_) => todo!(),
            PartitionProperty::CpuidResultList(_) => todo!(),
            PartitionProperty::CpuidResultList2(_) => todo!(),
            PartitionProperty::MsrActionList(_) => todo!(),
            PartitionProperty::UnimplementedMsrAction(_) => todo!(),
            PartitionProperty::ExceptionExitBitmap(_) => todo!(),
            PartitionProperty::LocalApicEmulationMode(_) => todo!(),
            PartitionProperty::SeparateSecurityDomain(_) => todo!(),
            PartitionProperty::NestedVirtualization(_) => todo!(),
            PartitionProperty::X64MsrExitBitmap(_) => todo!(),
            PartitionProperty::ProcessorClockFrequency(_) => todo!(),
            PartitionProperty::InterruptClockFrequency(_) => todo!(),
            PartitionProperty::ApicRemoteRead(_) => todo!(),
            PartitionProperty::ProcessorFeaturesBanks(_) => todo!(),
            PartitionProperty::ReferenceTime(_) => todo!(),
            PartitionProperty::PrimaryNumaNode(_) => todo!(),
            PartitionProperty::CpuReserve(_) => todo!(),
            PartitionProperty::CpuCap(_) => todo!(),
            PartitionProperty::CpuWeight(_) => todo!(),
            PartitionProperty::CpuGroupId(_) => todo!(),
            PartitionProperty::ProcessorFrequencyCap(_) => todo!(),
            PartitionProperty::AllowDeviceAssignment(_) => todo!(),
            PartitionProperty::ProcessorPerfmonFeatures(_) => todo!(),
            PartitionProperty::DisableSmt(_) => todo!(),
        }
    }

    pub const fn code(&self) -> PartitionPropertyCode {
        match self {
            PartitionProperty::ExtendedVmExits(_) => PartitionPropertyCode::ExtendedVmExits,
            PartitionProperty::ProcessorFeatures(_) => PartitionPropertyCode::ProcessorFeatures,
            PartitionProperty::SyntheticProcessorFeaturesBanks(_) => {
                PartitionPropertyCode::SyntheticProcessorFeaturesBanks
            }
            PartitionProperty::ProcessorXsaveFeatures(_) => {
                PartitionPropertyCode::ProcessorXsaveFeatures
            }
            PartitionProperty::ProcessorClFlushSize(_) => {
                PartitionPropertyCode::ProcessorClFlushSize
            }
            PartitionProperty::ProcessorCount(_) => PartitionPropertyCode::ProcessorCount,
            PartitionProperty::CpuidExitList(_) => PartitionPropertyCode::CpuidExitList,
            PartitionProperty::CpuidResultList(_) => PartitionPropertyCode::CpuidResultList,
            PartitionProperty::CpuidResultList2(_) => PartitionPropertyCode::CpuidResultList2,
            PartitionProperty::MsrActionList(_) => PartitionPropertyCode::MsrActionList,
            PartitionProperty::UnimplementedMsrAction(_) => {
                PartitionPropertyCode::UnimplementedMsrAction
            }
            PartitionProperty::ExceptionExitBitmap(_) => PartitionPropertyCode::ExceptionExitBitmap,
            PartitionProperty::LocalApicEmulationMode(_) => {
                PartitionPropertyCode::LocalApicEmulationMode
            }
            PartitionProperty::SeparateSecurityDomain(_) => {
                PartitionPropertyCode::SeparateSecurityDomain
            }
            PartitionProperty::NestedVirtualization(_) => {
                PartitionPropertyCode::NestedVirtualization
            }
            PartitionProperty::X64MsrExitBitmap(_) => PartitionPropertyCode::X64MsrExitBitmap,
            PartitionProperty::ProcessorClockFrequency(_) => {
                PartitionPropertyCode::ProcessorClockFrequency
            }
            PartitionProperty::InterruptClockFrequency(_) => {
                PartitionPropertyCode::InterruptClockFrequency
            }
            PartitionProperty::ApicRemoteRead(_) => PartitionPropertyCode::ApicRemoteRead,
            PartitionProperty::ProcessorFeaturesBanks(_) => {
                PartitionPropertyCode::ProcessorFeaturesBanks
            }
            PartitionProperty::ReferenceTime(_) => PartitionPropertyCode::ReferenceTime,
            PartitionProperty::PrimaryNumaNode(_) => PartitionPropertyCode::PrimaryNumaNode,
            PartitionProperty::CpuReserve(_) => PartitionPropertyCode::CpuReserve,
            PartitionProperty::CpuCap(_) => PartitionPropertyCode::CpuCap,
            PartitionProperty::CpuWeight(_) => PartitionPropertyCode::CpuWeight,
            PartitionProperty::CpuGroupId(_) => PartitionPropertyCode::CpuGroupId,
            PartitionProperty::ProcessorFrequencyCap(_) => {
                PartitionPropertyCode::ProcessorFrequencyCap
            }
            PartitionProperty::AllowDeviceAssignment(_) => {
                PartitionPropertyCode::AllowDeviceAssignment
            }
            PartitionProperty::ProcessorPerfmonFeatures(_) => {
                PartitionPropertyCode::ProcessorPerfmonFeatures
            }
            PartitionProperty::DisableSmt(_) => PartitionPropertyCode::DisableSmt,
        }
    }
}

impl From<PartitionProperty> for WHV_PARTITION_PROPERTY {
    fn from(value: PartitionProperty) -> Self {
        match value {
            PartitionProperty::ExtendedVmExits(v) => Self {
                ExtendedVmExits: v.into(),
            },
            PartitionProperty::ProcessorFeatures(v) => Self {
                ProcessorFeatures: v.into(),
            },
            PartitionProperty::SyntheticProcessorFeaturesBanks(v) => Self {
                SyntheticProcessorFeaturesBanks: v.into(),
            },
            PartitionProperty::ProcessorXsaveFeatures(v) => Self {
                ProcessorXsaveFeatures: v.into(),
            },
            PartitionProperty::ProcessorClFlushSize(v) => Self {
                ProcessorClFlushSize: v,
            },
            PartitionProperty::ProcessorCount(v) => Self { ProcessorCount: v },
            // TODO: Redo this...
            PartitionProperty::CpuidExitList(v) => Self { CpuidExitList: v },
            PartitionProperty::CpuidResultList(v) => Self {
                CpuidResultList: v.map(|r| r.into()),
            },
            PartitionProperty::CpuidResultList2(v) => Self {
                CpuidResultList2: v.map(|r| r.into()),
            },
            PartitionProperty::MsrActionList(v) => Self {
                MsrActionList: v.map(|r| r.into()),
            },
            PartitionProperty::UnimplementedMsrAction(v) => Self {
                UnimplementedMsrAction: v.into(),
            },
            PartitionProperty::ExceptionExitBitmap(v) => Self {
                ExceptionExitBitmap: v,
            },
            PartitionProperty::LocalApicEmulationMode(v) => Self {
                LocalApicEmulationMode: v.into(),
            },
            PartitionProperty::SeparateSecurityDomain(v) => Self {
                SeparateSecurityDomain: v.into(),
            },
            PartitionProperty::NestedVirtualization(v) => Self {
                NestedVirtualization: v.into(),
            },
            PartitionProperty::X64MsrExitBitmap(v) => Self {
                X64MsrExitBitmap: v.into(),
            },
            PartitionProperty::ProcessorClockFrequency(v) => Self {
                ProcessorClockFrequency: v,
            },
            PartitionProperty::InterruptClockFrequency(v) => Self {
                InterruptClockFrequency: v,
            },
            PartitionProperty::ApicRemoteRead(v) => Self {
                ApicRemoteRead: v.into(),
            },
            PartitionProperty::ProcessorFeaturesBanks(v) => Self {
                ProcessorFeaturesBanks: v.into(),
            },
            PartitionProperty::ReferenceTime(v) => Self { ReferenceTime: v },
            PartitionProperty::PrimaryNumaNode(v) => Self { PrimaryNumaNode: v },
            PartitionProperty::CpuReserve(v) => Self { CpuReserve: v },
            PartitionProperty::CpuCap(v) => Self { CpuCap: v },
            PartitionProperty::CpuWeight(v) => Self { CpuWeight: v },
            PartitionProperty::CpuGroupId(v) => Self { CpuGroupId: v },
            PartitionProperty::ProcessorFrequencyCap(v) => Self {
                ProcessorFrequencyCap: v,
            },
            PartitionProperty::AllowDeviceAssignment(v) => Self {
                AllowDeviceAssignment: v.into(),
            },
            PartitionProperty::ProcessorPerfmonFeatures(v) => Self {
                ProcessorPerfmonFeatures: v.into(),
            },
            PartitionProperty::DisableSmt(v) => Self {
                DisableSmt: v.into(),
            },
        }
    }
}

#[derive(Debug)]
pub struct PartitionHandle(pub WHV_PARTITION_HANDLE);

impl PartitionHandle {
    pub fn new() -> Result<Self> {
        let raw_handle = unsafe { WHvCreatePartition()? };
        Ok(Self(raw_handle))
    }
}

impl From<PartitionHandle> for WHV_PARTITION_HANDLE {
    fn from(handle: PartitionHandle) -> Self {
        handle.0
    }
}

impl From<WHV_PARTITION_HANDLE> for PartitionHandle {
    fn from(raw_handle: WHV_PARTITION_HANDLE) -> Self {
        Self(raw_handle)
    }
}

impl Drop for PartitionHandle {
    fn drop(&mut self) {
        // TODO: Error handling... in drop?
        let _ = unsafe { WHvDeletePartition(self.0) };
    }
}

pub struct PartitionBuilder {
    arc_handle: Arc<PartitionHandle>,
}

impl PartitionBuilder {
    pub fn new() -> Result<Self> {
        Ok(Self {
            arc_handle: Arc::new(PartitionHandle::new()?),
        })
    }

    // TODO: Be able to query properties? Or should we assume nothing about the default state of the partition.
    // TODO: Should we set processor count to 1 by default so that immediately calling [PartitionBuilder::setup] without setting processor count works?

    pub fn property(self, prop: PartitionProperty) -> Result<Self> {
        unsafe {
            WHvSetPartitionProperty(
                WHV_PARTITION_HANDLE::from(self.arc_handle.0),
                prop.code().into(),
                std::mem::transmute(&WHV_PARTITION_PROPERTY::from(prop)),
                std::mem::size_of::<WHV_PARTITION_PROPERTY>().try_into()?,
            )?;
        };
        Ok(self)
    }

    pub fn setup(self) -> Result<Partition> {
        // TODO: Error handling.
        // TODO: Should we check processor count here?
        let _ = unsafe { WHvSetupPartition(self.arc_handle.0)? };
        Ok(Partition {
            arc_handle: self.arc_handle.clone(),
            memory_regions: Vec::new(),
        })
    }
}

/// A setup partition, ready to create virtual cpu's.
#[derive(Debug)]
pub struct Partition {
    arc_handle: Arc<PartitionHandle>,
    memory_regions: Vec<MemoryRegion>,
}

impl Partition {
    // TODO: Add a `from_` or `new` function, disallow struct initialization (i.e. [PartitionBuilder::setup]).

    pub fn query_property(&self, prop_code: PartitionPropertyCode) -> Result<PartitionProperty> {
        let raw_property: WHV_PARTITION_PROPERTY = Default::default();

        unsafe {
            WHvGetPartitionProperty(
                self.arc_handle.0,
                prop_code.into(),
                std::mem::transmute(&raw_property),
                std::mem::size_of::<WHV_PARTITION_PROPERTY>().try_into()?,
                None,
            )?;
        }

        Ok(PartitionProperty::from_union(prop_code, raw_property))
    }

    pub fn set_property(&mut self, prop: PartitionProperty) -> Result<()> {
        match prop.availibility() {
            PartitionPropertyAvailability::BeforeSetup => {
                Err(crate::Error::IncompatiblePropertyAvailibility(prop))
            }
            PartitionPropertyAvailability::AfterSetup => {
                unsafe {
                    WHvSetPartitionProperty(
                        self.arc_handle.0,
                        prop.code().into(),
                        std::mem::transmute(&WHV_PARTITION_PROPERTY::from(prop)),
                        std::mem::size_of::<WHV_PARTITION_PROPERTY>().try_into()?,
                    )?;
                }
                Ok(())
            }
        }
    }

    pub fn map_memory_region(&mut self, memory_region: MemoryRegion) -> Result<()> {
        unsafe {
            WHvMapGpaRange(
                self.arc_handle.0,
                memory_region.address as *const _,
                memory_region.guest_address.try_into()?,
                memory_region.size.try_into()?,
                memory_region.flags.into(),
            )?;
        }

        self.memory_regions.push(memory_region);

        Ok(())
    }

    pub fn create_virtual_processor(&mut self, index: u32) -> Result<VirtualProcessor> {
        // Check to make sure we have processor count at or larger than index.
        match self.query_property(PartitionPropertyCode::ProcessorCount)? {
            PartitionProperty::ProcessorCount(count) if index >= count => {
                Err(Error::InvalidVpIndex(index, count))
            }
            _ => Ok(()),
        }?;

        // TODO: Safety docs.
        unsafe { WHvCreateVirtualProcessor(self.arc_handle.0, index, 0)? };
        Ok(VirtualProcessor::new(self.arc_handle.clone(), index))
    }
}
