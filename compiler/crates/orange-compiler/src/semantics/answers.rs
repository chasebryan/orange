//! Known-answer tests: `test "TITLE" { ... }` declarations, each a closed
//! `Bool` expression checked as a function without parameters and named by
//! its title.

use super::*;

/// The most bytes a test's title holds.
pub const MAX_TEST_TITLE_BYTES: usize = 128;

/// The note of every title diagnostic.
const TITLE_NOTE: &str = "a test's title is 1 through 128 printable ASCII characters, with no \
     backslash, and no two tests of a module share one";

/// What is wrong with a title's spelling, if anything.
enum TitleFault {
    Empty,
    Long(usize),
    /// A character that is not printable ASCII, or a backslash, at its byte
    /// offset in the title.
    Character(usize, char),
}

fn title_fault(text: &str) -> Option<TitleFault> {
    if text.is_empty() {
        return Some(TitleFault::Empty);
    }
    if let Some((offset, character)) = text
        .char_indices()
        .find(|&(_, character)| !(' '..='~').contains(&character) || character == '\\')
    {
        return Some(TitleFault::Character(offset, character));
    }
    (text.len() > MAX_TEST_TITLE_BYTES).then_some(TitleFault::Long(text.len()))
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Checks the module's known-answer tests after its functions, in source
    /// order: each title, for one event, and then each body as the body of a
    /// `spec` without parameters whose result type is `Bool`. The tests take
    /// the identities after `first_id`, in order, and each checked test is
    /// appended to `pending` with its title.
    ///
    /// Calls from tests are recorded in `edges`, apart from the functions'
    /// own, since no function calls a test and a test closes no cycle.
    pub(super) fn analyze_tests(
        &mut self,
        scope: &ModuleScope<'_, 'ast>,
        first_id: usize,
        pending: &mut Vec<PendingFunction>,
        edges: &mut Vec<CallEdge>,
    ) {
        let tests = &self.ast.module.tests;
        let mut titles: Vec<(&'ast str, usize)> = Vec::new();
        if titles.try_reserve_exact(tests.len()).is_err() {
            self.resource_limit(self.ast.module.span, "test title index allocation failed");
            return;
        }
        titles.extend(
            tests
                .iter()
                .enumerate()
                .map(|(index, test)| (test.title.text.as_str(), index)),
        );
        titles.sort_unstable();
        for (index, test) in tests.iter().enumerate() {
            // One event for the title's check.
            if !self.event(test.title.span) {
                return;
            }
            let first = titles
                .get(titles.partition_point(|&(title, _)| title < test.title.text.as_str()))
                .map(|&(_, first)| first);
            let earlier = first
                .filter(|&first| first != index)
                .and_then(|first| tests.get(first));
            self.check_test_title(test, earlier);
            let function = test.function();
            let FunctionBody::Typed(body) = &function.body else {
                self.resource_limit(test.span, "test body is inconsistent");
                return;
            };
            let Some(id) = first_id
                .checked_add(index)
                .and_then(CoreFunctionId::from_index)
            else {
                self.resource_limit(
                    test.span,
                    "Core function identity exceeds the u32 representation limit",
                );
                return;
            };
            let context = BodyContext {
                id,
                instance: Instance::NONE,
                name: &function.name,
                parameters: &function.parameters,
                parameter_types: Vec::new(),
                bindings: &body.bindings,
                binding_types: Vec::new(),
                loop_scopes: Vec::new(),
                blocks: Vec::new(),
                finished_blocks: Vec::new(),
                loops: Vec::new(),
                conditionals: Vec::new(),
            };
            let checked = self.analyze_typed_function(function, body, context, scope, edges);
            if self.halted {
                return;
            }
            let Some(mut checked) = checked else {
                continue;
            };
            let Some(title) = self.copy_core_name(&test.title.text, test.title.span) else {
                return;
            };
            checked.title = Some(title);
            if (self.reserve_pending_function_slot)(pending) {
                pending.push(checked);
            } else {
                self.resource_limit(
                    test.span,
                    "semantic analysis could not allocate pending test storage",
                );
                return;
            }
        }
    }

    /// Reports a title that is empty, too long, or holds a character other
    /// than printable ASCII or a backslash, or that repeats `earlier`'s.
    fn check_test_title(&mut self, test: &TestDeclaration, earlier: Option<&TestDeclaration>) {
        let span = test.title.span;
        let (message, label) = match title_fault(&test.title.text) {
            Some(TitleFault::Empty) => (
                String::from("this test's title is empty"),
                String::from("a title names the test in every report"),
            ),
            Some(TitleFault::Character(_, '\\')) => (
                String::from("a test's title holds no backslash"),
                String::from("titles have no escapes"),
            ),
            Some(TitleFault::Character(offset, character)) => (
                format!(
                    "a test's title holds {}, which is not printable ASCII",
                    character_for_diagnostic(character)
                ),
                format!("at byte {offset} of the title"),
            ),
            Some(TitleFault::Long(length)) => (
                format!("this test's title is {length} bytes long"),
                format!("a title holds at most {MAX_TEST_TITLE_BYTES} bytes"),
            ),
            None => match earlier {
                Some(earlier) => {
                    if self.begin_report(span) {
                        self.diagnostics.push(
                            Diagnostic::error(
                                DiagnosticCode::TestTitle,
                                "two tests of this module share a title",
                                span,
                            )
                            .with_label("this title repeats an earlier test's")
                            .with_secondary_span(earlier.title.span, "first test is here")
                            .with_note(TITLE_NOTE),
                        );
                    }
                    return;
                }
                None => return,
            },
        };
        if self.begin_report(span) {
            self.diagnostics.push(
                Diagnostic::error(DiagnosticCode::TestTitle, message, span)
                    .with_label(label)
                    .with_note(TITLE_NOTE),
            );
        }
    }
}

/// Spells a character of a title for a diagnostic: printable ASCII in
/// backquotes, and everything else by its code point.
fn character_for_diagnostic(character: char) -> String {
    if (' '..='~').contains(&character) {
        format!("`{character}`")
    } else {
        format!("U+{:04X}", u32::from(character))
    }
}
