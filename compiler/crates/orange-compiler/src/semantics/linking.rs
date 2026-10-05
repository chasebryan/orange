//! Module linking: the graph of a program's `use` declarations, the
//! program-wide checks it drives, and the call graph of each module.

use super::*;

/// The checked graph of a program's `use` declarations.
pub(super) struct ModuleGraph {
    /// Program indices of the reachable modules in dependency order: each
    /// module after every module it uses, the root last.
    pub(super) order: Vec<usize>,
    /// For each program module, the program index of the module each of its
    /// `use` declarations names, in source order; empty for a module the root
    /// does not reach.
    pub(super) targets: Vec<Vec<usize>>,
}

/// Retains the module graph's diagnostics under the per-source bound.
pub(super) struct GraphReport {
    pub(super) diagnostics: Vec<Diagnostic>,
    pub(super) ordinary: usize,
    pub(super) limit_reported: bool,
}

impl GraphReport {
    pub(super) fn report(&mut self, build: impl FnOnce() -> Diagnostic) {
        if self.ordinary < MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE {
            self.ordinary = self.ordinary.saturating_add(1);
            self.diagnostics.push(build());
        } else if !self.limit_reported {
            self.limit_reported = true;
            let span = build().primary_span();
            self.diagnostics.push(
                Diagnostic::error(
                    DiagnosticCode::TooManySemanticErrors,
                    "too many semantic errors; further errors are suppressed",
                    span,
                )
                .with_label("semantic diagnostic limit reached")
                .with_note(format!(
                    "at most {MAX_SEMANTIC_DIAGNOSTICS_PER_SOURCE} ordinary semantic diagnostics \
                     are retained for a program's modules and their uses"
                )),
            );
        }
    }
}

impl ModuleGraph {
    /// Checks module names and `use` declarations and orders the modules the
    /// root reaches, stopping at the module limit. Only reachable modules are
    /// entered, at most 64 of them with at most 64 uses each, and each is
    /// compared with every supplied module once per name it declares or
    /// uses, so the work is linear in the number of supplied modules. It
    /// consumes no semantic events.
    pub(super) fn build(
        program: &[(&SourceFile, &SyntaxTree)],
        root_span: Span,
    ) -> Result<Self, Vec<Diagnostic>> {
        let count = program.len();
        let failure = |span: Span| {
            program_resource_limit(span, "module graph storage allocation failed").diagnostics
        };
        let mut report = GraphReport {
            diagnostics: Vec::new(),
            ordinary: 0,
            limit_reported: false,
        };
        let mut targets: Vec<Vec<usize>> = Vec::new();
        let mut state = Vec::new();
        let mut order = Vec::new();
        let mut path: Vec<(usize, usize)> = Vec::new();
        let reachable = count.min(MAX_MODULES_PER_PROGRAM);
        if report
            .diagnostics
            .try_reserve_exact(MAX_RETAINED_SEMANTIC_DIAGNOSTICS)
            .is_err()
            || targets.try_reserve_exact(count).is_err()
            || state.try_reserve_exact(count).is_err()
            || order.try_reserve_exact(reachable).is_err()
            || path.try_reserve_exact(reachable).is_err()
        {
            return Err(failure(root_span));
        }
        let name_of = |index: usize| {
            program
                .get(index)
                .map_or("", |(_, ast)| ast.module.name.text.as_str())
        };

        targets.extend(program.iter().map(|_| Vec::new()));
        state.extend(program.iter().map(|_| VisitState::Unvisited));

        // A depth-first search from the root enters each reachable module
        // once, checks that no other supplied module shares its name, resolves
        // its uses, and finds each cycle at the use that closes it. Finished
        // modules follow the modules they use. A module the root does not
        // reach is never entered.
        let mut entered = 0_usize;
        if let Some(slot) = state.first_mut() {
            *slot = VisitState::OnPath;
            path.push((0, 0));
            entered = 1;
            report_namesakes(program, 0, &mut report);
            if !resolve_uses(program, 0, &mut targets, &mut report) {
                return Err(failure(root_span));
            }
        }
        while let Some((node, next_use)) = path.last().copied() {
            let uses = program
                .get(node)
                .map_or(&[][..], |(_, ast)| ast.module.uses.as_slice());
            let resolved = targets.get(node).map_or(&[][..], Vec::as_slice);
            let Some((declaration, target)) = uses.get(next_use).zip(resolved.get(next_use)) else {
                path.pop();
                if let Some(slot) = state.get_mut(node) {
                    *slot = VisitState::Done;
                }
                order.push(node);
                continue;
            };
            let target = *target;
            if let Some(top) = path.last_mut() {
                top.1 = next_use.saturating_add(1);
            }
            match state.get(target) {
                Some(VisitState::Unvisited) => {
                    if entered >= MAX_MODULES_PER_PROGRAM {
                        return Err(program_resource_limit(
                            root_span,
                            &format!("program reaches more than {MAX_MODULES_PER_PROGRAM} modules"),
                        )
                        .diagnostics);
                    }
                    entered = entered.saturating_add(1);
                    if let Some(slot) = state.get_mut(target) {
                        *slot = VisitState::OnPath;
                    }
                    path.push((target, 0));
                    report_namesakes(program, target, &mut report);
                    if !resolve_uses(program, target, &mut targets, &mut report) {
                        return Err(failure(declaration.span));
                    }
                }
                Some(VisitState::OnPath) => {
                    report.report(|| {
                        let start = path
                            .iter()
                            .position(|(module, _)| *module == target)
                            .unwrap_or(0);
                        let mut route = String::new();
                        for (position, (module, _)) in
                            path.get(start..).unwrap_or_default().iter().enumerate()
                        {
                            if position >= MAX_MODULES_IN_CYCLE_DIAGNOSTIC {
                                route.push_str(" -> ...");
                                break;
                            }
                            if position != 0 {
                                route.push_str(" -> ");
                            }
                            route.push('`');
                            route.push_str(
                                &identifier_spelling_for_diagnostic(name_of(*module)).to_string(),
                            );
                            route.push('`');
                        }
                        let target_name = identifier_spelling_for_diagnostic(name_of(target));
                        Diagnostic::error(
                            DiagnosticCode::ModuleCycle,
                            format!("module cycle {route} -> `{target_name}`"),
                            declaration.span,
                        )
                        .with_label("this `use` closes the cycle")
                        .with_note(
                            "modules may not depend on each other in a cycle; move the functions \
                             they share into a module that both use",
                        )
                    });
                }
                // A use that names no module was reported, and a finished
                // module adds no cycle.
                Some(VisitState::Done) | None => {}
            }
        }
        if report.diagnostics.is_empty() {
            Ok(Self { order, targets })
        } else {
            Err(report.diagnostics)
        }
    }
}

/// Reports each other supplied module whose name is that of `module`, a
/// module of the program the search has just entered. A `use` resolves to
/// the first supplied module of its name, so the module entered precedes its
/// namesakes, and each pair is reported once, at the later module.
pub(super) fn report_namesakes(
    program: &[(&SourceFile, &SyntaxTree)],
    module: usize,
    report: &mut GraphReport,
) {
    let Some((_, ast)) = program.get(module) else {
        return;
    };
    let name = &ast.module.name;
    for (other, (_, candidate)) in program.iter().enumerate() {
        let repeat = &candidate.module.name;
        if other == module || repeat.text != name.text {
            continue;
        }
        let (first, repeat) = if other < module {
            (repeat.span, name.span)
        } else {
            (name.span, repeat.span)
        };
        report.report(|| {
            let spelling = identifier_spelling_for_diagnostic(&name.text);
            Diagnostic::error(
                DiagnosticCode::DuplicateModule,
                format!("duplicate module `{spelling}`"),
                repeat,
            )
            .with_label("this module repeats the name of a module of the program")
            .with_secondary_span(first, "first module of this name is here")
            .with_note(
                "a `use` names one module, so no other supplied module may share the name of a \
                 module of the program",
            )
        });
    }
}

/// Resolves the `use` declarations of one module when the search first
/// enters it, reporting self-uses, repeated uses, and unknown modules. An
/// unresolved use gets a target past the program, which the search skips.
/// Returns whether the targets could be stored.
pub(super) fn resolve_uses(
    program: &[(&SourceFile, &SyntaxTree)],
    module: usize,
    targets: &mut [Vec<usize>],
    report: &mut GraphReport,
) -> bool {
    let Some((_, ast)) = program.get(module) else {
        return false;
    };
    let uses = &ast.module.uses;
    let Some(resolved) = targets.get_mut(module) else {
        return false;
    };
    if resolved.try_reserve_exact(uses.len()).is_err() {
        return false;
    }
    let own = &ast.module.name;
    for (index, declaration) in uses.iter().enumerate() {
        let name = &declaration.name;
        let earlier = uses.get(..index).and_then(|earlier| {
            earlier
                .iter()
                .find(|candidate| candidate.name.text == name.text)
        });
        let target = program
            .iter()
            .position(|(_, candidate)| candidate.module.name.text == name.text);
        if let Some(earlier) = earlier {
            report.report(|| {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                Diagnostic::error(
                    DiagnosticCode::DuplicateModule,
                    format!("module `{spelling}` is used twice"),
                    declaration.span,
                )
                .with_label("this declaration repeats an earlier `use`")
                .with_secondary_span(earlier.span, "first used here")
                .with_note("a module names each module it uses once")
            });
            resolved.push(usize::MAX);
        } else if name.text == own.text {
            report.report(|| {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                Diagnostic::error(
                    DiagnosticCode::ModuleCycle,
                    format!("module `{spelling}` uses itself"),
                    declaration.span,
                )
                .with_label("this `use` names its own module")
                .with_note("a module calls its own functions without a module name, as in `f(x)`")
            });
            resolved.push(usize::MAX);
        } else if let Some(target) = target {
            resolved.push(target);
        } else {
            report.report(|| {
                let spelling = identifier_spelling_for_diagnostic(&name.text);
                Diagnostic::error(
                    DiagnosticCode::UnknownModule,
                    format!("no module named `{spelling}` in this program"),
                    name.span,
                )
                .with_label("unknown module")
                .with_note(
                    "a `use` declaration names another module of the program; `orangec` \
                     reads the module `NAME` from the file `NAME.or` beside the file that uses it",
                )
            });
            resolved.push(usize::MAX);
        }
    }
    true
}

/// The name tables of a checked module that later modules call into.
pub(super) struct ModuleTables<'ast> {
    pub(super) declarations: DeclarationIndex<'ast>,
    pub(super) signatures: Vec<Option<Signature<'ast>>>,
}

/// What analysis of one module of a program produced.
pub(super) struct ModuleOutcome<'ast> {
    pub(super) core: Option<CoreModule>,
    pub(super) diagnostics: Vec<Diagnostic>,
    /// Present once the module's declarations and signatures are complete,
    /// even if its bodies have errors.
    pub(super) tables: Option<ModuleTables<'ast>>,
}

/// The number of Core function identities a module takes: one for each
/// instance of each typed `spec`.
pub(super) fn typed_spec_count(source: &SourceFile, ast: &SyntaxTree) -> usize {
    ast.module
        .functions
        .iter()
        .map(|function| instance_count(source, function))
        .fold(0, usize::saturating_add)
}

/// Checks each reachable module in dependency order and links their Core.
pub(super) fn link_program<'ast>(
    program: &[(&SourceFile, &'ast SyntaxTree)],
    graph: &ModuleGraph,
    root_span: Span,
) -> AnalysisResult {
    let mut tables: Vec<Option<ModuleTables<'ast>>> = Vec::new();
    let mut cores: Vec<CoreModule> = Vec::new();
    if tables.try_reserve_exact(program.len()).is_err()
        || cores.try_reserve_exact(graph.order.len()).is_err()
    {
        return program_resource_limit(root_span, "semantic program storage allocation failed");
    }
    tables.extend(program.iter().map(|_| None));
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let mut id_offset = 0_usize;
    let mut complete = true;
    for &index in &graph.order {
        let Some(&(source, ast)) = program.get(index) else {
            return program_resource_limit(root_span, "module graph index is inconsistent");
        };
        let resolved = graph.targets.get(index).map_or(&[][..], Vec::as_slice);
        let mut imports = Vec::new();
        if imports.try_reserve_exact(resolved.len()).is_err() {
            return program_resource_limit(
                ast.module.span,
                "semantic import table allocation failed",
            );
        }
        let mut available = true;
        for (declaration, target) in ast.module.uses.iter().zip(resolved) {
            match tables.get(*target).and_then(Option::as_ref) {
                Some(table) => imports.push(ImportScope {
                    name: &declaration.name.text,
                    declarations: &table.declarations,
                    signatures: &table.signatures,
                }),
                None => available = false,
            }
        }
        // A module whose used module stopped before its names were complete
        // is not checked; that module's analysis has already reported why.
        let outcome = available.then(|| {
            // Only the root's tests are checked; it is the program's first
            // module.
            Analyzer::new(source, ast, Limits::DEFAULT)
                .with_id_offset(id_offset)
                .with_tests(index == 0)
                .run_linked(
                    &imports,
                    |declarations, capacity| declarations.try_reserve(capacity).is_ok(),
                    |functions, capacity| functions.try_reserve_exact(capacity).is_ok(),
                )
        });
        drop(imports);
        let produced = outcome
            .as_ref()
            .and_then(|outcome| outcome.core.as_ref())
            .map_or_else(
                || typed_spec_count(source, ast),
                |core| core.functions.len().saturating_sub(core.tests),
            );
        id_offset = id_offset.saturating_add(produced);
        let Some(outcome) = outcome else {
            complete = false;
            continue;
        };
        if diagnostics
            .try_reserve_exact(outcome.diagnostics.len())
            .is_err()
        {
            return program_resource_limit(
                ast.module.span,
                "semantic program diagnostic storage allocation failed",
            );
        }
        diagnostics.extend(outcome.diagnostics);
        if let Some(slot) = tables.get_mut(index) {
            *slot = outcome.tables;
        }
        match outcome.core {
            Some(core) => cores.push(core),
            None => complete = false,
        }
    }
    if !complete || !diagnostics.is_empty() {
        if diagnostics.is_empty() {
            return program_resource_limit(root_span, "semantic program linking is inconsistent");
        }
        return AnalysisResult {
            core: None,
            diagnostics,
        };
    }
    let Some(mut root) = cores.pop() else {
        return program_resource_limit(root_span, "semantic program linking is inconsistent");
    };
    if cores.is_empty() {
        return AnalysisResult {
            core: Some(root),
            diagnostics,
        };
    }
    let total = cores
        .iter()
        .map(|core| core.functions.len())
        .fold(root.functions.len(), usize::saturating_add);
    let mut functions = Vec::new();
    if functions.try_reserve_exact(total).is_err() {
        return program_resource_limit(root_span, "typed Core function storage allocation failed");
    }
    for core in cores {
        functions.extend(core.functions);
    }
    root.entry = functions.len();
    functions.append(&mut root.functions);
    root.functions = functions;
    AnalysisResult {
        core: Some(root),
        diagnostics,
    }
}

impl<'source, 'ast> Analyzer<'source, 'ast> {
    /// Reports every call cycle among typed specifications.
    ///
    /// A depth-first search in function-ID order examines each function's call
    /// edges in checking order; each edge that returns to a function still on
    /// the search path closes a cycle and is reported once at that call. The
    /// search consumes no semantic events: it visits each function and each
    /// already counted call exactly once.
    pub(super) fn check_call_graph(
        &mut self,
        signatures: &[Option<Signature<'ast>>],
        edges: &[CallEdge],
    ) {
        // Each concrete instance is a node, then each specialization, in
        // the order their identities were reserved.
        let mut names = Vec::new();
        let concrete = signatures
            .iter()
            .flatten()
            .map(|signature| signature.instances.len())
            .fold(0_usize, usize::saturating_add);
        let function_count = concrete.saturating_add(self.spec_keys.len());
        let mut offsets = Vec::new();
        let mut targets: Vec<(CoreFunctionId, usize)> = Vec::new();
        let mut state = Vec::new();
        let mut path = Vec::new();
        if names.try_reserve_exact(function_count).is_err()
            || offsets
                .try_reserve_exact(function_count.saturating_add(1))
                .is_err()
            || targets.try_reserve_exact(edges.len()).is_err()
            || state.try_reserve_exact(function_count).is_err()
            || path.try_reserve_exact(function_count).is_err()
        {
            self.resource_limit(self.ast.module.span, "call graph storage allocation failed");
            return;
        }
        names.extend(
            self.ast
                .module
                .functions
                .iter()
                .zip(signatures)
                .filter_map(|(function, signature)| Some((function, signature.as_ref()?)))
                .flat_map(|(function, signature)| {
                    (0..signature.instances.len()).map(move |index| {
                        signature
                            .ranges
                            .and_then(|ranges| ranges.instance(signature.sizes, index))
                            .unwrap_or(Instance::NONE)
                            .label(&function.name.text, &signature.spellings)
                    })
                }),
        );
        for key in &self.spec_keys {
            let label = self
                .ast
                .module
                .functions
                .get(key.function)
                .zip(signatures.get(key.function).and_then(Option::as_ref))
                .map(|(function, signature)| {
                    Instance {
                        parameters: signature.sizes,
                        values: key.values,
                    }
                    .label(&function.name.text, &signature.spellings)
                })
                .unwrap_or_default();
            names.push(label);
        }
        // Group edges by caller; within one caller, edges keep the order in
        // which their calls finished checking.
        targets.extend(
            edges
                .iter()
                .enumerate()
                .map(|(index, edge)| (edge.caller, index)),
        );
        targets.sort_unstable();
        offsets.push(0_usize);
        let mut cursor = 0_usize;
        for index in 0..function_count {
            let global = index.checked_add(self.id_offset);
            while targets
                .get(cursor)
                .is_some_and(|(caller, _)| usize::try_from(caller.index()).ok() == global)
            {
                cursor = cursor.saturating_add(1);
            }
            offsets.push(cursor);
        }
        state.resize(function_count, VisitState::Unvisited);

        for root in 0..function_count {
            if state.get(root) != Some(&VisitState::Unvisited) {
                continue;
            }
            path.push((root, offsets.get(root).copied().unwrap_or(0)));
            if let Some(slot) = state.get_mut(root) {
                *slot = VisitState::OnPath;
            }
            while let Some((node, next_edge)) = path.last().copied() {
                let end = offsets.get(node.saturating_add(1)).copied().unwrap_or(0);
                if next_edge >= end {
                    path.pop();
                    if let Some(slot) = state.get_mut(node) {
                        *slot = VisitState::Done;
                    }
                    continue;
                }
                if let Some(top) = path.last_mut() {
                    top.1 = next_edge.saturating_add(1);
                }
                let Some(edge) = targets
                    .get(next_edge)
                    .and_then(|(_, index)| edges.get(*index))
                else {
                    self.resource_limit(self.ast.module.span, "call graph index is inconsistent");
                    return;
                };
                let Some(target) = usize::try_from(edge.callee.index())
                    .ok()
                    .and_then(|index| index.checked_sub(self.id_offset))
                else {
                    self.resource_limit(edge.span, "call graph index is inconsistent");
                    return;
                };
                match state.get(target) {
                    Some(VisitState::Unvisited) => {
                        if let Some(slot) = state.get_mut(target) {
                            *slot = VisitState::OnPath;
                        }
                        path.push((target, offsets.get(target).copied().unwrap_or(0)));
                    }
                    Some(VisitState::OnPath) => {
                        self.report_cycle(edge, target, &path, &names);
                        if self.halted {
                            return;
                        }
                    }
                    Some(VisitState::Done) => {}
                    None => {
                        self.resource_limit(edge.span, "call graph index is inconsistent");
                        return;
                    }
                }
            }
        }
    }

    pub(super) fn report_cycle(
        &mut self,
        edge: &CallEdge,
        target: usize,
        path: &[(usize, usize)],
        names: &[String],
    ) {
        // Each instance of a sized function repeats its calls, so a cycle
        // closed at a call already reported, by another instance, is not
        // reported again.
        if self.diagnostics.iter().any(|diagnostic| {
            diagnostic.code() == DiagnosticCode::CallCycle && diagnostic.primary_span() == edge.span
        }) || !self.begin_report(edge.span)
        {
            return;
        }
        let start = path
            .iter()
            .position(|(node, _)| *node == target)
            .unwrap_or(0);
        let cycle = path.get(start..).unwrap_or_default();
        let name_of = |index: usize| names.get(index).map(String::as_str).unwrap_or("");
        let target_name = name_of(target);
        let message = if cycle.len() <= 1 {
            format!("`{target_name}` calls itself")
        } else {
            let mut route = String::new();
            for (position, (node, _)) in cycle.iter().enumerate() {
                if position >= MAX_FUNCTIONS_IN_CYCLE_DIAGNOSTIC {
                    route.push_str(" -> ...");
                    break;
                }
                if position != 0 {
                    route.push_str(" -> ");
                }
                route.push('`');
                route.push_str(name_of(*node));
                route.push('`');
            }
            format!("call cycle {route} -> `{target_name}`")
        };
        self.diagnostics.push(
            Diagnostic::error(DiagnosticCode::CallCycle, message, edge.span)
                .with_label("this call closes the cycle")
                .with_note(
                    "a `spec` may not depend on itself; recursion is not part of Orange 2026",
                ),
        );
    }
}
