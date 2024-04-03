use std::{fmt::Debug, sync::Arc};

use c2rust_bitfields::BitfieldStruct;
use windows::Win32::System::Hypervisor::{
    WHvDeleteVirtualProcessor, WHvGetVirtualProcessorRegisters, WHvRunVirtualProcessor,
    WHvSetVirtualProcessorRegisters, WHV_HYPERCALL_CONTEXT,
    WHV_HYPERCALL_CONTEXT_MAX_XMM_REGISTERS, WHV_MEMORY_ACCESS_CONTEXT, WHV_REGISTER_NAME,
    WHV_REGISTER_VALUE, WHV_RUN_VP_CANCELED_CONTEXT, WHV_RUN_VP_CANCEL_REASON,
    WHV_RUN_VP_EXIT_CONTEXT, WHV_RUN_VP_EXIT_CONTEXT_0, WHV_RUN_VP_EXIT_REASON,
    WHV_SYNIC_SINT_DELIVERABLE_CONTEXT, WHV_VP_EXCEPTION_CONTEXT, WHV_VP_EXIT_CONTEXT,
    WHV_X64_APIC_EOI_CONTEXT, WHV_X64_APIC_INIT_SIPI_CONTEXT, WHV_X64_APIC_SMI_CONTEXT,
    WHV_X64_APIC_WRITE_CONTEXT, WHV_X64_APIC_WRITE_TYPE, WHV_X64_CPUID_ACCESS_CONTEXT,
    WHV_X64_FP_CONTROL_STATUS_REGISTER, WHV_X64_FP_CONTROL_STATUS_REGISTER_0,
    WHV_X64_FP_CONTROL_STATUS_REGISTER_0_0, WHV_X64_INTERRUPTION_DELIVERABLE_CONTEXT,
    WHV_X64_IO_PORT_ACCESS_CONTEXT, WHV_X64_MSR_ACCESS_CONTEXT, WHV_X64_PENDING_INTERRUPTION_TYPE,
    WHV_X64_RDTSC_CONTEXT, WHV_X64_SEGMENT_REGISTER, WHV_X64_TABLE_REGISTER,
    WHV_X64_UNSUPPORTED_FEATURE_CODE, WHV_X64_UNSUPPORTED_FEATURE_CONTEXT,
    WHV_X64_XMM_CONTROL_STATUS_REGISTER, WHV_X64_XMM_CONTROL_STATUS_REGISTER_0,
    WHV_X64_XMM_CONTROL_STATUS_REGISTER_0_0,
};

use crate::{
    fields::{
        DeliverabilityNotificationsRegister, FpRegister, PendingExceptionEvent, PendingExtIntEvent,
        PendingInterruptionRegister,
    },
    flags::{
        InterruptStateRegister, IoPortAccessInfo, MemoryAccessInfo, MsrAccessInfo, RdtscInfo,
        VpExceptionInfo, X64ExecutionState, X64SegmentRegisterAttributes,
    },
    partition::PartitionHandle,
    Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunExitReason {
    None = 0x0,
    MemoryAccess = 0x1,
    X64IoPortAccess = 0x2,
    UnrecoverableException = 0x4,
    InvalidVpRegisterValue = 0x5,
    UnsupportedFeature = 0x6,
    X64InterruptWindow = 0x7,
    X64Halt = 0x8,
    X64ApicEoi = 0x9,
    SynicSintDeliverable = 0xA,
    X64MsrAccess = 0x1000,
    X64Cpuid = 0x1001,
    Exception = 0x1002,
    X64Rdtsc = 0x1003,
    X64ApicSmiTrap = 0x1004,
    Hypercall = 0x1005,
    X64ApicInitSipiTrap = 0x1006,
    X64ApicWriteTrap = 0x1007,
    Canceled = 0x2001,
}

impl From<WHV_RUN_VP_EXIT_REASON> for RunExitReason {
    fn from(value: WHV_RUN_VP_EXIT_REASON) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0x0 => Self::None,
            0x1 => Self::MemoryAccess,
            0x2 => Self::X64IoPortAccess,
            0x4 => Self::UnrecoverableException,
            0x5 => Self::InvalidVpRegisterValue,
            0x6 => Self::UnsupportedFeature,
            0x7 => Self::X64InterruptWindow,
            0x8 => Self::X64Halt,
            0x9 => Self::X64ApicEoi,
            0xA => Self::SynicSintDeliverable,
            0x1000 => Self::X64MsrAccess,
            0x1001 => Self::X64Cpuid,
            0x1002 => Self::Exception,
            0x1003 => Self::X64Rdtsc,
            0x1004 => Self::X64ApicSmiTrap,
            0x1005 => Self::Hypercall,
            0x1006 => Self::X64ApicInitSipiTrap,
            0x1007 => Self::X64ApicWriteTrap,
            0x2001 => Self::Canceled,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentRegister {
    pub base: u64,
    pub limit: u32,
    pub selector: u16,
    pub attributes: X64SegmentRegisterAttributes,
}

impl From<WHV_X64_SEGMENT_REGISTER> for SegmentRegister {
    fn from(value: WHV_X64_SEGMENT_REGISTER) -> Self {
        Self {
            base: value.Base,
            limit: value.Limit,
            selector: value.Selector,
            attributes: value.Anonymous.into(),
        }
    }
}

impl From<SegmentRegister> for WHV_X64_SEGMENT_REGISTER {
    fn from(value: SegmentRegister) -> Self {
        Self {
            Base: value.base,
            Limit: value.limit,
            Selector: value.selector,
            Anonymous: windows::Win32::System::Hypervisor::WHV_X64_SEGMENT_REGISTER_0 {
                Attributes: value.attributes.bits(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X64FpControlStatusRegister {
    pub control: u16,
    pub status: u16,
    pub tag: u8,
    pub last_op: u16,
    // TODO: So this is technically a union (rip, eip then cs)
    pub last_rip: u64,
}

impl From<WHV_X64_FP_CONTROL_STATUS_REGISTER> for X64FpControlStatusRegister {
    fn from(value: WHV_X64_FP_CONTROL_STATUS_REGISTER) -> Self {
        // SAFETY: The value only has one real "value" and then a reinterpreted raw value.
        unsafe {
            Self {
                control: value.Anonymous.FpControl,
                status: value.Anonymous.FpStatus,
                tag: value.Anonymous.FpTag,
                last_op: value.Anonymous.LastFpOp,
                last_rip: value.Anonymous.Anonymous.LastFpRip,
            }
        }
    }
}

impl From<X64FpControlStatusRegister> for WHV_X64_FP_CONTROL_STATUS_REGISTER {
    fn from(value: X64FpControlStatusRegister) -> Self {
        Self {
            Anonymous: WHV_X64_FP_CONTROL_STATUS_REGISTER_0 {
                FpControl: value.control,
                FpStatus: value.status,
                FpTag: value.tag,
                Reserved: 0,
                LastFpOp: value.last_op,
                Anonymous: WHV_X64_FP_CONTROL_STATUS_REGISTER_0_0 {
                    LastFpRip: value.last_rip,
                },
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X64XmmControlStatusRegister {
    // TODO: So this is technically a union (rdp, dp then ds)
    pub last_rdp: u64,
    pub status_control: u32,
    pub status_control_mask: u32,
}

impl From<WHV_X64_XMM_CONTROL_STATUS_REGISTER> for X64XmmControlStatusRegister {
    fn from(value: WHV_X64_XMM_CONTROL_STATUS_REGISTER) -> Self {
        // SAFETY: The value only has one real "value" and then a reinterpreted raw value.
        unsafe {
            Self {
                last_rdp: value.Anonymous.Anonymous.LastFpRdp,
                status_control: value.Anonymous.XmmStatusControl,
                status_control_mask: value.Anonymous.XmmStatusControlMask,
            }
        }
    }
}

impl From<X64XmmControlStatusRegister> for WHV_X64_XMM_CONTROL_STATUS_REGISTER {
    fn from(value: X64XmmControlStatusRegister) -> Self {
        Self {
            Anonymous: WHV_X64_XMM_CONTROL_STATUS_REGISTER_0 {
                Anonymous: WHV_X64_XMM_CONTROL_STATUS_REGISTER_0_0 {
                    LastFpRdp: value.last_rdp,
                },
                XmmStatusControl: value.status_control,
                XmmStatusControlMask: value.status_control_mask,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableRegister {
    pad: [u16; 3],
    limit: u16,
    base: u64,
}

impl From<WHV_X64_TABLE_REGISTER> for TableRegister {
    fn from(value: WHV_X64_TABLE_REGISTER) -> Self {
        Self {
            pad: value.Pad,
            limit: value.Limit,
            base: value.Base,
        }
    }
}

impl From<TableRegister> for WHV_X64_TABLE_REGISTER {
    fn from(value: TableRegister) -> Self {
        Self {
            Pad: value.pad,
            Limit: value.limit,
            Base: value.base,
        }
    }
}

#[derive(BitfieldStruct, Clone, Copy)]
pub struct ExitContext {
    pub execution_state: X64ExecutionState,
    #[bitfield(name = "instruction_len", ty = "u8", bits = "0..=3")]
    #[bitfield(name = "cr8", ty = "u8", bits = "4..=7")]
    _bitfield: [u8; 1],
    // TODO: Rename this to something more meaningful then add an alias for "cs"
    pub cs: SegmentRegister,
    pub rip: u64,
    pub rflags: u64,
}

impl Debug for ExitContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExitContext")
            .field("execution_state", &self.execution_state)
            .field("instruction_len", &self.instruction_len())
            .field("cr8", &self.cr8())
            .field("cs", &self.cs)
            .field("rip", &self.rip)
            .field("rflags", &self.rflags)
            .finish()
    }
}

impl From<WHV_VP_EXIT_CONTEXT> for ExitContext {
    fn from(value: WHV_VP_EXIT_CONTEXT) -> Self {
        Self {
            execution_state: value.ExecutionState.into(),
            _bitfield: [value._bitfield],
            cs: value.Cs.into(),
            rip: value.Rip,
            rflags: value.Rflags,
        }
    }
}
// TODO: Terrible name microsoft...
// TODO: Rename to ExitContextEvent?
#[derive(Debug, Clone, Copy)]
pub struct RunExitContext {
    pub exit_reason: RunExitReason,
    pub context: ExitContext,
    // TODO: Do we need this as an option? Or do all reasons have this... (See [RunExitContextExt::from_union] for more info)
    pub ext: Option<RunExitContextExt>,
}

impl From<WHV_RUN_VP_EXIT_CONTEXT> for RunExitContext {
    fn from(value: WHV_RUN_VP_EXIT_CONTEXT) -> Self {
        let exit_reason = RunExitReason::from(value.ExitReason);
        Self {
            exit_reason,
            context: ExitContext::from(value.VpContext),
            ext: RunExitContextExt::from_union(exit_reason, value.Anonymous),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryAccessContext {
    pub instruction_byte_count: u8,
    // TODO: Type this better, maybe these should be [Instruction]'s?
    pub instruction_bytes: [u8; 16],
    pub access_info: MemoryAccessInfo,
    pub gpa: u64,
    pub gva: u64,
}

impl From<WHV_MEMORY_ACCESS_CONTEXT> for MemoryAccessContext {
    fn from(value: WHV_MEMORY_ACCESS_CONTEXT) -> Self {
        Self {
            instruction_byte_count: value.InstructionByteCount,
            instruction_bytes: value.InstructionBytes,
            access_info: value.AccessInfo.into(),
            gpa: value.Gpa,
            gva: value.Gva,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IoPortAccessContext {
    pub instruction_byte_count: u8,
    // TODO: Type this better, maybe these should be [Instruction]'s?
    pub instruction_bytes: [u8; 16],
    pub access_info: IoPortAccessInfo,
    pub port_number: u16,
    pub rax: u64,
    pub rcx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub ds: SegmentRegister,
    pub es: SegmentRegister,
}

impl From<WHV_X64_IO_PORT_ACCESS_CONTEXT> for IoPortAccessContext {
    fn from(value: WHV_X64_IO_PORT_ACCESS_CONTEXT) -> Self {
        Self {
            instruction_byte_count: value.InstructionByteCount,
            instruction_bytes: value.InstructionBytes,
            access_info: value.AccessInfo.into(),
            port_number: value.PortNumber,
            rax: value.Rax,
            rcx: value.Rcx,
            rsi: value.Rsi,
            rdi: value.Rdi,
            ds: value.Ds.into(),
            es: value.Es.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MsrAccessContext {
    pub access_info: MsrAccessInfo,
    pub msr_number: u32,
    pub rax: u64,
    pub rdx: u64,
}

impl From<WHV_X64_MSR_ACCESS_CONTEXT> for MsrAccessContext {
    fn from(value: WHV_X64_MSR_ACCESS_CONTEXT) -> Self {
        Self {
            access_info: value.AccessInfo.into(),
            msr_number: value.MsrNumber,
            rax: value.Rax,
            rdx: value.Rdx,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuidAccessContext {
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rbx: u64,
    pub default_result_rax: u64,
    pub default_result_rcx: u64,
    pub default_result_rdx: u64,
    pub default_result_rbx: u64,
}

impl From<WHV_X64_CPUID_ACCESS_CONTEXT> for CpuidAccessContext {
    fn from(value: WHV_X64_CPUID_ACCESS_CONTEXT) -> Self {
        Self {
            rax: value.Rax,
            rcx: value.Rcx,
            rdx: value.Rdx,
            rbx: value.Rbx,
            default_result_rax: value.DefaultResultRax,
            default_result_rcx: value.DefaultResultRcx,
            default_result_rdx: value.DefaultResultRdx,
            default_result_rbx: value.DefaultResultRbx,
        }
    }
}

// TODO: Ahhh this has Vp prefix when variant doesnt...
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VpExceptionContext {
    pub instruction_byte_count: u8,
    // TODO: Type this better, maybe these should be [Instruction]'s?
    pub instruction_bytes: [u8; 16],
    pub exception_info: VpExceptionInfo,
    pub exception_type: u8,
    // TODO: We should type this.
    pub error_code: u32,
    pub exception_param: u64,
}

impl From<WHV_VP_EXCEPTION_CONTEXT> for VpExceptionContext {
    fn from(value: WHV_VP_EXCEPTION_CONTEXT) -> Self {
        Self {
            instruction_byte_count: value.InstructionByteCount,
            instruction_bytes: value.InstructionBytes,
            exception_info: value.ExceptionInfo.into(),
            exception_type: value.ExceptionType,
            error_code: value.ErrorCode,
            exception_param: value.ExceptionParameter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedFeatureCode {
    Intercept = 1,
    TaskSwitchTss = 2,
}

impl From<WHV_X64_UNSUPPORTED_FEATURE_CODE> for UnsupportedFeatureCode {
    fn from(value: WHV_X64_UNSUPPORTED_FEATURE_CODE) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            1 => Self::Intercept,
            2 => Self::TaskSwitchTss,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnsupportedFeatureContext {
    pub feature_code: UnsupportedFeatureCode,
    pub feature_param: u64,
}

impl From<WHV_X64_UNSUPPORTED_FEATURE_CONTEXT> for UnsupportedFeatureContext {
    fn from(value: WHV_X64_UNSUPPORTED_FEATURE_CONTEXT) -> Self {
        Self {
            feature_code: value.FeatureCode.into(),
            feature_param: value.FeatureParameter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VpCancelReason {
    User = 0,
}

impl From<WHV_RUN_VP_CANCEL_REASON> for VpCancelReason {
    fn from(value: WHV_RUN_VP_CANCEL_REASON) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0x0 => Self::User,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VpCanceledContext {
    pub cancel_reason: VpCancelReason,
}

impl From<WHV_RUN_VP_CANCELED_CONTEXT> for VpCanceledContext {
    fn from(value: WHV_RUN_VP_CANCELED_CONTEXT) -> Self {
        Self {
            cancel_reason: value.CancelReason.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApicEoiContext {
    pub interrupt_vec: u32,
}

impl From<WHV_X64_APIC_EOI_CONTEXT> for ApicEoiContext {
    fn from(value: WHV_X64_APIC_EOI_CONTEXT) -> Self {
        Self {
            interrupt_vec: value.InterruptVector.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RdtscContext {
    pub tsc_aux: u64,
    pub virtual_offset: u64,
    pub tsc: u64,
    pub reference_time: u64,
    pub rdtsc_info: RdtscInfo,
}

impl From<WHV_X64_RDTSC_CONTEXT> for RdtscContext {
    fn from(value: WHV_X64_RDTSC_CONTEXT) -> Self {
        Self {
            tsc_aux: value.TscAux,
            virtual_offset: value.VirtualOffset,
            tsc: value.Tsc,
            reference_time: value.ReferenceTime,
            rdtsc_info: value.RdtscInfo.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApicSmiContext {
    pub apic_icr: u64,
}

impl From<WHV_X64_APIC_SMI_CONTEXT> for ApicSmiContext {
    fn from(value: WHV_X64_APIC_SMI_CONTEXT) -> Self {
        Self {
            apic_icr: value.ApicIcr.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HypercallContext {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub r8: u64,
    pub rsi: u64,
    pub rdi: u64,
    // TODO: We should really have an adjustable stride for this. Like splitting 128 bits into 4 * 32 bits for processing.
    pub xmm_registers: [u128; WHV_HYPERCALL_CONTEXT_MAX_XMM_REGISTERS as usize],
}

impl From<WHV_HYPERCALL_CONTEXT> for HypercallContext {
    fn from(value: WHV_HYPERCALL_CONTEXT) -> Self {
        Self {
            rax: value.Rax,
            rbx: value.Rbx,
            rcx: value.Rcx,
            rdx: value.Rdx,
            r8: value.R8,
            rsi: value.Rsi,
            rdi: value.Rdi,
            // TODO: This needs to be removed...
            xmm_registers: unsafe { std::mem::transmute(value.XmmRegisters) },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptionDeliverableContext {
    pub deliverable_type: PendingInterruptionType,
}

impl From<WHV_X64_INTERRUPTION_DELIVERABLE_CONTEXT> for InterruptionDeliverableContext {
    fn from(value: WHV_X64_INTERRUPTION_DELIVERABLE_CONTEXT) -> Self {
        Self {
            deliverable_type: value.DeliverableType.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingInterruptionType {
    Interrupt = 0,
    Nmi = 2,
    Exception = 3,
}

impl From<WHV_X64_PENDING_INTERRUPTION_TYPE> for PendingInterruptionType {
    fn from(value: WHV_X64_PENDING_INTERRUPTION_TYPE) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0 => Self::Interrupt,
            2 => Self::Nmi,
            3 => Self::Exception,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X64ApicInitSipiContext {
    pub apic_icr: u64,
}

impl From<WHV_X64_APIC_INIT_SIPI_CONTEXT> for X64ApicInitSipiContext {
    fn from(value: WHV_X64_APIC_INIT_SIPI_CONTEXT) -> Self {
        Self {
            apic_icr: value.ApicIcr,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct X64ApicWriteContext {
    pub ty: ApicWriteType,
    pub write_value: u64,
}

impl From<WHV_X64_APIC_WRITE_CONTEXT> for X64ApicWriteContext {
    fn from(value: WHV_X64_APIC_WRITE_CONTEXT) -> Self {
        Self {
            ty: value.Type.into(),
            write_value: value.WriteValue,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApicWriteType {
    Ldr = 0xD0,
    Dfr = 0xE0,
    Svr = 0xF0,
    Lint0 = 0x350,
    Lint1 = 0x360,
}

impl From<WHV_X64_APIC_WRITE_TYPE> for ApicWriteType {
    fn from(value: WHV_X64_APIC_WRITE_TYPE) -> Self {
        // TODO: Can we enforce this differently?
        match value.0 {
            0xD0 => Self::Ldr,
            0xE0 => Self::Dfr,
            0xF0 => Self::Svr,
            0x350 => Self::Lint0,
            0x360 => Self::Lint1,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SynicSintDeliverableContext {
    pub deliverable_sints: u16,
}

impl From<WHV_SYNIC_SINT_DELIVERABLE_CONTEXT> for SynicSintDeliverableContext {
    fn from(value: WHV_SYNIC_SINT_DELIVERABLE_CONTEXT) -> Self {
        Self {
            deliverable_sints: value.DeliverableSints,
        }
    }
}

// TODO: Terrible placeholder name.
// TODO: Verify all extended contexts are listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunExitContextExt {
    MemoryAccess(MemoryAccessContext),
    IoPortAccess(IoPortAccessContext),
    MsrAccess(MsrAccessContext),
    CpuidAccess(CpuidAccessContext),
    // TODO: Ahhh this has Vp prefix...
    VpException(VpExceptionContext),
    InterruptWindow(InterruptionDeliverableContext),
    UnsupportedFeature(UnsupportedFeatureContext),
    // TODO: Rename to VpCancelReason?
    CancelReason(VpCanceledContext),
    ApicEoi(ApicEoiContext),
    ReadTsc(RdtscContext),
    ApicSmi(ApicSmiContext),
    Hypercall(HypercallContext),
    ApicInitSipi(X64ApicInitSipiContext),
    ApicWrite(X64ApicWriteContext),
    SynicSintDeliverable(SynicSintDeliverableContext),
}

impl RunExitContextExt {
    pub fn from_union(
        exit_reason: RunExitReason,
        context_ext: WHV_RUN_VP_EXIT_CONTEXT_0,
    ) -> Option<Self> {
        // SAFETY: The reason corresponds to the union variant.
        unsafe {
            match exit_reason {
                RunExitReason::None => None,
                RunExitReason::MemoryAccess => {
                    Some(Self::MemoryAccess(context_ext.MemoryAccess.into()))
                }
                RunExitReason::X64IoPortAccess => {
                    Some(Self::IoPortAccess(context_ext.IoPortAccess.into()))
                }
                RunExitReason::UnrecoverableException => None, // TODO: Validate this is None
                RunExitReason::InvalidVpRegisterValue => None, // TODO: Validate this is None
                RunExitReason::UnsupportedFeature => Some(Self::UnsupportedFeature(
                    context_ext.UnsupportedFeature.into(),
                )),
                RunExitReason::X64InterruptWindow => {
                    Some(Self::InterruptWindow(context_ext.InterruptWindow.into()))
                }
                RunExitReason::X64Halt => None,
                RunExitReason::X64ApicEoi => Some(Self::ApicEoi(context_ext.ApicEoi.into())),
                RunExitReason::SynicSintDeliverable => Some(Self::SynicSintDeliverable(
                    context_ext.SynicSintDeliverable.into(),
                )),
                RunExitReason::X64MsrAccess => Some(Self::MsrAccess(context_ext.MsrAccess.into())),
                RunExitReason::X64Cpuid => Some(Self::CpuidAccess(context_ext.CpuidAccess.into())),
                RunExitReason::Exception => Some(Self::VpException(context_ext.VpException.into())),
                RunExitReason::X64Rdtsc => Some(Self::ReadTsc(context_ext.ReadTsc.into())),
                RunExitReason::X64ApicSmiTrap => Some(Self::ApicSmi(context_ext.ApicSmi.into())),
                RunExitReason::Hypercall => Some(Self::Hypercall(context_ext.Hypercall.into())),
                RunExitReason::X64ApicInitSipiTrap => {
                    Some(Self::ApicInitSipi(context_ext.ApicInitSipi.into()))
                }
                RunExitReason::X64ApicWriteTrap => {
                    Some(Self::ApicWrite(context_ext.ApicWrite.into()))
                }
                RunExitReason::Canceled => {
                    Some(Self::CancelReason(context_ext.CancelReason.into()))
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct VirtualProcessor {
    partition_handle: Arc<PartitionHandle>,
    index: u32,
}

impl VirtualProcessor {
    pub fn new(partition_handle: Arc<PartitionHandle>, index: u32) -> Self {
        // TODO: Sanity checks here. (Check index to make sure its at or below the processor count in partition.)
        Self {
            partition_handle,
            index,
        }
    }

    // TODO: Return exit context.
    pub fn run(&mut self) -> Result<RunExitContext> {
        let mut raw_exit_context: WHV_RUN_VP_EXIT_CONTEXT = Default::default();
        unsafe {
            WHvRunVirtualProcessor(
                self.partition_handle.0,
                self.index,
                std::mem::transmute(&mut raw_exit_context),
                std::mem::size_of::<WHV_RUN_VP_EXIT_CONTEXT>().try_into()?,
            )?;
        }
        Ok(raw_exit_context.into())
    }

    pub fn set_register(&mut self, register: Register, value: RegisterVal) -> Result<()> {
        self.set_registers(&[(register, value)])
    }

    pub fn set_registers(&mut self, register_vals: &[(Register, RegisterVal)]) -> Result<()> {
        let (raw_registers, raw_values): (Vec<_>, Vec<_>) = register_vals
            .into_iter()
            .map(|(r, v)| (WHV_REGISTER_NAME::from(*r), WHV_REGISTER_VALUE::from(*v)))
            .unzip();

        unsafe {
            WHvSetVirtualProcessorRegisters(
                self.partition_handle.0,
                self.index,
                raw_registers.as_ptr(),
                raw_registers.len().try_into()?,
                raw_values.as_ptr(),
            )?;
        }

        Ok(())
    }

    pub fn get_register(&mut self, register: Register) -> Result<RegisterVal> {
        Ok(self.get_registers(&[register])?[0].1)
    }

    pub fn get_registers<'a>(
        &mut self,
        registers: &'a [Register],
    ) -> Result<Vec<(&'a Register, RegisterVal)>> {
        let raw_registers: Vec<_> = registers
            .into_iter()
            .map(|&r| WHV_REGISTER_NAME::from(r))
            .collect();
        let mut raw_values: Vec<WHV_REGISTER_VALUE> = vec![Default::default(); registers.len()];

        unsafe {
            WHvGetVirtualProcessorRegisters(
                self.partition_handle.0,
                self.index,
                raw_registers.as_ptr(),
                raw_registers.len().try_into()?,
                raw_values.as_mut_ptr(),
            )?;
        }

        let reg_tup = registers
            .iter()
            .zip(raw_values.iter())
            .map(|(r, v)| (r, RegisterVal::from_union(r.ty(), *v)))
            .collect::<Vec<(_, _)>>();

        Ok(reg_tup)
    }
}

impl Drop for VirtualProcessor {
    fn drop(&mut self) {
        let _ = unsafe { WHvDeleteVirtualProcessor(self.partition_handle.0, self.index) };
    }
}

// TODO: Support real mode registers and other duplicate registers (i.e. eax is rax)?
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    // X64 General purpose registers
    Rax = 0x00000000,
    Rcx = 0x00000001,
    Rdx = 0x00000002,
    Rbx = 0x00000003,
    Rsp = 0x00000004,
    Rbp = 0x00000005,
    Rsi = 0x00000006,
    Rdi = 0x00000007,
    R8 = 0x00000008,
    R9 = 0x00000009,
    R10 = 0x0000000A,
    R11 = 0x0000000B,
    R12 = 0x0000000C,
    R13 = 0x0000000D,
    R14 = 0x0000000E,
    R15 = 0x0000000F,
    Rip = 0x00000010,
    Rflags = 0x00000011,
    // X64 Segment registers
    Es = 0x00000012,
    Cs = 0x00000013,
    Ss = 0x00000014,
    Ds = 0x00000015,
    Fs = 0x00000016,
    Gs = 0x00000017,
    Ldtr = 0x00000018,
    Tr = 0x00000019,
    // X64 Table registers
    Idtr = 0x0000001A,
    Gdtr = 0x0000001B,
    // X64 Control Registers
    Cr0 = 0x0000001C,
    Cr2 = 0x0000001D,
    Cr3 = 0x0000001E,
    Cr4 = 0x0000001F,
    Cr8 = 0x00000020,
    // X64 Debug Registers
    Dr0 = 0x00000021,
    Dr1 = 0x00000022,
    Dr2 = 0x00000023,
    Dr3 = 0x00000024,
    Dr6 = 0x00000025,
    Dr7 = 0x00000026,
    // X64 Extended Control Registers
    XCr0 = 0x00000027,
    VirtualCr0 = 0x00000028,
    VirtualCr3 = 0x00000029,
    VirtualCr4 = 0x0000002A,
    VirtualCr8 = 0x0000002B,
    // X64 Floating Point and Vector Registers
    Xmm0 = 0x00001000,
    Xmm1 = 0x00001001,
    Xmm2 = 0x00001002,
    Xmm3 = 0x00001003,
    Xmm4 = 0x00001004,
    Xmm5 = 0x00001005,
    Xmm6 = 0x00001006,
    Xmm7 = 0x00001007,
    Xmm8 = 0x00001008,
    Xmm9 = 0x00001009,
    Xmm10 = 0x0000100A,
    Xmm11 = 0x0000100B,
    Xmm12 = 0x0000100C,
    Xmm13 = 0x0000100D,
    Xmm14 = 0x0000100E,
    Xmm15 = 0x0000100F,
    FpMmx0 = 0x00001010,
    FpMmx1 = 0x00001011,
    FpMmx2 = 0x00001012,
    FpMmx3 = 0x00001013,
    FpMmx4 = 0x00001014,
    FpMmx5 = 0x00001015,
    FpMmx6 = 0x00001016,
    FpMmx7 = 0x00001017,
    FpControlStatus = 0x00001018,
    XmmControlStatus = 0x00001019,
    // X64 MSRs
    Tsc = 0x00002000,
    Efer = 0x00002001,
    KernelGsBase = 0x00002002,
    ApicBase = 0x00002003,
    Pat = 0x00002004,
    SysenterCs = 0x00002005,
    SysenterEip = 0x00002006,
    SysenterEsp = 0x00002007,
    Star = 0x00002008,
    Lstar = 0x00002009,
    Cstar = 0x0000200A,
    Sfmask = 0x0000200B,
    InitialApicId = 0x0000200C,
    MsrMtrrCap = 0x0000200D,
    MsrMtrrDefType = 0x0000200E,
    MsrMtrrPhysBase0 = 0x00002010,
    MsrMtrrPhysBase1 = 0x00002011,
    MsrMtrrPhysBase2 = 0x00002012,
    MsrMtrrPhysBase3 = 0x00002013,
    MsrMtrrPhysBase4 = 0x00002014,
    MsrMtrrPhysBase5 = 0x00002015,
    MsrMtrrPhysBase6 = 0x00002016,
    MsrMtrrPhysBase7 = 0x00002017,
    MsrMtrrPhysBase8 = 0x00002018,
    MsrMtrrPhysBase9 = 0x00002019,
    MsrMtrrPhysBaseA = 0x0000201A,
    MsrMtrrPhysBaseB = 0x0000201B,
    MsrMtrrPhysBaseC = 0x0000201C,
    MsrMtrrPhysBaseD = 0x0000201D,
    MsrMtrrPhysBaseE = 0x0000201E,
    MsrMtrrPhysBaseF = 0x0000201F,
    MsrMtrrPhysMask0 = 0x00002040,
    MsrMtrrPhysMask1 = 0x00002041,
    MsrMtrrPhysMask2 = 0x00002042,
    MsrMtrrPhysMask3 = 0x00002043,
    MsrMtrrPhysMask4 = 0x00002044,
    MsrMtrrPhysMask5 = 0x00002045,
    MsrMtrrPhysMask6 = 0x00002046,
    MsrMtrrPhysMask7 = 0x00002047,
    MsrMtrrPhysMask8 = 0x00002048,
    MsrMtrrPhysMask9 = 0x00002049,
    MsrMtrrPhysMaskA = 0x0000204A,
    MsrMtrrPhysMaskB = 0x0000204B,
    MsrMtrrPhysMaskC = 0x0000204C,
    MsrMtrrPhysMaskD = 0x0000204D,
    MsrMtrrPhysMaskE = 0x0000204E,
    MsrMtrrPhysMaskF = 0x0000204F,
    MsrMtrrFix64k00000 = 0x00002070,
    MsrMtrrFix16k80000 = 0x00002071,
    MsrMtrrFix16kA0000 = 0x00002072,
    MsrMtrrFix4kC0000 = 0x00002073,
    MsrMtrrFix4kC8000 = 0x00002074,
    MsrMtrrFix4kD0000 = 0x00002075,
    MsrMtrrFix4kD8000 = 0x00002076,
    MsrMtrrFix4kE0000 = 0x00002077,
    MsrMtrrFix4kE8000 = 0x00002078,
    MsrMtrrFix4kF0000 = 0x00002079,
    MsrMtrrFix4kF8000 = 0x0000207A,
    TscAux = 0x0000207B,
    Bndcfgs = 0x0000207C,
    MCount = 0x0000207E,
    ACount = 0x0000207F,
    SpecCtrl = 0x00002084,
    PredCmd = 0x00002085,
    TscVirtualOffset = 0x00002087,
    TsxCtrl = 0x00002088,
    Xss = 0x0000208B,
    UCet = 0x0000208C,
    SCet = 0x0000208D,
    Ssp = 0x0000208E,
    Pl0Ssp = 0x0000208F,
    Pl1Ssp = 0x00002090,
    Pl2Ssp = 0x00002091,
    Pl3Ssp = 0x00002092,
    InterruptSspTableAddr = 0x00002093,
    TscDeadline = 0x00002095,
    TscAdjust = 0x00002096,
    UmwaitControl = 0x00002098,
    Xfd = 0x00002099,
    XfdErr = 0x0000209A,
    // APIC state (also accessible via WHv(Get/Set)VirtualProcessorInterruptControllerState)
    ApicId = 0x00003002,
    ApicVersion = 0x00003003,
    ApicTpr = 0x00003008,
    ApicPpr = 0x0000300A,
    ApicEoi = 0x0000300B,
    ApicLdr = 0x0000300D,
    ApicSpurious = 0x0000300F,
    ApicIsr0 = 0x00003010,
    ApicIsr1 = 0x00003011,
    ApicIsr2 = 0x00003012,
    ApicIsr3 = 0x00003013,
    ApicIsr4 = 0x00003014,
    ApicIsr5 = 0x00003015,
    ApicIsr6 = 0x00003016,
    ApicIsr7 = 0x00003017,
    ApicTmr0 = 0x00003018,
    ApicTmr1 = 0x00003019,
    ApicTmr2 = 0x0000301A,
    ApicTmr3 = 0x0000301B,
    ApicTmr4 = 0x0000301C,
    ApicTmr5 = 0x0000301D,
    ApicTmr6 = 0x0000301E,
    ApicTmr7 = 0x0000301F,
    ApicIrr0 = 0x00003020,
    ApicIrr1 = 0x00003021,
    ApicIrr2 = 0x00003022,
    ApicIrr3 = 0x00003023,
    ApicIrr4 = 0x00003024,
    ApicIrr5 = 0x00003025,
    ApicIrr6 = 0x00003026,
    ApicIrr7 = 0x00003027,
    ApicEse = 0x00003028,
    ApicIcr = 0x00003030,
    ApicLvtTimer = 0x00003032,
    ApicLvtThermal = 0x00003033,
    ApicLvtPerfmon = 0x00003034,
    ApicLvtLint0 = 0x00003035,
    ApicLvtLint1 = 0x00003036,
    ApicLvtError = 0x00003037,
    ApicInitCount = 0x00003038,
    ApicCurrentCount = 0x00003039,
    ApicDivide = 0x0000303E,
    ApicSelfIpi = 0x0000303F,
    Sint0 = 0x00004000,
    Sint1 = 0x00004001,
    Sint2 = 0x00004002,
    Sint3 = 0x00004003,
    Sint4 = 0x00004004,
    Sint5 = 0x00004005,
    Sint6 = 0x00004006,
    Sint7 = 0x00004007,
    Sint8 = 0x00004008,
    Sint9 = 0x00004009,
    Sint10 = 0x0000400A,
    Sint11 = 0x0000400B,
    Sint12 = 0x0000400C,
    Sint13 = 0x0000400D,
    Sint14 = 0x0000400E,
    Sint15 = 0x0000400F,
    Scontrol = 0x00004010,
    Sversion = 0x00004011,
    Siefp = 0x00004012,
    Simp = 0x00004013,
    Eom = 0x00004014,
    VpRuntime = 0x00005000,
    Hypercall = 0x00005001,
    GuestOsId = 0x00005002,
    VpAssistPage = 0x00005013,
    ReferenceTsc = 0x00005017,
    ReferenceTscSequence = 0x0000501A,
    // Interrupt / Event Registers
    PendingInterruption = 0x80000000u32 as i32,
    InterruptState = 0x80000001u32 as i32,
    PendingEvent = 0x80000002u32 as i32,
    DeliverabilityNotifications = 0x80000004u32 as i32,
    InternalActivityState = 0x80000005u32 as i32,
    PendingDebugException = 0x80000006u32 as i32,
}

impl Register {
    // TODO: This is completely wrong...
    pub const fn ty(&self) -> RegisterType {
        match self {
            Register::Rax
            | Register::Rcx
            | Register::Rdx
            | Register::Rbx
            | Register::Rsp
            | Register::Rbp
            | Register::Rsi
            | Register::Rdi
            | Register::R8
            | Register::R9
            | Register::R10
            | Register::R11
            | Register::R12
            | Register::R13
            | Register::R14
            | Register::R15
            | Register::Rip
            | Register::Rflags => RegisterType::Reg64,
            Register::Es
            | Register::Cs
            | Register::Ss
            | Register::Ds
            | Register::Fs
            | Register::Gs
            | Register::Ldtr
            | Register::Tr => RegisterType::Segment,
            Register::Idtr | Register::Gdtr => RegisterType::Table,
            Register::Cr0 | Register::Cr2 | Register::Cr3 | Register::Cr4 | Register::Cr8 => {
                RegisterType::Reg64
            }
            Register::Dr0
            | Register::Dr1
            | Register::Dr2
            | Register::Dr3
            | Register::Dr6
            | Register::Dr7 => RegisterType::Reg64,
            Register::XCr0
            | Register::VirtualCr0
            | Register::VirtualCr3
            | Register::VirtualCr4
            | Register::VirtualCr8 => RegisterType::Reg64,
            Register::Xmm0
            | Register::Xmm1
            | Register::Xmm2
            | Register::Xmm3
            | Register::Xmm4
            | Register::Xmm5
            | Register::Xmm6
            | Register::Xmm7
            | Register::Xmm8
            | Register::Xmm9
            | Register::Xmm10
            | Register::Xmm11
            | Register::Xmm12
            | Register::Xmm13
            | Register::Xmm14
            | Register::Xmm15 => RegisterType::Reg128,
            Register::FpMmx0
            | Register::FpMmx1
            | Register::FpMmx2
            | Register::FpMmx3
            | Register::FpMmx4
            | Register::FpMmx5
            | Register::FpMmx6
            | Register::FpMmx7 => RegisterType::Fp, // TODO: Might not be right, this is also aliased for mmx.
            Register::FpControlStatus => RegisterType::FpControlStatus,
            Register::XmmControlStatus => RegisterType::XmmControlStatus,
            Register::Tsc
            | Register::Efer
            | Register::KernelGsBase
            | Register::ApicBase
            | Register::Pat
            | Register::SysenterCs
            | Register::SysenterEip
            | Register::SysenterEsp
            | Register::Star
            | Register::Lstar
            | Register::Cstar
            | Register::Sfmask
            | Register::InitialApicId
            | Register::MsrMtrrCap
            | Register::MsrMtrrDefType
            | Register::MsrMtrrPhysBase0
            | Register::MsrMtrrPhysBase1
            | Register::MsrMtrrPhysBase2
            | Register::MsrMtrrPhysBase3
            | Register::MsrMtrrPhysBase4
            | Register::MsrMtrrPhysBase5
            | Register::MsrMtrrPhysBase6
            | Register::MsrMtrrPhysBase7
            | Register::MsrMtrrPhysBase8
            | Register::MsrMtrrPhysBase9
            | Register::MsrMtrrPhysBaseA
            | Register::MsrMtrrPhysBaseB
            | Register::MsrMtrrPhysBaseC
            | Register::MsrMtrrPhysBaseD
            | Register::MsrMtrrPhysBaseE
            | Register::MsrMtrrPhysBaseF
            | Register::MsrMtrrPhysMask0
            | Register::MsrMtrrPhysMask1
            | Register::MsrMtrrPhysMask2
            | Register::MsrMtrrPhysMask3
            | Register::MsrMtrrPhysMask4
            | Register::MsrMtrrPhysMask5
            | Register::MsrMtrrPhysMask6
            | Register::MsrMtrrPhysMask7
            | Register::MsrMtrrPhysMask8
            | Register::MsrMtrrPhysMask9
            | Register::MsrMtrrPhysMaskA
            | Register::MsrMtrrPhysMaskB
            | Register::MsrMtrrPhysMaskC
            | Register::MsrMtrrPhysMaskD
            | Register::MsrMtrrPhysMaskE
            | Register::MsrMtrrPhysMaskF
            | Register::MsrMtrrFix64k00000
            | Register::MsrMtrrFix16k80000
            | Register::MsrMtrrFix16kA0000
            | Register::MsrMtrrFix4kC0000
            | Register::MsrMtrrFix4kC8000
            | Register::MsrMtrrFix4kD0000
            | Register::MsrMtrrFix4kD8000
            | Register::MsrMtrrFix4kE0000
            | Register::MsrMtrrFix4kE8000
            | Register::MsrMtrrFix4kF0000
            | Register::MsrMtrrFix4kF8000
            | Register::TscAux
            | Register::Bndcfgs
            | Register::MCount
            | Register::ACount
            | Register::SpecCtrl
            | Register::PredCmd
            | Register::TscVirtualOffset
            | Register::TsxCtrl
            | Register::Xss
            | Register::UCet
            | Register::SCet
            | Register::Ssp
            | Register::Pl0Ssp
            | Register::Pl1Ssp
            | Register::Pl2Ssp
            | Register::Pl3Ssp
            | Register::InterruptSspTableAddr
            | Register::TscDeadline
            | Register::TscAdjust
            | Register::UmwaitControl
            | Register::Xfd
            | Register::XfdErr => RegisterType::Reg64,
            Register::ApicId
            | Register::ApicVersion
            | Register::ApicTpr
            | Register::ApicPpr
            | Register::ApicEoi
            | Register::ApicLdr
            | Register::ApicSpurious
            | Register::ApicIsr0
            | Register::ApicIsr1
            | Register::ApicIsr2
            | Register::ApicIsr3
            | Register::ApicIsr4
            | Register::ApicIsr5
            | Register::ApicIsr6
            | Register::ApicIsr7
            | Register::ApicTmr0
            | Register::ApicTmr1
            | Register::ApicTmr2
            | Register::ApicTmr3
            | Register::ApicTmr4
            | Register::ApicTmr5
            | Register::ApicTmr6
            | Register::ApicTmr7
            | Register::ApicIrr0
            | Register::ApicIrr1
            | Register::ApicIrr2
            | Register::ApicIrr3
            | Register::ApicIrr4
            | Register::ApicIrr5
            | Register::ApicIrr6
            | Register::ApicIrr7
            | Register::ApicEse
            | Register::ApicIcr
            | Register::ApicLvtTimer
            | Register::ApicLvtThermal
            | Register::ApicLvtPerfmon
            | Register::ApicLvtLint0
            | Register::ApicLvtLint1
            | Register::ApicLvtError
            | Register::ApicInitCount
            | Register::ApicCurrentCount
            | Register::ApicDivide
            | Register::ApicSelfIpi => RegisterType::Reg64,
            Register::Sint0
            | Register::Sint1
            | Register::Sint2
            | Register::Sint3
            | Register::Sint4
            | Register::Sint5
            | Register::Sint6
            | Register::Sint7
            | Register::Sint8
            | Register::Sint9
            | Register::Sint10
            | Register::Sint11
            | Register::Sint12
            | Register::Sint13
            | Register::Sint14
            | Register::Sint15
            | Register::Scontrol
            | Register::Sversion
            | Register::Siefp
            | Register::Simp
            | Register::Eom
            | Register::VpRuntime
            | Register::Hypercall
            | Register::GuestOsId
            | Register::VpAssistPage
            | Register::ReferenceTsc
            | Register::ReferenceTscSequence
            | Register::PendingInterruption
            | Register::InterruptState
            | Register::PendingEvent
            | Register::DeliverabilityNotifications
            | Register::InternalActivityState
            | Register::PendingDebugException => todo!(),
        }
    }
}

impl From<Register> for WHV_REGISTER_NAME {
    fn from(value: Register) -> Self {
        Self(value as _)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterType {
    Reg128,
    Reg64,
    Reg32,
    Reg16,
    Reg8,
    Fp,
    FpControlStatus,
    XmmControlStatus,
    Segment,
    Table,
    InterruptState,
    PendingInterruption,
    DeliverabilityNotifications,
    ExceptionEvent,
    ExtIntEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterVal {
    Reg128(u128),
    Reg64(u64),
    Reg32(u32),
    Reg16(u16),
    Reg8(u8),
    Fp(FpRegister),
    FpControlStatus(X64FpControlStatusRegister),
    XmmControlStatus(X64XmmControlStatusRegister),
    Segment(SegmentRegister),
    Table(TableRegister),
    InterruptState(InterruptStateRegister),
    PendingInterruption(PendingInterruptionRegister),
    DeliverabilityNotifications(DeliverabilityNotificationsRegister),
    ExceptionEvent(PendingExceptionEvent),
    ExtIntEvent(PendingExtIntEvent),
}

impl RegisterVal {
    pub fn from_union(ty: RegisterType, raw_val: WHV_REGISTER_VALUE) -> Self {
        // SAFETY: The code corresponds to the union variant.
        unsafe {
            match ty {
                // TODO: Get rid of this transmute.
                RegisterType::Reg128 => Self::Reg128(std::mem::transmute(raw_val.Reg128)),
                RegisterType::Reg64 => Self::Reg64(raw_val.Reg64),
                RegisterType::Reg32 => Self::Reg32(raw_val.Reg32),
                RegisterType::Reg16 => Self::Reg16(raw_val.Reg16),
                RegisterType::Reg8 => Self::Reg8(raw_val.Reg8),
                RegisterType::Fp => Self::Fp(raw_val.Fp.into()),
                RegisterType::FpControlStatus => {
                    Self::FpControlStatus(raw_val.FpControlStatus.into())
                }
                RegisterType::XmmControlStatus => {
                    Self::XmmControlStatus(raw_val.XmmControlStatus.into())
                }
                RegisterType::Segment => Self::Segment(raw_val.Segment.into()),
                RegisterType::Table => Self::Table(raw_val.Table.into()),
                RegisterType::InterruptState => Self::InterruptState(raw_val.InterruptState.into()),
                RegisterType::PendingInterruption => {
                    Self::PendingInterruption(raw_val.PendingInterruption.into())
                }
                RegisterType::DeliverabilityNotifications => {
                    Self::DeliverabilityNotifications(raw_val.DeliverabilityNotifications.into())
                }
                RegisterType::ExceptionEvent => Self::ExceptionEvent(raw_val.ExceptionEvent.into()),
                RegisterType::ExtIntEvent => Self::ExtIntEvent(raw_val.ExtIntEvent.into()),
            }
        }
    }
}

impl From<RegisterVal> for WHV_REGISTER_VALUE {
    fn from(value: RegisterVal) -> Self {
        match value {
            RegisterVal::Reg128(v) => WHV_REGISTER_VALUE {
                // TODO: This needs to be redone.
                Reg128: unsafe { std::mem::transmute(v) },
            },
            RegisterVal::Reg64(v) => WHV_REGISTER_VALUE { Reg64: v },
            RegisterVal::Reg32(v) => WHV_REGISTER_VALUE { Reg32: v },
            RegisterVal::Reg16(v) => WHV_REGISTER_VALUE { Reg16: v },
            RegisterVal::Reg8(v) => WHV_REGISTER_VALUE { Reg8: v },
            RegisterVal::Fp(v) => WHV_REGISTER_VALUE { Fp: v.into() },
            RegisterVal::FpControlStatus(v) => WHV_REGISTER_VALUE {
                FpControlStatus: v.into(),
            },
            RegisterVal::XmmControlStatus(v) => WHV_REGISTER_VALUE {
                XmmControlStatus: v.into(),
            },
            RegisterVal::Segment(v) => WHV_REGISTER_VALUE { Segment: v.into() },
            RegisterVal::Table(v) => WHV_REGISTER_VALUE { Table: v.into() },
            RegisterVal::InterruptState(v) => WHV_REGISTER_VALUE {
                InterruptState: v.into(),
            },
            RegisterVal::PendingInterruption(v) => WHV_REGISTER_VALUE {
                PendingInterruption: v.into(),
            },
            RegisterVal::DeliverabilityNotifications(v) => WHV_REGISTER_VALUE {
                DeliverabilityNotifications: v.into(),
            },
            RegisterVal::ExceptionEvent(v) => WHV_REGISTER_VALUE {
                ExceptionEvent: v.into(),
            },
            RegisterVal::ExtIntEvent(v) => WHV_REGISTER_VALUE {
                ExtIntEvent: v.into(),
            },
        }
    }
}
