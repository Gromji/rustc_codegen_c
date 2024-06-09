use core::fmt;
use std::fmt::Debug;

use crate::{
    crepr::{indent, Representable, RepresentationContext},
    expression::Expression,
};

#[derive(Clone)]
pub struct StaticAllocation {
    bytes: Vec<u8>,

    // {ctype} ptr_{offset} = {expr};
    ptrs: Vec<(usize, Expression)>,
    name: String,
}

impl StaticAllocation {
    const BYTES_PREFIX: &'static str = "bytes_";
    const PTRS_PREFIX: &'static str = "ptr_";

    pub fn new(name: String, bytes: Vec<u8>, ptrs: Vec<(usize, Expression)>) -> Self {
        Self { bytes, ptrs, name }
    }

    pub fn ptr_size() -> usize {
        return 8; //TODO: hardcoded pointer size, unsure of where to grab this from
    }
}

impl StaticAllocation {
    fn build_bytes_definition(
        &self,
        f: &mut (dyn fmt::Write),
        _context: &RepresentationContext,
        bytes_num: usize,
        from: usize,
        to: usize,
    ) -> fmt::Result {
        write!(f, "int8_t {}{}[{}];", StaticAllocation::BYTES_PREFIX, bytes_num, to - from)
    }

    fn build_ptr_definition(
        &self,
        f: &mut (dyn fmt::Write),
        _context: &RepresentationContext,
        ptr_idx: usize,
    ) -> fmt::Result {
        write!(
            f,
            "void* {}{}; /* offset {} */",
            StaticAllocation::PTRS_PREFIX,
            ptr_idx,
            self.ptrs.get(ptr_idx).map_or_else(|| 0, |(offset, _)| *offset)
        )
    }

    fn build_bytes_declaration(
        &self,
        f: &mut (dyn fmt::Write),
        from: usize,
        to: usize,
    ) -> fmt::Result {
        write!(f, "{{")?;

        for i in from..to {
            if i > from {
                write!(f, ", ")?;
            }
            write!(f, "0x{:02X?}", self.bytes[i])?;
        }

        write!(f, "}}")
    }

    fn next_ptr_offset(&self, ptr_idx: usize) -> usize {
        self.ptrs.get(ptr_idx).map_or_else(|| self.bytes.len(), |(offset, _)| *offset)
    }

    fn build_declaration(
        &self,
        f: &mut (dyn fmt::Write),
        context: &mut RepresentationContext,
    ) -> fmt::Result {
        // prelude
        write!(f, "struct {{")?;
        self.newline(f, context)?;

        let mut byte_idx = 0;
        let mut ptr_idx = 0;
        let mut cur_idx = 0;

        while cur_idx < self.bytes.len() {
            let next_ptr_offset = self.next_ptr_offset(ptr_idx);

            if cur_idx < next_ptr_offset {
                indent(f, context)?;
                self.build_bytes_definition(f, context, byte_idx, cur_idx, next_ptr_offset)?;
                self.newline(f, context)?;

                byte_idx += 1;
                cur_idx += next_ptr_offset - cur_idx;
            } else {
                indent(f, context)?;
                self.build_ptr_definition(f, context, ptr_idx)?;
                self.newline(f, context)?;

                ptr_idx += 1;
                cur_idx += StaticAllocation::ptr_size();
            }
        }

        write!(f, "}} {}", self.name)?;
        write!(f, " = {{")?;
        self.newline(f, context)?;

        cur_idx = 0;
        ptr_idx = 0;

        while cur_idx < self.bytes.len() {
            let next_ptr_offset = self.next_ptr_offset(ptr_idx);

            if cur_idx > 0 {
                write!(f, ",")?;
                self.newline(f, context)?;
            }

            if cur_idx < next_ptr_offset {
                indent(f, context)?;
                self.build_bytes_declaration(f, cur_idx, next_ptr_offset)?;

                byte_idx += 1;
                cur_idx += next_ptr_offset - cur_idx;
            } else {
                indent(f, context)?;

                self.ptrs[ptr_idx].1.repr(f, context)?;

                ptr_idx += 1;
                cur_idx += StaticAllocation::ptr_size();
            }
        }

        self.newline(f, context)?;

        write!(f, "}};")
    }
}

impl Representable for StaticAllocation {
    fn repr(&self, f: &mut (dyn fmt::Write), context: &mut RepresentationContext) -> fmt::Result {
        self.build_declaration(f, context)
    }
}

impl Debug for StaticAllocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}
