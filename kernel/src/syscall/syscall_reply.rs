use crate::kernel::boot::{current_lookup_fault, current_syscall_error};
use sel4_common::arch::ArchReg;
use sel4_common::message_info::seL4_MessageInfo_func;
use sel4_common::sel4_config::*;
use sel4_common::shared_types_bf_gen::seL4_MessageInfo;
use sel4_task::tcb_t;

#[inline]
pub fn reply_error_from_kernel(thread: &mut tcb_t) {
    thread.tcbArch.set_register(ArchReg::Badge, 0);
    unsafe {
        let len = set_mrs_for_syscall_error(thread);
        thread.tcbArch.set_register(
            ArchReg::MsgInfo,
            seL4_MessageInfo::new(current_syscall_error._type as u64, 0, 0, len as u64).to_word(),
        );
    }
}

#[inline]
pub fn reply_success_from_kernel(thread: &mut tcb_t) {
    thread.tcbArch.set_register(ArchReg::Badge, 0);
    thread.tcbArch.set_register(
        ArchReg::MsgInfo,
        seL4_MessageInfo::new(0, 0, 0, 0).to_word(),
    );
}

// TODO: Remove this attribute to improve security.
#[allow(static_mut_ref)]
pub unsafe fn set_mrs_for_syscall_error(thread: &mut tcb_t) -> usize {
    match current_syscall_error._type {
        seL4_InvalidArgument => thread.set_mr(0, current_syscall_error.invalidArgumentNumber),
        seL4_InvalidCapability => thread.set_mr(0, current_syscall_error.invalidCapNumber),
        seL4_RangeError => {
            thread.set_mr(0, current_syscall_error.rangeErrorMin);
            thread.set_mr(1, current_syscall_error.rangeErrorMax)
        }
        seL4_FailedLookup => {
            let flag = current_syscall_error.failedLookupWasSource == 1;
            thread.set_mr(0, flag as usize);
            return thread.set_lookup_fault_mrs(1, &current_lookup_fault);
        }
        seL4_IllegalOperation
        | seL4_AlignmentError
        | seL4_TruncatedMessage
        | seL4_DeleteFirst
        | seL4_RevokeFirst => 0,
        seL4_NotEnoughMemory => thread.set_mr(0, current_syscall_error.memoryLeft),
        _ => {
            panic!("invalid syscall error")
        }
    }
}
