//! NOVA Diagnostics System

use std::fmt;
use std::sync::Arc;

use crate::{Span, FileId, SourceMap};

/// Diagnostic severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Compiler bug - should never happen
    Bug,
    /// Error - compilation fails
    Error,
    /// Warning - potential issue
    Warning,
    /// Note - additional information
    Note,
    /// Help - suggestion for fixing
    Help,
}

/// A single diagnostic message
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
    pub code: Option<String>,
    pub labels: Vec<Label>,
    pub notes: Vec<Note>,
    pub children: Vec<Diagnostic>,
}

/// Label attached to a diagnostic (for highlighting source code)
#[derive(Debug, Clone)]
pub struct Label {
    pub span: Span,
    pub message: String,
    pub style: LabelStyle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

/// Note attached to a diagnostic
#[derive(Debug, Clone)]
pub struct Note {
    pub message: String,
    pub span: Option<Span>,
}

impl Diagnostic {
    pub fn new(severity: Severity, message: String, span: Span) -> Self {
        Self {
            severity,
            message,
            span,
            code: None,
            labels: Vec::new(),
            notes: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn error(message: String, span: Span) -> Self {
        Self::new(Severity::Error, message, span)
    }

    pub fn warning(message: String, span: Span) -> Self {
        Self::new(Severity::Warning, message, span)
    }

    pub fn note(message: String, span: Span) -> Self {
        Self::new(Severity::Note, message, span)
    }

    pub fn help(message: String, span: Span) -> Self {
        Self::new(Severity::Help, message, span)
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.code = Some(code);
        self
    }

    pub fn with_label(mut self, label: Label) -> Self {
        self.labels.push(label);
        self
    }

    pub fn with_note(mut self, note: Note) -> Self {
        self.notes.push(note);
        self
    }

    pub fn with_child(mut self, child: Diagnostic) -> Self {
        self.children.push(child);
        self
    }

    pub fn is_error(&self) -> bool {
        self.severity >= Severity::Error
    }
}

/// Collection of diagnostics
#[derive(Debug, Clone, Default)]
pub struct DiagnosticCollection {
    pub diagnostics: Vec<Diagnostic>,
}

impl DiagnosticCollection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    pub fn extend(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.is_error())
    }

    pub fn errors(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.is_error()).collect()
    }

    pub fn warnings(&self) -> Vec<&Diagnostic> {
        self.diagnostics.iter().filter(|d| d.severity == Severity::Warning).collect()
    }

    pub fn len(&self) -> usize {
        self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Diagnostic emitter for pretty printing
pub struct DiagnosticEmitter<'a> {
    source_map: &'a SourceMap,
    color: bool,
}

impl<'a> DiagnosticEmitter<'a> {
    pub fn new(source_map: &'a SourceMap, color: bool) -> Self {
        Self { source_map, color }
    }

    pub fn emit(&self, diagnostic: &Diagnostic) -> String {
        let mut output = String::new();
        self.emit_diagnostic(&mut output, diagnostic, 0);
        output
    }

    fn emit_diagnostic(&self, output: &mut String, diagnostic: &Diagnostic, indent: usize) {
        let prefix = "  ".repeat(indent);
        let severity_str = self.severity_str(diagnostic.severity);
        let code_str = diagnostic.code.as_ref().map(|c| format!("[{}] ", c)).unwrap_or_default();

        output.push_str(&format!("{} {}: {}{}\n", prefix, severity_str, code_str, diagnostic.message));

        // Show source location
        if let Some(location) = self.source_map.span_to_location(diagnostic.span) {
            if let Some(file) = self.source_map.get_file(location.file) {
                output.push_str(&format!("{}  --> {}:{}:{}\n", prefix, file.path.display(), location.line, location.column));
                self.emit_source_context(output, file, location, diagnostic.span, &prefix);
            }
        }

        // Emit labels
        for label in &diagnostic.labels {
            self.emit_label(output, label, &prefix);
        }

        // Emit notes
        for note in &diagnostic.notes {
            output.push_str(&format!("{}   = note: {}\n", prefix, note.message));
            if let Some(span) = note.span {
                if let Some(location) = self.source_map.span_to_location(span) {
                    if let Some(file) = self.source_map.get_file(location.file) {
                        output.push_str(&format!("{}     --> {}:{}:{}\n", prefix, file.path.display(), location.line, location.column));
                    }
                }
            }
        }

        // Emit children
        for child in &diagnostic.children {
            self.emit_diagnostic(output, child, indent + 1);
        }
    }

    fn severity_str(&self, severity: Severity) -> &str {
        match severity {
            Severity::Bug => "error[internal]",
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }

    fn emit_source_context(&self, output: &mut String, file: &crate::SourceFile, location: crate::Location, span: crate::Span, prefix: &str) {
        let line_idx = location.line - 1;
        if line_idx < file.lines.len() {
            let line_start = file.lines[line_idx];
            let line_end = file.lines.get(line_idx + 1).copied().unwrap_or(file.content.len());
            let line_content = &file.content[line_start..line_end.min(file.content.len())];

            output.push_str(&format!("{}   |\n", prefix));
            output.push_str(&format!("{} {:4} | {}\n", prefix, location.line, line_content.trim_end()));
            output.push_str(&format!("{}   | {}{}^\n", prefix, " ".repeat(location.column - 1), "^".repeat(1.max(span.end - span.start))));
        }
    }

    fn emit_label(&self, output: &mut String, label: &Label, prefix: &str) {
        if let Some(location) = self.source_map.span_to_location(label.span) {
            if let Some(file) = self.source_map.get_file(location.file) {
                output.push_str(&format!("{}   |\n", prefix));
                output.push_str(&format!("{} {:4} | {}\n", prefix, location.line, label.message));
            }
        }
    }
}

/// Render diagnostics to string
pub fn render_diagnostics(source_map: &SourceMap, diagnostics: &DiagnosticCollection, color: bool) -> String {
    let emitter = DiagnosticEmitter::new(source_map, color);
    let mut output = String::new();

    for diagnostic in &diagnostics.diagnostics {
        output.push_str(&emitter.emit(diagnostic));
    }

    // Summary
    let error_count = diagnostics.errors().len();
    let warning_count = diagnostics.warnings().len();

    if error_count > 0 || warning_count > 0 {
        output.push('\n');
        if error_count > 0 {
            output.push_str(&format!("error: {} error{}", error_count, if error_count == 1 { "" } else { "s" }));
        }
        if warning_count > 0 {
            if error_count > 0 {
                output.push_str("; ");
            }
            output.push_str(&format!("warning: {} warning{}", warning_count, if warning_count == 1 { "" } else { "s" }));
        }
        output.push('\n');
    }

    output
}

/// Diagnostic codes
pub mod codes {
    // Lexer errors
    pub const UNTERMINATED_STRING: &str = "E001";
    pub const UNTERMINATED_CHAR: &str = "E002";
    pub const INVALID_ESCAPE: &str = "E003";
    pub const INVALID_NUMBER: &str = "E004";

    // Parser errors
    pub const EXPECTED_ITEM: &str = "E010";
    pub const EXPECTED_EXPRESSION: &str = "E011";
    pub const EXPECTED_TYPE: &str = "E012";
    pub const EXPECTED_PATTERN: &str = "E013";
    pub const EXPECTED_IDENTIFIER: &str = "E014";
    pub const UNEXPECTED_TOKEN: &str = "E015";
    pub const UNCLOSED_DELIMITER: &str = "E016";

    // Name resolution errors
    pub const UNRESOLVED_NAME: &str = "E020";
    pub const AMBIGUOUS_NAME: &str = "E021";
    pub const UNUSED_IMPORT: &str = "W020";
    pub const SHADOWED_NAME: &str = "W021";

    // Type errors
    pub const TYPE_MISMATCH: &str = "E030";
    pub const TYPE_MISMATCH_ARGS: &str = "E031";
    pub const TYPE_MISMATCH_RETURN: &str = "E032";
    pub const MISSING_TYPE_ANNOTATION: &str = "E033";
    pub const RECURSIVE_TYPE: &str = "E034";
    pub const UNSIZED_TYPE: &str = "E035";
    pub const TRAIT_NOT_IMPLEMENTED: &str = "E036";
    pub const AMBIGUOUS_TRAIT: &str = "E037";
    pub const CONFLICTING_IMPL: &str = "E038";
    pub const ORPHAN_RULE: &str = "E039";

    // Borrow checker errors
    pub const USE_AFTER_MOVE: &str = "E040";
    pub const USE_AFTER_FREE: &str = "E041";
    pub const MUTABLE_BORROW_CONFLICT: &str = "E042";
    pub const IMMUTABLE_BORROW_CONFLICT: &str = "E043";
    pub const BORROW_OUTLIVES_OWNER: &str = "E044";
    pub const CANNOT_MOVE_OUT_OF_BORROW: &str = "E045";
    pub const CANNOT_MUTATE_IMMUTABLE: &str = "E046";

    // Lifetime errors
    pub const LIFETIME_MISMATCH: &str = "E050";
    pub const MISSING_LIFETIME: &str = "E051";
    pub const CONFLICTING_LIFETIMES: &str = "E052";

    // Pattern errors
    pub const NON_EXHAUSTIVE_MATCH: &str = "E060";
    pub const UNREACHABLE_PATTERN: &str = "W060";
    pub const BINDING_SHADOWS_OUTER: &str = "W061";

    // Const evaluation errors
    pub const CONST_EVAL_FAILED: &str = "E070";
    pub const NON_CONST_FN_IN_CONST: &str = "E071";

    // Async errors
    pub const AWAIT_OUTSIDE_ASYNC: &str = "E080";
    pub const YIELD_OUTSIDE_GENERATOR: &str = "E081";
    pub const SEND_BOUND_MISSING: &str = "E082";

    // Effect errors
    pub const UNHANDLED_EFFECT: &str = "E090";
    pub const EFFECT_MISMATCH: &str = "E091";

    // Linear type errors
    pub const LINEAR_USED_TWICE: &str = "E100";
    pub const LINEAR_NOT_USED: &str = "E101";
    pub const LINEAR_DROPPED_EARLY: &str = "E102";

    // Codegen errors
    pub const CODEGEN_FAILED: &str = "E110";
    pub const UNSUPPORTED_TARGET: &str = "E111";
    pub const LINK_FAILED: &str = "E112";

    // Internal errors
    pub const INTERNAL_COMPILER_ERROR: &str = "E999";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SourceMap, Span, FileId};

    #[test]
    fn test_diagnostic_creation() {
        let diag = Diagnostic::error("Test error".to_string(), Span::new(FileId(0), 0, 10));
        assert_eq!(diag.severity, Severity::Error);
        assert!(diag.is_error());
    }

    #[test]
    fn test_diagnostic_collection() {
        let mut coll = DiagnosticCollection::new();
        coll.push(Diagnostic::error("Error 1".to_string(), Span::new(FileId(0), 0, 10)));
        coll.push(Diagnostic::warning("Warning 1".to_string(), Span::new(FileId(0), 10, 20)));
        assert!(coll.has_errors());
        assert_eq!(coll.errors().len(), 1);
        assert_eq!(coll.warnings().len(), 1);
    }

    #[test]
    fn test_render_diagnostics() {
        let mut source_map = SourceMap::new();
        let file_id = source_map.add_file("test.nova".into(), "fn main() {\n    let x = 42;\n}".to_string());

        let mut coll = DiagnosticCollection::new();
        coll.push(Diagnostic::error(
            "Expected ;".to_string(),
            Span::new(file_id, 15, 16)
        ).with_code(codes::EXPECTED_EXPRESSION.to_string()));

        let output = render_diagnostics(&source_map, &coll, false);
        assert!(output.contains("error"));
        assert!(output.contains("Expected ;"));
    }
}