//! AST visitor for code formatting

use atlas_runtime::ast::*;

use crate::comments::{Comment, CommentPosition};
use crate::formatter::FormatConfig;

/// AST visitor that produces formatted source code
pub struct FormatVisitor {
    /// Output buffer
    output: String,
    /// Current indentation level
    indent_level: usize,
    /// Formatter configuration
    config: FormatConfig,
    /// Comments to reinsert
    comments: Vec<Comment>,
    /// Index of next comment to consider
    comment_index: usize,
    /// Source text for span lookups
    source: String,
}

impl FormatVisitor {
    pub fn new(config: FormatConfig, comments: Vec<Comment>, source: String) -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            config,
            comments,
            comment_index: 0,
            source,
        }
    }

    pub fn into_output(self) -> String {
        let mut result = self.output;
        // Ensure file ends with a single newline
        if !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        result
    }

    /// Write indentation at current level
    fn write_indent(&mut self) {
        let spaces = " ".repeat(self.indent_level * self.config.indent_size);
        self.output.push_str(&spaces);
    }

    /// Write a string to output
    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    /// Write a newline
    fn writeln(&mut self) {
        self.output.push('\n');
    }

    /// Emit any leading comments before a given span offset
    fn emit_leading_comments(&mut self, before_offset: usize) {
        while self.comment_index < self.comments.len() {
            let start = self.comments[self.comment_index].span.start;
            let pos = self.comments[self.comment_index].position;
            if start >= before_offset {
                break;
            }
            if pos == CommentPosition::Leading || pos == CommentPosition::Standalone {
                let text = self.comments[self.comment_index].text.clone();
                self.write_indent();
                self.write(&text);
                self.writeln();
                self.comment_index += 1;
            } else {
                break;
            }
        }
    }

    /// Emit trailing comment after a statement if present
    fn emit_trailing_comment(&mut self, after_offset: usize) {
        if self.comment_index < self.comments.len() {
            let pos = self.comments[self.comment_index].position;
            let start = self.comments[self.comment_index].span.start;
            if pos == CommentPosition::Trailing && start >= after_offset {
                let text = self.comments[self.comment_index].text.clone();
                self.write(" ");
                self.write(&text);
                self.comment_index += 1;
            }
        }
    }

    /// Emit any remaining comments at end of file
    fn emit_remaining_comments(&mut self) {
        while self.comment_index < self.comments.len() {
            let text = self.comments[self.comment_index].text.clone();
            self.write_indent();
            self.write(&text);
            self.writeln();
            self.comment_index += 1;
        }
    }

    // === Program ===

    pub fn visit_program(&mut self, program: &Program) {
        for (i, item) in program.items.iter().enumerate() {
            if i > 0 {
                // Add blank line between top-level items for readability
                if self.should_add_blank_line_before(
                    item,
                    if i > 0 {
                        Some(&program.items[i - 1])
                    } else {
                        None
                    },
                ) {
                    self.writeln();
                }
            }
            self.visit_item(item);
        }
        self.emit_remaining_comments();
    }

    fn should_add_blank_line_before(&self, item: &Item, prev: Option<&Item>) -> bool {
        match item {
            Item::Function(_) => true,
            Item::TypeAlias(_) => true,
            Item::Import(_) => !matches!(prev, Some(Item::Import(_))),
            _ => matches!(prev, Some(Item::Function(_))),
        }
    }

    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::Function(f) => {
                self.emit_leading_comments(f.span.start);
                self.visit_function_decl(f);
            }
            Item::Statement(s) => {
                self.emit_leading_comments(s.span().start);
                self.visit_statement(s);
            }
            Item::Import(i) => {
                self.emit_leading_comments(i.span.start);
                self.visit_import(i);
            }
            Item::Export(e) => {
                self.emit_leading_comments(e.span.start);
                self.visit_export(e);
            }
            Item::Extern(e) => {
                self.emit_leading_comments(e.span.start);
                self.visit_extern(e);
            }
            Item::TypeAlias(alias) => {
                self.emit_leading_comments(alias.span.start);
                self.visit_type_alias(alias);
            }
            Item::Trait(_) | Item::Impl(_) => {
                // Trait/impl formatting handled in Block 3
            }
        }
    }

    // === Statements ===

    fn visit_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(v) => self.visit_var_decl(v),
            Stmt::FunctionDecl(f) => self.visit_function_decl(f),
            Stmt::Assign(a) => self.visit_assign(a),
            Stmt::CompoundAssign(c) => self.visit_compound_assign(c),
            Stmt::Increment(i) => self.visit_increment(i),
            Stmt::Decrement(d) => self.visit_decrement(d),
            Stmt::If(i) => self.visit_if(i),
            Stmt::While(w) => self.visit_while(w),
            Stmt::For(f) => self.visit_for(f),
            Stmt::ForIn(f) => self.visit_for_in(f),
            Stmt::Return(r) => self.visit_return(r),
            Stmt::Break(span) => {
                self.write_indent();
                self.write("break;");
                self.emit_trailing_comment(span.end);
                self.writeln();
            }
            Stmt::Continue(span) => {
                self.write_indent();
                self.write("continue;");
                self.emit_trailing_comment(span.end);
                self.writeln();
            }
            Stmt::Expr(e) => self.visit_expr_stmt(e),
        }
    }

    fn visit_var_decl(&mut self, v: &VarDecl) {
        self.write_indent();
        if v.mutable {
            self.write("var ");
        } else {
            self.write("let ");
        }
        self.write(&v.name.name);

        if let Some(ref type_ref) = v.type_ref {
            self.write(": ");
            self.visit_type_ref(type_ref);
        }

        self.write(" = ");
        self.visit_expr(&v.init);
        self.write(";");
        self.emit_trailing_comment(v.span.end);
        self.writeln();
    }

    fn visit_type_alias(&mut self, alias: &TypeAliasDecl) {
        self.write_indent();
        self.write("type ");
        self.write(&alias.name.name);

        if !alias.type_params.is_empty() {
            self.write("<");
            for (idx, param) in alias.type_params.iter().enumerate() {
                if idx > 0 {
                    self.write(", ");
                }
                self.write(&param.name);
            }
            self.write(">");
        }

        self.write(" = ");
        self.visit_type_ref(&alias.type_ref);
        self.write(";");
        self.emit_trailing_comment(alias.span.end);
        self.writeln();
    }

    fn visit_assign(&mut self, a: &Assign) {
        self.write_indent();
        self.visit_assign_target(&a.target);
        self.write(" = ");
        self.visit_expr(&a.value);
        self.write(";");
        self.emit_trailing_comment(a.span.end);
        self.writeln();
    }

    fn visit_compound_assign(&mut self, c: &CompoundAssign) {
        self.write_indent();
        self.visit_assign_target(&c.target);
        let op = match c.op {
            CompoundOp::AddAssign => " += ",
            CompoundOp::SubAssign => " -= ",
            CompoundOp::MulAssign => " *= ",
            CompoundOp::DivAssign => " /= ",
            CompoundOp::ModAssign => " %= ",
        };
        self.write(op);
        self.visit_expr(&c.value);
        self.write(";");
        self.emit_trailing_comment(c.span.end);
        self.writeln();
    }

    fn visit_increment(&mut self, i: &IncrementStmt) {
        self.write_indent();
        self.visit_assign_target(&i.target);
        self.write("++;");
        self.emit_trailing_comment(i.span.end);
        self.writeln();
    }

    fn visit_decrement(&mut self, d: &DecrementStmt) {
        self.write_indent();
        self.visit_assign_target(&d.target);
        self.write("--;");
        self.emit_trailing_comment(d.span.end);
        self.writeln();
    }

    fn visit_assign_target(&mut self, target: &AssignTarget) {
        match target {
            AssignTarget::Name(id) => self.write(&id.name),
            AssignTarget::Index { target, index, .. } => {
                self.visit_expr(target);
                self.write("[");
                self.visit_expr(index);
                self.write("]");
            }
        }
    }

    fn visit_if(&mut self, i: &IfStmt) {
        self.write_indent();
        self.write("if (");
        self.visit_expr(&i.cond);
        self.write(") ");
        self.visit_block(&i.then_block);
        if let Some(ref else_block) = i.else_block {
            self.write(" else ");
            // Check if else block is a single if statement (else-if chain)
            if else_block.statements.len() == 1 {
                if let Stmt::If(nested_if) = &else_block.statements[0] {
                    self.write("if (");
                    self.visit_expr(&nested_if.cond);
                    self.write(") ");
                    self.visit_block(&nested_if.then_block);
                    if let Some(ref nested_else) = nested_if.else_block {
                        self.write(" else ");
                        self.visit_block(nested_else);
                    }
                    self.writeln();
                    return;
                }
            }
            self.visit_block(else_block);
        }
        self.writeln();
    }

    fn visit_while(&mut self, w: &WhileStmt) {
        self.write_indent();
        self.write("while (");
        self.visit_expr(&w.cond);
        self.write(") ");
        self.visit_block(&w.body);
        self.writeln();
    }

    fn visit_for(&mut self, f: &ForStmt) {
        self.write_indent();
        self.write("for (");
        self.visit_inline_statement(&f.init);
        self.write("; ");
        self.visit_expr(&f.cond);
        self.write("; ");
        self.visit_inline_statement(&f.step);
        self.write(") ");
        self.visit_block(&f.body);
        self.writeln();
    }

    fn visit_for_in(&mut self, f: &ForInStmt) {
        self.write_indent();
        self.write("for ");
        self.write(&f.variable.name);
        self.write(" in ");
        self.visit_expr(&f.iterable);
        self.write(" ");
        self.visit_block(&f.body);
        self.writeln();
    }

    /// Visit a statement inline (no indent, no trailing newline) - for `for` loop init/step
    fn visit_inline_statement(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::VarDecl(v) => {
                if v.mutable {
                    self.write("var ");
                } else {
                    self.write("let ");
                }
                self.write(&v.name.name);
                if let Some(ref type_ref) = v.type_ref {
                    self.write(": ");
                    self.visit_type_ref(type_ref);
                }
                self.write(" = ");
                self.visit_expr(&v.init);
            }
            Stmt::Assign(a) => {
                self.visit_assign_target(&a.target);
                self.write(" = ");
                self.visit_expr(&a.value);
            }
            Stmt::CompoundAssign(c) => {
                self.visit_assign_target(&c.target);
                let op = match c.op {
                    CompoundOp::AddAssign => " += ",
                    CompoundOp::SubAssign => " -= ",
                    CompoundOp::MulAssign => " *= ",
                    CompoundOp::DivAssign => " /= ",
                    CompoundOp::ModAssign => " %= ",
                };
                self.write(op);
                self.visit_expr(&c.value);
            }
            Stmt::Increment(i) => {
                self.visit_assign_target(&i.target);
                self.write("++");
            }
            Stmt::Decrement(d) => {
                self.visit_assign_target(&d.target);
                self.write("--");
            }
            Stmt::Expr(e) => self.visit_expr(&e.expr),
            _ => {} // Other statement types shouldn't appear in for init/step
        }
    }

    fn visit_return(&mut self, r: &ReturnStmt) {
        self.write_indent();
        self.write("return");
        if let Some(ref value) = r.value {
            self.write(" ");
            self.visit_expr(value);
        }
        self.write(";");
        self.emit_trailing_comment(r.span.end);
        self.writeln();
    }

    fn visit_expr_stmt(&mut self, e: &ExprStmt) {
        self.write_indent();
        self.visit_expr(&e.expr);
        self.write(";");
        self.emit_trailing_comment(e.span.end);
        self.writeln();
    }

    fn visit_function_decl(&mut self, f: &FunctionDecl) {
        self.write_indent();
        self.write("fn ");
        self.write(&f.name.name);

        // Type parameters
        if !f.type_params.is_empty() {
            self.write("<");
            for (i, tp) in f.type_params.iter().enumerate() {
                if i > 0 {
                    self.write(", ");
                }
                self.write(&tp.name);
            }
            self.write(">");
        }

        self.write("(");
        let params_str = self.format_params(&f.params);
        if self.would_exceed_max_width(&params_str) && f.params.len() > 1 {
            self.write_params_multiline(&f.params);
        } else {
            self.write(&params_str);
        }
        self.write(")");

        // Return type
        let type_name = self.type_ref_to_string(&f.return_type);
        if type_name != "null" {
            self.write(" -> ");
            self.write(&type_name);
        }

        self.write(" ");
        self.visit_block(&f.body);
        self.writeln();
    }

    fn format_params(&self, params: &[Param]) -> String {
        params
            .iter()
            .map(|p| {
                let type_name = self.type_ref_to_string(&p.type_ref);
                if type_name == "any" {
                    p.name.name.clone()
                } else {
                    format!("{}: {}", p.name.name, type_name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn write_params_multiline(&mut self, params: &[Param]) {
        self.writeln();
        self.indent_level += 1;
        for (i, p) in params.iter().enumerate() {
            self.write_indent();
            self.write(&p.name.name);
            let type_name = self.type_ref_to_string(&p.type_ref);
            if type_name != "any" {
                self.write(": ");
                self.write(&type_name);
            }
            if i < params.len() - 1 || self.config.trailing_commas {
                self.write(",");
            }
            self.writeln();
        }
        self.indent_level -= 1;
        self.write_indent();
    }

    fn would_exceed_max_width(&self, content: &str) -> bool {
        let current_line_len = self
            .output
            .rfind('\n')
            .map(|pos| self.output.len() - pos - 1)
            .unwrap_or(self.output.len());
        current_line_len + content.len() > self.config.max_width
    }

    // === Block ===

    fn visit_block(&mut self, block: &Block) {
        self.write("{");
        if block.statements.is_empty() {
            self.write("}");
            return;
        }
        self.writeln();
        self.indent_level += 1;
        for stmt in &block.statements {
            self.emit_leading_comments(stmt.span().start);
            self.visit_statement(stmt);
        }
        self.indent_level -= 1;
        self.write_indent();
        self.write("}");
    }

    // === Expressions ===

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit, _) => self.visit_literal(lit),
            Expr::Identifier(id) => self.write(&id.name),
            Expr::Unary(u) => self.visit_unary(u),
            Expr::Binary(b) => self.visit_binary(b),
            Expr::Call(c) => self.visit_call(c),
            Expr::Index(i) => self.visit_index(i),
            Expr::Member(m) => self.visit_member(m),
            Expr::ArrayLiteral(a) => self.visit_array_literal(a),
            Expr::Group(g) => {
                self.write("(");
                self.visit_expr(&g.expr);
                self.write(")");
            }
            Expr::Match(m) => self.visit_match(m),
            Expr::Try(t) => {
                self.visit_expr(&t.expr);
                self.write("?");
            }
        }
    }

    fn visit_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::Number(n) => {
                // Format numbers cleanly
                if *n == (*n as i64) as f64 && !n.is_infinite() && !n.is_nan() {
                    self.write(&format!("{}", *n as i64));
                } else {
                    self.write(&format!("{}", n));
                }
            }
            Literal::String(s) => {
                self.write("\"");
                self.write(&escape_string(s));
                self.write("\"");
            }
            Literal::Bool(b) => self.write(if *b { "true" } else { "false" }),
            Literal::Null => self.write("null"),
        }
    }

    fn visit_unary(&mut self, u: &UnaryExpr) {
        match u.op {
            UnaryOp::Negate => self.write("-"),
            UnaryOp::Not => self.write("!"),
        }
        self.visit_expr(&u.expr);
    }

    fn visit_binary(&mut self, b: &BinaryExpr) {
        self.visit_expr(&b.left);
        let op = match b.op {
            BinaryOp::Add => " + ",
            BinaryOp::Sub => " - ",
            BinaryOp::Mul => " * ",
            BinaryOp::Div => " / ",
            BinaryOp::Mod => " % ",
            BinaryOp::Eq => " == ",
            BinaryOp::Ne => " != ",
            BinaryOp::Lt => " < ",
            BinaryOp::Le => " <= ",
            BinaryOp::Gt => " > ",
            BinaryOp::Ge => " >= ",
            BinaryOp::And => " && ",
            BinaryOp::Or => " || ",
        };
        self.write(op);
        self.visit_expr(&b.right);
    }

    fn visit_call(&mut self, c: &CallExpr) {
        self.visit_expr(&c.callee);
        self.write("(");
        let args_str = self.format_args(&c.args);
        if self.would_exceed_max_width(&args_str) && c.args.len() > 1 {
            self.write_args_multiline(&c.args);
        } else {
            self.write(&args_str);
        }
        self.write(")");
    }

    fn format_args(&self, args: &[Expr]) -> String {
        let mut visitor = FormatVisitor::new(self.config.clone(), Vec::new(), self.source.clone());
        let parts: Vec<String> = args
            .iter()
            .map(|a| {
                visitor.output.clear();
                visitor.visit_expr(a);
                visitor.output.clone()
            })
            .collect();
        parts.join(", ")
    }

    fn write_args_multiline(&mut self, args: &[Expr]) {
        self.writeln();
        self.indent_level += 1;
        for (i, arg) in args.iter().enumerate() {
            self.write_indent();
            self.visit_expr(arg);
            if i < args.len() - 1 || self.config.trailing_commas {
                self.write(",");
            }
            self.writeln();
        }
        self.indent_level -= 1;
        self.write_indent();
    }

    fn visit_index(&mut self, i: &IndexExpr) {
        self.visit_expr(&i.target);
        self.write("[");
        self.visit_expr(&i.index);
        self.write("]");
    }

    fn visit_member(&mut self, m: &MemberExpr) {
        self.visit_expr(&m.target);
        self.write(".");
        self.write(&m.member.name);
        if let Some(ref args) = m.args {
            self.write("(");
            let args_str = self.format_args(args);
            if self.would_exceed_max_width(&args_str) && args.len() > 1 {
                self.write_args_multiline(args);
            } else {
                self.write(&args_str);
            }
            self.write(")");
        }
    }

    fn visit_array_literal(&mut self, a: &ArrayLiteral) {
        if a.elements.is_empty() {
            self.write("[]");
            return;
        }

        let elements_str = self.format_args(&a.elements);
        if self.would_exceed_max_width(&format!("[{}]", elements_str)) && a.elements.len() > 1 {
            self.write("[");
            self.writeln();
            self.indent_level += 1;
            for (i, elem) in a.elements.iter().enumerate() {
                self.write_indent();
                self.visit_expr(elem);
                if i < a.elements.len() - 1 || self.config.trailing_commas {
                    self.write(",");
                }
                self.writeln();
            }
            self.indent_level -= 1;
            self.write_indent();
            self.write("]");
        } else {
            self.write("[");
            self.write(&elements_str);
            self.write("]");
        }
    }

    fn visit_match(&mut self, m: &MatchExpr) {
        self.write("match ");
        self.visit_expr(&m.scrutinee);
        self.write(" {");
        self.writeln();
        self.indent_level += 1;
        for arm in &m.arms {
            self.write_indent();
            self.visit_pattern(&arm.pattern);
            if let Some(guard) = &arm.guard {
                self.write(" if ");
                self.visit_expr(guard);
            }
            self.write(" => ");
            self.visit_expr(&arm.body);
            self.write(",");
            self.writeln();
        }
        self.indent_level -= 1;
        self.write_indent();
        self.write("}");
    }

    fn visit_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Literal(lit, _) => self.visit_literal(lit),
            Pattern::Wildcard(_) => self.write("_"),
            Pattern::Variable(id) => self.write(&id.name),
            Pattern::Constructor { name, args, .. } => {
                self.write(&name.name);
                if !args.is_empty() {
                    self.write("(");
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.visit_pattern(arg);
                    }
                    self.write(")");
                }
            }
            Pattern::Array { elements, .. } => {
                self.write("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.write(", ");
                    }
                    self.visit_pattern(elem);
                }
                self.write("]");
            }
            Pattern::Or(alternatives, _) => {
                for (i, alt) in alternatives.iter().enumerate() {
                    if i > 0 {
                        self.write(" | ");
                    }
                    self.visit_pattern(alt);
                }
            }
        }
    }

    // === Types ===

    fn visit_type_ref(&mut self, type_ref: &TypeRef) {
        let s = self.type_ref_to_string(type_ref);
        self.write(&s);
    }

    fn type_ref_to_string(&self, type_ref: &TypeRef) -> String {
        match type_ref {
            TypeRef::Named(name, _) => name.clone(),
            TypeRef::Array(inner, _) => format!("{}[]", self.type_ref_to_string(inner)),
            TypeRef::Function {
                params,
                return_type,
                ..
            } => {
                let params_str: Vec<String> =
                    params.iter().map(|p| self.type_ref_to_string(p)).collect();
                format!(
                    "fn({}) -> {}",
                    params_str.join(", "),
                    self.type_ref_to_string(return_type)
                )
            }
            TypeRef::Generic {
                name, type_args, ..
            } => {
                let args_str: Vec<String> = type_args
                    .iter()
                    .map(|a| self.type_ref_to_string(a))
                    .collect();
                format!("{}<{}>", name, args_str.join(", "))
            }
            TypeRef::Union { members, .. } => members
                .iter()
                .map(|m| self.type_ref_to_string(m))
                .collect::<Vec<_>>()
                .join(" | "),
            TypeRef::Intersection { members, .. } => members
                .iter()
                .map(|m| self.type_ref_to_string(m))
                .collect::<Vec<_>>()
                .join(" & "),
            TypeRef::Structural { members, .. } => {
                let parts = members
                    .iter()
                    .map(|member| {
                        format!(
                            "{}: {}",
                            member.name,
                            self.type_ref_to_string(&member.type_ref)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{{ {} }}", parts)
            }
        }
    }

    // === Imports/Exports/Extern ===

    fn visit_import(&mut self, import: &ImportDecl) {
        self.write_indent();
        self.write("import ");

        let mut named = Vec::new();
        let mut namespace = None;

        for spec in &import.specifiers {
            match spec {
                ImportSpecifier::Named { name, .. } => named.push(name.name.clone()),
                ImportSpecifier::Namespace { alias, .. } => {
                    namespace = Some(alias.name.clone());
                }
            }
        }

        if let Some(alias) = namespace {
            self.write("* as ");
            self.write(&alias);
        } else if !named.is_empty() {
            self.write("{ ");
            self.write(&named.join(", "));
            self.write(" }");
        }

        self.write(" from \"");
        self.write(&import.source);
        self.write("\";");
        self.emit_trailing_comment(import.span.end);
        self.writeln();
    }

    fn visit_export(&mut self, export: &ExportDecl) {
        self.write_indent();
        self.write("export ");
        match &export.item {
            ExportItem::Function(f) => {
                // Don't add indent since visit_function_decl will
                let indent = self.indent_level;
                self.indent_level = 0;
                // Write without indent prefix
                self.write("fn ");
                self.write(&f.name.name);
                if !f.type_params.is_empty() {
                    self.write("<");
                    for (i, tp) in f.type_params.iter().enumerate() {
                        if i > 0 {
                            self.write(", ");
                        }
                        self.write(&tp.name);
                    }
                    self.write(">");
                }
                self.write("(");
                let params_str = self.format_params(&f.params);
                self.write(&params_str);
                self.write(")");
                let type_name = self.type_ref_to_string(&f.return_type);
                if type_name != "null" {
                    self.write(" -> ");
                    self.write(&type_name);
                }
                self.write(" ");
                self.indent_level = indent;
                self.visit_block(&f.body);
                self.writeln();
            }
            ExportItem::Variable(v) => {
                if v.mutable {
                    self.write("var ");
                } else {
                    self.write("let ");
                }
                self.write(&v.name.name);
                if let Some(ref type_ref) = v.type_ref {
                    self.write(": ");
                    self.visit_type_ref(type_ref);
                }
                self.write(" = ");
                self.visit_expr(&v.init);
                self.write(";");
                self.writeln();
            }
            ExportItem::TypeAlias(alias) => {
                self.visit_type_alias(alias);
            }
        }
    }

    fn visit_extern(&mut self, ext: &ExternDecl) {
        self.write_indent();
        self.write("extern fn ");
        self.write(&ext.name);
        self.write("(");
        for (i, (name, ty)) in ext.params.iter().enumerate() {
            if i > 0 {
                self.write(", ");
            }
            self.write(name);
            self.write(": ");
            self.write(extern_type_str(ty));
        }
        self.write(") -> ");
        self.write(extern_type_str(&ext.return_type));
        self.write(" from \"");
        self.write(&ext.library);
        self.write("\";");
        self.writeln();
    }
}

fn extern_type_str(ty: &ExternTypeAnnotation) -> &'static str {
    match ty {
        ExternTypeAnnotation::CInt => "c_int",
        ExternTypeAnnotation::CLong => "c_long",
        ExternTypeAnnotation::CDouble => "c_double",
        ExternTypeAnnotation::CCharPtr => "c_char_ptr",
        ExternTypeAnnotation::CVoid => "c_void",
        ExternTypeAnnotation::CBool => "c_bool",
    }
}

fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\r' => result.push_str("\\r"),
            c => result.push(c),
        }
    }
    result
}
