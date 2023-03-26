use cairo_lang_semantic::{ConcreteFunctionWithBodyId, FunctionId};
use cairo_lang_syntax::node::ids::SyntaxStablePtrId;
use cairo_lang_utils::define_short_id;

use crate::db::LoweringGroup;

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum LoweredFunctionId {
    Root(ConcreteFunctionWithBodyId),
    Generated { parent: ConcreteFunctionWithBodyId, element: SyntaxStablePtrId },
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum FinalFunctionLongId {
    FunctionId(FunctionId),
    Generated { parent: ConcreteFunctionWithBodyId, element: SyntaxStablePtrId },
}
define_short_id!(FinalFunctionId, FinalFunctionLongId, LoweringGroup, lookup_intern_final_function);
