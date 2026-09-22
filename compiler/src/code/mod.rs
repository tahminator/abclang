pub mod error;

use bytes::{BufMut, Bytes, BytesMut};
use phf::phf_map;
use strum::{Display, IntoStaticStr};

use crate::code::error::CodeError;

pub type Instructions = Bytes;

#[derive(Debug, PartialEq, Copy, Clone, Eq, IntoStaticStr, Display)]
#[repr(u8)]
pub enum OpCode {
    OpConstant,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub op_widths: &'static [u8],
}

impl TryFrom<OpCode> for Definition {
    type Error = CodeError;

    fn try_from(value: OpCode) -> Result<Self, Self::Error> {
        OP_CODE_DEFINITIONS
            .get(value.into())
            .cloned()
            .ok_or(CodeError::FailedToLookupDefinitionFromOpCode(value))
    }
}

static OP_CODE_DEFINITIONS: phf::Map<&'static str, Definition> = phf_map! {
    "OpConstant" => Definition {
        op_widths: &[2],
    }
};

pub fn make(op: OpCode, operands: &[u16]) -> Option<Instructions> {
    let def = OP_CODE_DEFINITIONS.get(op.into())?;

    if operands.len() != def.op_widths.len() {
        return None;
    }

    let instr_len = 1 + def.op_widths.iter().map(|&w| usize::from(w)).sum::<usize>();
    let mut instr = BytesMut::with_capacity(instr_len);
    instr.put_u8(op as u8);

    for (&operand, &width) in operands.iter().zip(def.op_widths) {
        match width {
            2 => instr.put_u16(operand),
            _ => return None,
        }
    }

    Some(instr.freeze())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make() {
        struct Test {
            op: OpCode,
            operands: Vec<u16>,
            expected: Vec<u8>,
        }

        let tests = [Test {
            op: OpCode::OpConstant,
            operands: vec![65534],
            expected: vec![OpCode::OpConstant as u8, 255, 254],
        }];

        for t in &tests {
            let instruction = make(t.op, &t.operands).expect("valid instruction");
            assert_eq!(instruction.as_ref(), t.expected.as_slice());
        }
    }

    #[test]
    fn test_make_operand_boundaries() {
        for operand in [0, 255, 256, u16::MAX] {
            let instruction = make(OpCode::OpConstant, &[operand]).unwrap();
            let [high, low] = operand.to_be_bytes();
            assert_eq!(instruction.as_ref(), &[OpCode::OpConstant as u8, high, low]);
        }
    }

    #[test]
    fn test_make_rejects_wrong_operand_count() {
        assert!(make(OpCode::OpConstant, &[]).is_none());
        assert!(make(OpCode::OpConstant, &[1, 2]).is_none());
    }
}
