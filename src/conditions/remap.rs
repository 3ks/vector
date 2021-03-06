use crate::{
    conditions::{Condition, ConditionConfig, ConditionDescription},
    emit,
    internal_events::RemapConditionExecutionFailed,
    Event,
};
use remap::{value, Program, RemapError, Runtime, TypeDef, Value};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct RemapConfig {
    source: String,
}

inventory::submit! {
    ConditionDescription::new::<RemapConfig>("remap")
}

impl_generate_config_from_default!(RemapConfig);

#[typetag::serde(name = "remap")]
impl ConditionConfig for RemapConfig {
    fn build(&self) -> crate::Result<Box<dyn Condition>> {
        let expected_result = TypeDef {
            fallible: true,
            optional: true,
            kind: value::Kind::Boolean,
        };

        let program = Program::new(&self.source, &crate::remap::FUNCTIONS, expected_result)
            .map_err(|e| e.to_string())?;

        Ok(Box::new(Remap { program }))
    }
}

//------------------------------------------------------------------------------

#[derive(Clone)]
pub struct Remap {
    program: Program,
}

impl Remap {
    fn execute(&self, event: &Event) -> Result<remap::Value, RemapError> {
        // TODO(jean): This clone exists until remap-lang has an "immutable"
        // mode.
        //
        // For now, mutability in reduce "remap ends-when conditions" is
        // allowed, but it won't mutate the original event, since we cloned it
        // here.
        //
        // Having first-class immutability support in the language allows for
        // more performance (one less clone), and boot-time errors when a
        // program wants to mutate its events.
        //
        // see: https://github.com/timberio/vector/issues/4744
        Runtime::default().execute(&mut event.clone(), &self.program)
    }
}

impl Condition for Remap {
    fn check(&self, event: &Event) -> bool {
        self.execute(&event)
            .map(|value| match value {
                Value::Boolean(boolean) => boolean,
                _ => unreachable!("boolean type constraint set"),
            })
            .unwrap_or_else(|_| {
                emit!(RemapConditionExecutionFailed);
                false
            })
    }

    fn check_with_context(&self, event: &Event) -> Result<(), String> {
        let value = self
            .execute(event)
            .map_err(|err| format!("source execution failed: {:#}", err))?;

        match value {
            Value::Boolean(v) if v => Ok(()),
            Value::Boolean(v) if !v => Err("source execution resolved to false".into()),
            _ => unreachable!("boolean type constraint set"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::log_event;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<RemapConfig>();
    }

    #[test]
    fn check_remap() {
        let checks = vec![
            (
                log_event![],   // event
                "true == true", // source
                Ok(()),         // build result
                Ok(()),         // check result
            ),
            (
                log_event!["foo" => true, "bar" => false],
                "to_bool(.bar || .foo)",
                Ok(()),
                Ok(()),
            ),
            (
                log_event![],
                "true == false",
                Ok(()),
                Err("source execution resolved to false"),
            ),
            (
                log_event![],
                "",
                Err("remap error: program error: expected to resolve to boolean value, but instead resolves to any value"),
                Ok(()),
            ),
            (
                log_event!["foo" => "string"],
                ".foo",
                Err("remap error: program error: expected to resolve to boolean or no value, but instead resolves to any value"),
                Ok(()),
            ),
            (
                log_event![],
                ".",
                Err(
                    "remap error: parser error:  --> 1:2\n  |\n1 | .\n  |  ^---\n  |\n  = expected path_segment",
                ),
                Ok(()),
            ),
        ];

        for (event, source, build, check) in checks {
            let source = source.to_owned();
            let config = RemapConfig { source };

            assert_eq!(
                config.build().map(|_| ()).map_err(|e| e.to_string()),
                build.map_err(|e| e.to_string())
            );

            if let Ok(cond) = config.build() {
                assert_eq!(
                    cond.check_with_context(&event),
                    check.map_err(|e| e.to_string())
                );
            }
        }
    }
}
