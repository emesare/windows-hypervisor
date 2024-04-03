use std::fmt::Debug;

use c2rust_bitfields::BitfieldStruct;
use windows::Win32::System::Hypervisor::{
    WHV_X64_DELIVERABILITY_NOTIFICATIONS_REGISTER, WHV_X64_FP_REGISTER, WHV_X64_FP_REGISTER_0,
    WHV_X64_PENDING_EXCEPTION_EVENT, WHV_X64_PENDING_EXCEPTION_EVENT_0,
    WHV_X64_PENDING_EXT_INT_EVENT, WHV_X64_PENDING_EXT_INT_EVENT_0,
    WHV_X64_PENDING_INTERRUPTION_REGISTER,
};

// TODO: Add helpers to these, i.e. f128 to FpRegister.
// TODO: Unit tests for these.

#[repr(C, align(1))]
#[derive(BitfieldStruct, Clone, Copy, PartialEq, Eq)]
pub struct FpRegister {
    mantissa: u64,
    #[bitfield(name = "biased_exponent", ty = "u16", bits = "0..=14")]
    #[bitfield(name = "sign", ty = "bool", bits = "15..=15")]
    bitfield: [u8; 8],
}

impl Debug for FpRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FpRegister")
            .field("mantissa", &self.mantissa)
            .field("biased_exponent", &self.biased_exponent())
            .field("sign", &self.sign())
            .finish()
    }
}

impl From<WHV_X64_FP_REGISTER> for FpRegister {
    fn from(value: WHV_X64_FP_REGISTER) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        unsafe {
            Self {
                mantissa: value.Anonymous.Mantissa,
                bitfield: value.Anonymous._bitfield.to_ne_bytes(),
            }
        }
    }
}

impl From<FpRegister> for WHV_X64_FP_REGISTER {
    fn from(value: FpRegister) -> Self {
        Self {
            Anonymous: WHV_X64_FP_REGISTER_0 {
                Mantissa: value.mantissa,
                _bitfield: u64::from_ne_bytes(value.bitfield),
            },
        }
    }
}

#[repr(C, align(1))]
#[derive(BitfieldStruct, Clone, Copy, PartialEq, Eq)]
pub struct PendingInterruptionRegister {
    #[bitfield(name = "interruption_pending", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "interruption_type", ty = "u8", bits = "1..=3")] // TODO: Interruption type... type?
    #[bitfield(name = "deliver_error_code", ty = "bool", bits = "4..=4")]
    #[bitfield(name = "instruction_len", ty = "u8", bits = "5..=8")]
    #[bitfield(name = "nested_event", ty = "bool", bits = "9..=9")]
    #[bitfield(name = "interruption_vector", ty = "u16", bits = "15..=31")]
    #[bitfield(name = "error_code", ty = "u32", bits = "32..=64")]
    bitfield: [u8; 8],
}

impl Debug for PendingInterruptionRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingInterruptionRegister")
            .field("interruption_pending", &self.interruption_pending())
            .field("interruption_type", &self.interruption_type())
            .field("deliver_error_code", &self.deliver_error_code())
            .field("instruction_len", &self.instruction_len())
            .field("nested_event", &self.nested_event())
            .field("interruption_vector", &self.interruption_vector())
            .field("error_code", &self.error_code())
            .finish()
    }
}

impl From<WHV_X64_PENDING_INTERRUPTION_REGISTER> for PendingInterruptionRegister {
    fn from(value: WHV_X64_PENDING_INTERRUPTION_REGISTER) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        Self {
            bitfield: unsafe { value.AsUINT64 }.to_ne_bytes(),
        }
    }
}

impl From<PendingInterruptionRegister> for WHV_X64_PENDING_INTERRUPTION_REGISTER {
    fn from(value: PendingInterruptionRegister) -> Self {
        Self {
            AsUINT64: u64::from_ne_bytes(value.bitfield),
        }
    }
}

#[repr(C, align(1))]
#[derive(BitfieldStruct, Clone, Copy, PartialEq, Eq)]
pub struct DeliverabilityNotificationsRegister {
    #[bitfield(name = "nmi_notification", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "interrupt_notification", ty = "bool", bits = "1..=1")]
    #[bitfield(name = "interruption_priority", ty = "u8", bits = "2..=5")]
    #[bitfield(name = "sint", ty = "u16", bits = "48..=64")]
    bitfield: [u8; 8],
}

impl Debug for DeliverabilityNotificationsRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeliverabilityNotificationsRegister")
            .field("nmi_notification", &self.nmi_notification())
            .field("interrupt_priority", &self.interrupt_notification())
            .field("interruption_priority", &self.interruption_priority())
            .field("sint", &self.sint())
            .finish()
    }
}

impl From<WHV_X64_DELIVERABILITY_NOTIFICATIONS_REGISTER> for DeliverabilityNotificationsRegister {
    fn from(value: WHV_X64_DELIVERABILITY_NOTIFICATIONS_REGISTER) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        Self {
            bitfield: unsafe { value.AsUINT64 }.to_ne_bytes(),
        }
    }
}

impl From<DeliverabilityNotificationsRegister> for WHV_X64_DELIVERABILITY_NOTIFICATIONS_REGISTER {
    fn from(value: DeliverabilityNotificationsRegister) -> Self {
        Self {
            AsUINT64: u64::from_ne_bytes(value.bitfield),
        }
    }
}

#[repr(C, align(1))]
#[derive(BitfieldStruct, Clone, Copy, PartialEq, Eq)]
pub struct PendingExceptionEvent {
    #[bitfield(name = "event_pending", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "event_type", ty = "bool", bits = "1..=3")]
    #[bitfield(name = "deliver_error_code", ty = "u32", bits = "8..=8")]
    #[bitfield(name = "vector", ty = "u16", bits = "16..=31")]
    bitfield: [u8; 4],
    error_code: u32,
    exception_param: u64,
}

impl Debug for PendingExceptionEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingExceptionEvent")
            .field("event_pending", &self.event_pending())
            .field("event_type", &self.event_type())
            .field("deliver_error_code", &self.deliver_error_code())
            .field("vector", &self.vector())
            .field("error_code", &self.error_code)
            .field("exception_param", &self.exception_param)
            .finish()
    }
}

impl From<WHV_X64_PENDING_EXCEPTION_EVENT> for PendingExceptionEvent {
    fn from(value: WHV_X64_PENDING_EXCEPTION_EVENT) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        unsafe {
            Self {
                bitfield: value.Anonymous._bitfield.to_ne_bytes(),
                error_code: value.Anonymous.ErrorCode,
                exception_param: value.Anonymous.ExceptionParameter,
            }
        }
    }
}

impl From<PendingExceptionEvent> for WHV_X64_PENDING_EXCEPTION_EVENT {
    fn from(value: PendingExceptionEvent) -> Self {
        Self {
            Anonymous: WHV_X64_PENDING_EXCEPTION_EVENT_0 {
                _bitfield: u32::from_ne_bytes(value.bitfield),
                ErrorCode: value.error_code,
                ExceptionParameter: value.exception_param,
            },
        }
    }
}

#[repr(C, align(1))]
#[derive(BitfieldStruct, Clone, Copy, PartialEq, Eq)]
pub struct PendingExtIntEvent {
    #[bitfield(name = "event_pending", ty = "bool", bits = "0..=0")]
    #[bitfield(name = "event_type", ty = "bool", bits = "1..=3")]
    #[bitfield(name = "vector", ty = "u32", bits = "8..=15")]
    bitfield: [u8; 8],
}

impl Debug for PendingExtIntEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PendingExtIntEvent")
            .field("event_pending", &self.event_pending())
            .field("event_type", &self.event_type())
            .field("vector", &self.vector())
            .finish()
    }
}

impl From<WHV_X64_PENDING_EXT_INT_EVENT> for PendingExtIntEvent {
    fn from(value: WHV_X64_PENDING_EXT_INT_EVENT) -> Self {
        // SAFETY: Reinterpreting the bits in-place.
        unsafe {
            Self {
                bitfield: value.Anonymous._bitfield.to_ne_bytes(),
            }
        }
    }
}

impl From<PendingExtIntEvent> for WHV_X64_PENDING_EXT_INT_EVENT {
    fn from(value: PendingExtIntEvent) -> Self {
        Self {
            Anonymous: WHV_X64_PENDING_EXT_INT_EVENT_0 {
                _bitfield: u64::from_ne_bytes(value.bitfield),
                Reserved2: 0,
            },
        }
    }
}
