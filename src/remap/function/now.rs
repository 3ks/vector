use chrono::Utc;
use remap::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Now;

impl Function for Now {
    fn identifier(&self) -> &'static str {
        "now"
    }

    fn compile(&self, _: ArgumentList) -> Result<Box<dyn Expression>> {
        Ok(Box::new(NowFn))
    }
}

#[derive(Debug, Clone)]
struct NowFn;

impl Expression for NowFn {
    fn execute(&self, _: &mut state::Program, _: &mut dyn Object) -> Result<Option<Value>> {
        Ok(Some(Utc::now().into()))
    }

    fn type_def(&self, _: &state::Compiler) -> TypeDef {
        TypeDef {
            constraint: value::Kind::Timestamp.into(),
            ..Default::default()
        }
    }
}
