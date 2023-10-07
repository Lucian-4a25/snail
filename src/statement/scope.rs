use std::vec;

#[derive(Debug)]
pub struct Scope {
    pub flags: u32,
    // list of var-declared names
    pub var: Vec<String>,
    // list of lexical names
    pub lexial: Vec<String>,
    // list of lexical function names
    pub functions: Vec<String>,
    // a flag indicate if identifier reference 'arguments'
    pub in_class_field_init: bool,
}

impl Scope {
    pub fn new(flags: u32) -> Self {
        Self {
            flags,
            var: vec![],
            lexial: vec![],
            functions: vec![],
            in_class_field_init: false,
        }
    }
}

// Each scope gets a bitset that may contain these flags
pub const SCOPE_TOP: u32 = 0b1;
pub const SCOPE_FUNCTION: u32 = 0b11;
pub const SCOPE_ASYNC: u32 = 0b111;
pub const SCOPE_GENERATOR: u32 = 0b1_111;
pub const SCOPE_ARROW: u32 = 0b11_111;
pub const SCOPE_SIMPLE_CATCH: u32 = 0b111_111;
pub const SCOPE_SUPER: u32 = 0b1_111_111;
pub const SCOPE_DIRECT_SUPER: u32 = 0b11_111_111;
pub const SCOPE_CLASS_STATIC_BLOCK: u32 = 0b111_111_111;
pub const SCOPE_VAR: u32 = SCOPE_TOP | SCOPE_FUNCTION | SCOPE_CLASS_STATIC_BLOCK;

pub fn get_func_flags(asy: bool, generator: bool) -> u32 {
    SCOPE_FUNCTION | if asy { SCOPE_ASYNC } else { 0 } | if generator { SCOPE_GENERATOR } else { 0 }
}
