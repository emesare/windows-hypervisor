// NOTE: We used numerical bit values instead of masks due to the 64 bit flags, otherwise this would be unreadable.

use windows::Win32::System::Hypervisor::{
    WHV_CAPABILITY_FEATURES, WHV_EXTENDED_VM_EXITS, WHV_MAP_GPA_RANGE_FLAGS,
    WHV_MEMORY_ACCESS_INFO, WHV_PROCESSOR_FEATURES, WHV_PROCESSOR_FEATURES1,
    WHV_PROCESSOR_PERFMON_FEATURES, WHV_PROCESSOR_XSAVE_FEATURES, WHV_SYNTHETIC_PROCESSOR_FEATURES,
    WHV_VP_EXCEPTION_INFO, WHV_X64_CPUID_RESULT2_FLAGS, WHV_X64_INTERRUPT_STATE_REGISTER,
    WHV_X64_IO_PORT_ACCESS_INFO, WHV_X64_MSR_ACCESS_INFO, WHV_X64_MSR_EXIT_BITMAP,
    WHV_X64_RDTSC_INFO, WHV_X64_SEGMENT_REGISTER_0, WHV_X64_VP_EXECUTION_STATE,
};

use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct CapabilityFeatures: u64 {
        const PartialUnmap = 1;
        const LocalApicEmulation = 2;
        const Xsave = 3;
        const DirtyPageTracking = 4;
        // TODO: This looks to be wrong...
        const SpeculationControl = 5;
        const ApicRemoteRead = 6;
        const IdleSuspend = 7;
        const VirtualPciDeviceSupport = 8;
        const IommuSupport = 9;
        const VpHotAddRemove = 10;
    }
}

impl From<WHV_CAPABILITY_FEATURES> for CapabilityFeatures {
    fn from(value: WHV_CAPABILITY_FEATURES) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<CapabilityFeatures> for WHV_CAPABILITY_FEATURES {
    fn from(value: CapabilityFeatures) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MapGpaRangeFlags: i32 {
        const None = 0x0;
        const Read = 0x1;
        const Write = 0x2;
        const Execute = 0x4;
        const TrackDirtyPages = 0x8;
    }
}

impl From<WHV_MAP_GPA_RANGE_FLAGS> for MapGpaRangeFlags {
    fn from(value: WHV_MAP_GPA_RANGE_FLAGS) -> Self {
        Self::from_bits_retain(value.0)
    }
}

impl From<MapGpaRangeFlags> for WHV_MAP_GPA_RANGE_FLAGS {
    fn from(value: MapGpaRangeFlags) -> Self {
        Self(value.bits())
    }
}

bitflags! {
    /// Represents a set of additional exit reasons, can be adjusted by [PartitionBuilder::set_extended_vm_exits].
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ExtendedVmExits: u64 {
        /// Exit whenever the CPUID is accessed.
        ///
        /// NOTE: Must be set in WHvCapabilityCodeExtendedVmExits to have an effect.
        const CpuId = 1;
        /// Exit whenever an MSR is accessed.
        ///
        /// NOTE: Must be set in WHvCapabilityCodeExtendedVmExits to have an effect.
        const Msr = 2;
        const Exception = 3;
        /// Exit whenever the RDTSC is accessed.
        ///
        /// NOTE: Must be set in WHvCapabilityCodeExtendedVmExits to have an effect.
        const Rdtsc = 4;
        const ApicSmiTrap = 5;
        const Hypercall = 6;
        const ApicInitSipiTrap = 7;
        const ApicWriteLint0Trap = 8;
        const ApicWriteLint1Trap = 9;
        const ApicWriteSvrTrap = 10;
        const UnknownSynicConnection = 11;
        const RetargetUnknownVpciDevice = 12;
        const ApicWriteLdrTrap = 13;
        const ApicWriteDfrTrap = 14;
        const GpaAccessFault = 15;
    }
}

impl From<WHV_EXTENDED_VM_EXITS> for ExtendedVmExits {
    fn from(value: WHV_EXTENDED_VM_EXITS) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<ExtendedVmExits> for WHV_EXTENDED_VM_EXITS {
    fn from(value: ExtendedVmExits) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ProcessorFeatures: u64 {
        const Sse3Support = 1;
        const LahfSahfSupport = 2;
        const Ssse3Support = 3;
        const Sse4_1Support = 4;
        const Sse4_2Support = 5;
        const Sse4aSupport = 6;
        const XopSupport = 7;
        const PopCntSupport = 8;
        const Cmpxchg16bSupport = 9;
        const Altmovcr8Support = 10;
        const LzcntSupport = 11;
        const MisAlignSseSupport = 12;
        const MmxExtSupport = 13;
        const Amd3DNowSupport = 14;
        const ExtendedAmd3DNowSupport = 15;
        const Page1GbSupport = 16;
        const AesSupport = 17;
        const PclmulqdqSupport = 18;
        const PcidSupport = 19;
        const Fma4Support = 20;
        const F16CSupport = 21;
        const RdRandSupport = 22;
        const RdWrFsGsSupport = 23;
        const SmepSupport = 24;
        const EnhancedFastStringSupport = 25;
        const Bmi1Support = 26;
        const Bmi2Support = 27;
        const Reserved1 = 28;
        const MovbeSupport = 29;
        const Npiep1Support = 30;
        const DepX87FPUSaveSupport = 31;
        const RdSeedSupport = 32;
        const AdxSupport = 33;
        const IntelPrefetchSupport = 34;
        const SmapSupport = 35;
        const HleSupport = 36;
        const RtmSupport = 37;
        const RdtscpSupport = 38;
        const ClflushoptSupport = 39;
        const ClwbSupport = 40;
        const ShaSupport = 41;
        const X87PointersSavedSupport = 42;
        const InvpcidSupport = 43;
        const IbrsSupport = 44;
        const StibpSupport = 45;
        const IbpbSupport = 46;
        const Reserved2 = 47;
        const SsbdSupport = 48;
        const FastShortRepMovSupport = 49;
        const Reserved3 = 50;
        const RdclNo = 51;
        const IbrsAllSupport = 52;
        const Reserved4 = 53;
        const SsbNo = 54;
        const RsbANo = 55;
    }
}

impl From<WHV_PROCESSOR_FEATURES> for ProcessorFeatures {
    fn from(value: WHV_PROCESSOR_FEATURES) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<ProcessorFeatures> for WHV_PROCESSOR_FEATURES {
    fn from(value: ProcessorFeatures) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ProcessorFeatures1: u64 {
        const ACountMCountSupport = 1;
        const TscInvariantSupport = 2;
        const ClZeroSupport = 3;
        const RdpruSupport = 4;
        const Reserved2 = 5 | 6;
        const NestedVirtSupport = 7;
        const PsfdSupport = 8;
        const CetSsSupport = 9;
        const CetIbtSupport = 10;
        const VmxExceptionInjectSupport = 11;
        const Reserved4 = 12;
        const UmwaitTpauseSupport = 13;
        const MovdiriSupport = 14;
        const Movdir64bSupport = 15;
        const CldemoteSupport = 16;
        const SerializeSupport = 17;
        const TscDeadlineTmrSupport = 18;
        const TscAdjustSupport = 19;
        const FZLRepMovsb = 20;
        const FSRepStosb = 21;
        const FSRepCmpsb = 22;
        const TsxLdTrkSupport = 23;
    }
}

impl From<WHV_PROCESSOR_FEATURES1> for ProcessorFeatures1 {
    fn from(value: WHV_PROCESSOR_FEATURES1) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<ProcessorFeatures1> for WHV_PROCESSOR_FEATURES1 {
    fn from(value: ProcessorFeatures1) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ProcessorXsaveFeatures: u64 {
        const XsaveSupport = 1;
        const XsaveoptSupport = 2;
        const AvxSupport = 3;
        const Avx2Support = 4;
        const FmaSupport = 5;
        const MpxSupport = 6;
        const Avx512Support = 7;
        const Avx512DQSupport = 8;
        const Avx512CDSupport = 9;
        const Avx512BWSupport = 10;
        const Avx512VLSupport = 11;
        const XsaveCompSupport = 12;
        const XsaveSupervisorSupport = 13;
        const Xcr1Support = 14;
        const Avx512BitalgSupport = 15;
        const Avx512IfmaSupport = 16;
        const Avx512VBmiSupport = 17;
        const Avx512VBmi2Support = 18;
        const Avx512VnniSupport = 19;
        const GfniSupport = 20;
        const VaesSupport = 21;
        const Avx512VPopcntdqSupport = 22;
        const VpclmulqdqSupport = 23;
    }
}

impl From<WHV_PROCESSOR_XSAVE_FEATURES> for ProcessorXsaveFeatures {
    fn from(value: WHV_PROCESSOR_XSAVE_FEATURES) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<ProcessorXsaveFeatures> for WHV_PROCESSOR_XSAVE_FEATURES {
    fn from(value: ProcessorXsaveFeatures) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

// TODO: Keep Vp prefix?
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct X64ExecutionState: u16 {
        const Cpl = 1 | 2;
        const Cr0Pe = 3;
        const Cr0Am = 4;
        const EferLma = 5;
        const DebugActive = 6;
        const InterruptionPending = 7;
        const Reserved0 = 8 | 9 | 10 | 11 | 12;
        const InterruptShadow = 13;
    }
}

impl From<WHV_X64_VP_EXECUTION_STATE> for X64ExecutionState {
    fn from(value: WHV_X64_VP_EXECUTION_STATE) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT16 };
        Self::from_bits_retain(bits)
    }
}

impl From<X64ExecutionState> for WHV_X64_VP_EXECUTION_STATE {
    fn from(value: X64ExecutionState) -> Self {
        Self {
            AsUINT16: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct X64SegmentRegisterAttributes: u16 {
        const SegmentType = 1 | 2 | 3 | 4;
        const NonSystemSegment = 5;
        const DescriptorPrivilegeLevel = 6 | 7;
        const Present = 8;
        const Reserved = 9 | 10 | 11 | 12;
        const Available = 13;
        const Long = 14;
        const Default = 15;
        const Granularity = 16;
    }
}

impl From<WHV_X64_SEGMENT_REGISTER_0> for X64SegmentRegisterAttributes {
    fn from(value: WHV_X64_SEGMENT_REGISTER_0) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.Attributes };
        Self::from_bits_retain(bits)
    }
}

impl From<X64SegmentRegisterAttributes> for WHV_X64_SEGMENT_REGISTER_0 {
    fn from(value: X64SegmentRegisterAttributes) -> Self {
        Self {
            Attributes: value.bits(),
        }
    }
}

// TODO: This is NOT working... its printing out some ugly 0x2 value, it should give us access type...
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MemoryAccessInfo: u32 {
        // TODO: Provide helper for access type?
        const AccessType = 1 | 2;
        const GpaUnmapped = 5;
        const GpaValid = 6 | 7;
    }
}

impl From<WHV_MEMORY_ACCESS_INFO> for MemoryAccessInfo {
    fn from(value: WHV_MEMORY_ACCESS_INFO) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT32 };
        Self::from_bits_retain(bits)
    }
}

impl From<MemoryAccessInfo> for WHV_MEMORY_ACCESS_INFO {
    fn from(value: MemoryAccessInfo) -> Self {
        Self {
            AsUINT32: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct IoPortAccessInfo: u32 {
        const IsWrite = 1;
        const AccessSize = 2 | 3 | 4;
        const StringOp = 5;
        const RepPrefix = 6;
    }
}

impl From<WHV_X64_IO_PORT_ACCESS_INFO> for IoPortAccessInfo {
    fn from(value: WHV_X64_IO_PORT_ACCESS_INFO) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT32 };
        Self::from_bits_retain(bits)
    }
}

impl From<IoPortAccessInfo> for WHV_X64_IO_PORT_ACCESS_INFO {
    fn from(value: IoPortAccessInfo) -> Self {
        Self {
            AsUINT32: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MsrAccessInfo: u32 {
        const IsWrite = 1;
    }
}

impl From<WHV_X64_MSR_ACCESS_INFO> for MsrAccessInfo {
    fn from(value: WHV_X64_MSR_ACCESS_INFO) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT32 };
        Self::from_bits_retain(bits)
    }
}

impl From<MsrAccessInfo> for WHV_X64_MSR_ACCESS_INFO {
    fn from(value: MsrAccessInfo) -> Self {
        Self {
            AsUINT32: value.bits(),
        }
    }
}

// TODO: We keep Vp here for obvious reasons, i guess keep prefix everywhere?
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct VpExceptionInfo: u32 {
        const ErrorCodeValid = 1;
        const SoftwareException = 1;
    }
}

impl From<WHV_VP_EXCEPTION_INFO> for VpExceptionInfo {
    fn from(value: WHV_VP_EXCEPTION_INFO) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT32 };
        Self::from_bits_retain(bits)
    }
}

impl From<VpExceptionInfo> for WHV_VP_EXCEPTION_INFO {
    fn from(value: VpExceptionInfo) -> Self {
        Self {
            AsUINT32: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct RdtscInfo: u64 {
        const IsRdtscp = 1;
    }
}

impl From<WHV_X64_RDTSC_INFO> for RdtscInfo {
    fn from(value: WHV_X64_RDTSC_INFO) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<RdtscInfo> for WHV_X64_RDTSC_INFO {
    fn from(value: RdtscInfo) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SyntheticProcessorFeatures: u64 {
        const HypervisorPresent = 1;
        const Hv1 = 2;
        const AccessVpRunTimeReg = 3;
        const AccessPartitionReferenceCounter = 4;
        const AccessSynicRegs = 5;
        const AccessSyntheticTimerRegs = 6;
        const AccessIntrCtrlRegs = 7;
        const ReservedZ6 = 8;
        const AccessHypercallRegs = 9;
        const AccessVpIndex = 10;
        const AccessPartitionReferenceTsc = 11;
        const AccessGuestIdleReg = 12;
        const AccessFrequencyRegs = 13;
        const ReservedZ10 = 14;
        const ReservedZ11 = 15;
        const ReservedZ12 = 16;
        const ReservedZ13 = 17;
        const ReservedZ14 = 18;
        const EnableExtendedGvaRangesForFlushVirtualAddressList = 19;
        const ReservedZ15 = 20;
        const ReservedZ16 = 21;
        const ReservedZ17 = 22;
        const FastHypercallOutput = 23;
        const ReservedZ19 = 24;
        const ReservedZ20 = 25;
        const ReservedZ21 = 26;
        const DirectSyntheticTimers = 26;
        const ReservedZ23 = 27;
        const ExtendedProcessorMasks = 28;
        const TbFlushHypercalls = 29;
        const ReservedZ25 = 30;
        const SyntheticClusterIpi = 31;
        const NotifyLongSpinWait = 32;
        const QueryNumaDistance = 33;
        const SignalEvents = 34;
        const RetargetDeviceInterrupt = 35;
    }
}

impl From<WHV_SYNTHETIC_PROCESSOR_FEATURES> for SyntheticProcessorFeatures {
    fn from(value: WHV_SYNTHETIC_PROCESSOR_FEATURES) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<SyntheticProcessorFeatures> for WHV_SYNTHETIC_PROCESSOR_FEATURES {
    fn from(value: SyntheticProcessorFeatures) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct X64CpuidResult2Flags: i32 {
        const SubleafSpecific = 1;
        const VpSpecific = 2;
    }
}

impl From<WHV_X64_CPUID_RESULT2_FLAGS> for X64CpuidResult2Flags {
    fn from(value: WHV_X64_CPUID_RESULT2_FLAGS) -> Self {
        Self::from_bits_retain(value.0)
    }
}

impl From<X64CpuidResult2Flags> for WHV_X64_CPUID_RESULT2_FLAGS {
    fn from(value: X64CpuidResult2Flags) -> Self {
        Self(value.bits())
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct X64MsrExitBitmap: u64 {
        const UnhandledMsrs = 1;
        const TscMsrWrite = 2;
        const TscMsrRead = 3;
        const ApicBaseMsrWrite = 4;
        const MiscEnableMsrRead = 5;
        const McUpdatePatchLevelMsrRead = 6;
    }
}

impl From<WHV_X64_MSR_EXIT_BITMAP> for X64MsrExitBitmap {
    fn from(value: WHV_X64_MSR_EXIT_BITMAP) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<X64MsrExitBitmap> for WHV_X64_MSR_EXIT_BITMAP {
    fn from(value: X64MsrExitBitmap) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ProcessorPerfmonFeatures: u64 {
        const PmuSupport = 1;
        const LbrSupport = 2;
    }
}

impl From<WHV_PROCESSOR_PERFMON_FEATURES> for ProcessorPerfmonFeatures {
    fn from(value: WHV_PROCESSOR_PERFMON_FEATURES) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<ProcessorPerfmonFeatures> for WHV_PROCESSOR_PERFMON_FEATURES {
    fn from(value: ProcessorPerfmonFeatures) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InterruptStateRegister: u64 {
        const InterruptShadow = 1;
        const NmiMasked = 2;
    }
}

impl From<WHV_X64_INTERRUPT_STATE_REGISTER> for InterruptStateRegister {
    fn from(value: WHV_X64_INTERRUPT_STATE_REGISTER) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        let bits = unsafe { value.AsUINT64 };
        Self::from_bits_retain(bits)
    }
}

impl From<InterruptStateRegister> for WHV_X64_INTERRUPT_STATE_REGISTER {
    fn from(value: InterruptStateRegister) -> Self {
        Self {
            AsUINT64: value.bits(),
        }
    }
}
