use crate::cpu::TrapFrame;
use crate::plic::complete;
use crate::{plic, uart};
use crate::{print, println};

#[no_mangle]
extern "C" fn m_trap(
    epc: usize,
    tval: usize,
    cause: usize,
    hart: usize,
    _status: usize,
    _frame: &mut TrapFrame,
) -> usize {
    // We're going to handle all traps in machine mode. RISC-V lets
    // us delegate to supervisor mode, but switching out SATP (virtual memory)
    // gets hairy.
    let is_async = {
        if cause >> 63 & 1 == 1 {
            true
        } else {
            false
        }
    };
    // The cause contains the type of trap (sync, async) as well as the cause
    // number. So, here we narrow down just the cause number.

    // 0xfff = 1111_1111_1111
    let cause_num = cause & 0xfff;
    let mut return_pc = epc;
    if is_async {
        // Asynchronous trap
        match cause_num {
            3 => {
                // Machine software
                println!("Machine software interrupt CPU#{}", hart);
            }
            7 => unsafe {
                // Machine timer
                let mtimecmp = 0x0200_4000 as *mut u64;
                let mtime = 0x0200_bff8 as *const u64;
                // The frequency given by QEMU is 10_000_000 Hz, so this sets
                // the next interrupt to fire one second from now.
                mtimecmp.write_volatile(mtime.read_volatile() + 10_000_000);
            },
            11 => {
                // Machine external (interrupt from Platform Interrupt Controller (PLIC))
                if let Some(interrupt) = plic::next() {
                    match interrupt {
                        10 => {
                            let mut uart = uart::Uart::new(0x1000_0000);
                            if let Some(c) = uart.get() {
                                match c {
                                    8 | 127 => {
                                        // This is a backspace, so we
                                        // essentially have to write a space and
                                        // backup again:
                                        print!("{} {}", 8 as char, 8 as char);
                                    }
                                    10 | 13 => {
                                        // Newline or carriage-return
                                        println!();
                                    }
                                    _ => {
                                        print!("{}", c as char);
                                    }
                                }
                            }
                        }
                        _ => {
                            println!("Non-UART external interrupt: {}", interrupt);
                        }
                    }
                    plic::complete(interrupt);
                }
            }
            _ => {
                panic!("Unhandled async trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    } else {
        // Synchronous trap
        match cause_num {
            2 => {
                // Illeagal instruction
                panic!(
                    "Illeagal instruction CPU#{} -> 0x{:08x}: 0x{:08x}\n",
                    hart, epc, tval
                );
            }
            8 => {
                // Environment (system) call from User mode
                println!("E-call from User mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            }
            9 => {
                // Environment (system) call from Supervisor mode
                println!("E-call from Supervisor mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            }
            11 => {
                // Environment (system) call from Machine mode
                println!("E-call from Machine mode! CPU#{} -> 0x{:08x}", hart, epc);
                return_pc += 4;
            }
            12 => {
                // Instruction page fault
                println!(
                    "Instruction page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            13 => {
                // Load page fault
                println!(
                    "Load page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            15 => {
                // Store page fault
                println!(
                    "Store page fault CPU#{} -> 0x{:08x}: 0x{:08x}",
                    hart, epc, tval
                );
                return_pc += 4;
            }
            _ => {
                panic!("Unhandled sync trap CPU#{} -> {}\n", hart, cause_num);
            }
        }
    }

    // Finally, return the updated program counter
    return_pc
}
