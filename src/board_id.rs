// Read HwRev at +0x2084 from NOR flash

use crate::mmio;

const HW_REVISION: usize = 0x20002084;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Model {
    Mini1g,
    Photo,
    Unknown,
}

pub fn model() -> Model {
    match unsafe { mmio::read::<u32>(HW_REVISION) } >> 16 {
        0x4 => Model::Mini1g,
        0x6 => Model::Photo,
        _ => Model::Unknown,
    }
}
