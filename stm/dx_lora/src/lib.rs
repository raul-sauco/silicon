#![no_std]

use core::fmt::Write;
use heapless::String;

pub const NODE_WATER_METER: u8 = 0x01;

pub const FLAG_LOW_BATTERY: u8 = 0b0000_0001;
pub const FLAG_FLOW_ACTIVE: u8 = 0b0000_0010;
// pub const FLAG_LEAK_ALERT: u8 = 0b0000_0100;

pub const FREQUENCY_HZ: u32 = 869_525_000;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct WaterMeterPayload {
    pub node_id: u8,
    pub packet_id: u8,
    pub edge_count: u32,
    pub battery_mv: u16,
    pub soc: u8,
    pub flags: u8,
}

impl WaterMeterPayload {
    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < core::mem::size_of::<Self>() {
            return None;
        }
        Some(unsafe { core::ptr::read_unaligned(bytes.as_ptr() as *const Self) })
    }

    pub fn to_log_string(&self) -> String<64> {
        // copy fields to locals to avoid unaligned reference UB
        let node_id = self.node_id;
        let packet_id = self.packet_id;
        let edge_count = self.edge_count;
        let battery_mv = self.battery_mv;
        let soc = self.soc;
        let flags = self.flags;

        let mut s: String<64> = String::new();
        let _ = write!(
            s,
            "{},{},{},{},{},{:08b}",
            node_id, packet_id, edge_count, battery_mv, soc, flags
        );
        s
    }
}
