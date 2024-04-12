use crate::prelude::*;

#[derive(PartialEq, Debug, Clone, Copy, EnumNum)]
#[repr(u64)]
pub enum ReplyId {
    InstantiateExecutor = 0,
    ExecProgram = 2,
    TransportSent = 3,
}

impl From<ReplyId> for u64 {
    fn from(val: ReplyId) -> Self {
        val as u64
    }
}
