use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use crate::core::parser::segments::base::Segment;
use crate::core::rules::base::{LintResult, Rule};
use crate::core::rules::context::RuleContext;
use crate::core::rules::crawlers::{BaseCrawler, SegmentSeekerCrawler};
use crate::helpers::Boxed;

#[derive(Debug, Default)]
pub struct RuleAL08 {}

impl Rule for RuleAL08 {
    fn eval(&self, context: RuleContext) -> Vec<LintResult> {
        let mut used_aliases = HashMap::new();
        let mut violations = Vec::new();

        for clause_element in context.segment.children(&["select_clause_element"]) {
            let mut column_alias = None;

            if let Some(_alias_expression) = clause_element.child(&["alias_expression"]) {
            } else {
                if let Some(column_reference) = clause_element.child(&["column_reference"]) {
                    column_alias = column_reference.get_segments().pop();
                }
            }

            let Some(column_alias) = column_alias else { continue };

            let key = column_alias.get_raw_upper().unwrap().replace(['\"', '\'', '`'], "");

            match used_aliases.entry(key) {
                Entry::Occupied(entry) => {
                    let previous: &Box<dyn Segment> = entry.get();

                    let alias = column_alias.get_raw().unwrap();
                    let line_no = previous.get_position_marker().unwrap().source_position().0;

                    violations.push(LintResult::new(
                        column_alias.clone().into(),
                        vec![],
                        None,
                        format!("Reuse of column alias {alias} from line {line_no}.").into(),
                        None,
                    ))
                }
                Entry::Vacant(entry) => _ = entry.insert(clause_element),
            };
        }

        violations
    }

    fn crawl_behaviour(&self) -> Box<dyn BaseCrawler> {
        SegmentSeekerCrawler::new(HashSet::from(["select_clause"])).boxed()
    }
}

#[cfg(test)]
mod tests {
    use crate::api::simple::lint;
    use crate::core::errors::SQLLintError;
    use crate::core::rules::base::Erased;
    use crate::rules::aliasing::AL08::RuleAL08;

    #[test]
    fn test_fail_references() {
        let sql = "select foo, foo";
        let result =
            lint(sql.to_string(), "ansi".into(), vec![RuleAL08::default().erased()], None, None)
                .unwrap();

        assert_eq!(
            result,
            vec![SQLLintError { description: "Reuse of column alias foo from line 1.".into() }]
        )
    }
}
