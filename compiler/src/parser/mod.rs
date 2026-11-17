//! NOVA Parser - Converts tokens to AST

use std::collections::VecDeque;

use crate::{Session, Span, FileId, NodeId, Token, TokenKind, Diagnostic, Severity};
use crate::ast::*;

/// Parser state
pub struct Parser<'a> {
    session: &'a mut Session,
    tokens: VecDeque<Token>,
    file_id: FileId,
    next_node_id: u32,
}

impl<'a> Parser<'a> {
    pub fn new(session: &'a mut Session, tokens: Vec<Token>, file_id: FileId) -> Self {
        Self {
            session,
            tokens: tokens.into(),
            file_id,
            next_node_id: 1,
        }
    }

    pub fn parse(&mut self) -> Result<Program, Vec<Diagnostic>> {
        let mut items = Vec::new();
        let start_span = self.current_span();

        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.session.diagnostics.push(e);
                    self.synchronize();
                }
            }
        }

        let end_span = self.previous_span();
        Ok(Program {
            items,
            span: Span::new(self.file_id, start_span.start, end_span.end),
        })
    }

    fn next_node_id(&mut self) -> NodeId {
        let id = NodeId(self.next_node_id);
        self.next_node_id += 1;
        id
    }

    // ========================================================================
    // Token stream operations
    // ========================================================================

    fn current(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn peek(&self, n: usize) -> Option<&Token> {
        self.tokens.get(n)
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn is_at_end(&self) -> bool {
        self.current().map(|t| matches!(t.kind, TokenKind::Eof)).unwrap_or(true)
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.current().map(|t| std::mem::discriminant(&t.kind) == std::mem::discriminant(&kind)).unwrap_or(false)
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind, msg: &str) -> Result<Token, Diagnostic> {
        if self.check(kind) {
            Ok(self.advance().unwrap())
        } else {
            let found = self.current().map(|t| format!("{:?}", t.kind)).unwrap_or("EOF".to_string());
            Err(self.error(format!("Expected {:?}, found {}", kind, found)))
        }
    }

    fn current_span(&self) -> Span {
        self.current().map(|t| t.span).unwrap_or(Span::new(self.file_id, 0, 0))
    }

    fn previous_span(&self) -> Span {
        // In a real implementation, track previous token
        self.current_span()
    }

    fn error(&self, msg: String) -> Diagnostic {
        Diagnostic::new(Severity::Error, msg, self.current_span())
    }

    fn synchronize(&mut self) {
        // Skip tokens until we find a synchronization point
        while !self.is_at_end() {
            if matches!(self.current(), Some(t) if matches!(t.kind, TokenKind::Semi)) {
                self.advance();
                return;
            }
            match self.current().map(|t| &t.kind) {
                Some(TokenKind::Fn) | Some(TokenKind::Struct) | Some(TokenKind::Enum) |
                Some(TokenKind::Mod) | Some(TokenKind::Const) | Some(TokenKind::Static) |
                Some(TokenKind::Trait) | Some(TokenKind::Impl) | Some(TokenKind::Use) |
                Some(TokenKind::Type) | Some(TokenKind::Macro) | Some(TokenKind::Pub) => return,
                _ => { self.advance(); }
            }
        }
    }

    // ========================================================================
    // Item parsing
    // ========================================================================

    fn parse_item(&mut self) -> Result<Item, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let visibility = self.parse_visibility()?;

        if self.match_token(TokenKind::Use) {
            return self.parse_use_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Mod) {
            return self.parse_module_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Extern) {
            return self.parse_extern_crate(attrs);
        }
        if self.match_token(TokenKind::Fn) {
            return self.parse_function_decl(attrs, visibility, Constness::NotConst, Asyncness::NotAsync, Unsafety::NotUnsafe);
        }
        if self.match_token(TokenKind::Struct) {
            return self.parse_struct_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Enum) {
            return self.parse_enum_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Type) {
            return self.parse_type_alias(attrs, visibility);
        }
        if self.match_token(TokenKind::Trait) {
            return self.parse_trait_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Impl) {
            return self.parse_impl_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Const) {
            return self.parse_const_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Static) {
            return self.parse_static_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Macro) {
            return self.parse_macro_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::MacroRules) {
            return self.parse_macro_rules_decl(attrs, visibility);
        }
        // NOVA Unique Features - Items
        if self.match_token(TokenKind::Effect) {
            return self.parse_effect_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Handle) {
            return self.parse_effect_handler_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Actor) {
            return self.parse_actor_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Comptime) {
            return self.parse_comptime_decl(attrs, visibility);
        }
        if self.match_token(TokenKind::Pipeline) {
            return self.parse_pipeline_decl(attrs, visibility);
        }

        // Check for modifiers before fn
        let mut constness = Constness::NotConst;
        let mut asyncness = Asyncness::NotAsync;
        let mut unsafety = Unsafety::NotUnsafe;

        if self.match_token(TokenKind::Const) {
            constness = Constness::Const;
        }
        if self.match_token(TokenKind::Async) {
            asyncness = Asyncness::Async;
        }
        if self.match_token(TokenKind::Unsafe) {
            unsafety = Unsafety::Unsafe;
        }

        if self.match_token(TokenKind::Fn) {
            return self.parse_function_decl(attrs, visibility, constness, asyncness, unsafety);
        }

        // If we get here, it's an error
        Err(self.error("Expected item".to_string()))
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, Diagnostic> {
        let mut attrs = Vec::new();
        while self.match_token(TokenKind::Hash) {
            self.expect(TokenKind::LBracket, "Expected [ after #")?;
            let attr = self.parse_attribute()?;
            self.expect(TokenKind::RBracket, "Expected ]")?;
            attrs.push(attr);
        }
        Ok(attrs)
    }

    fn parse_attribute(&mut self) -> Result<Attribute, Diagnostic> {
        let path = self.parse_path()?;
        let args = if self.match_token(TokenKind::LParen) {
            let mut args = Vec::new();
            if !self.check(TokenKind::RBracket) && !self.check(TokenKind::RBrace) {
                loop {
                    args.push(self.parse_meta_item()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            Some(args)
        } else {
            None
        };
        Ok(Attribute {
            style: AttrStyle::Outer,
            path,
            args,
            span: path.span,
        })
    }

    fn parse_meta_item(&mut self) -> Result<MetaItem, Diagnostic> {
        if self.check(TokenKind::Ident("")) {
            // Could be path or name-value
            let path = self.parse_path()?;
            if self.match_token(TokenKind::Assign) {
                let value = self.parse_expr()?;
                Ok(MetaItem::NameValue(path, value))
            } else {
                Ok(MetaItem::Path(path))
            }
        } else if self.check(TokenKind::Literal(String::new())) {
            // Literal
            let token = self.advance().unwrap();
            let lit = self.token_to_literal(token)?;
            Ok(MetaItem::Literal(lit))
        } else {
            // Path with list
            let path = self.parse_path()?;
            self.expect(TokenKind::LParen, "Expected (")?;
            let mut items = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    items.push(self.parse_meta_item()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            Ok(MetaItem::List(path, items))
        }
    }

    fn parse_visibility(&mut self) -> Result<Visibility, Diagnostic> {
        if self.match_token(TokenKind::Pub) {
            if self.match_token(TokenKind::LParen) {
                let vis = if self.match_token(TokenKind::Crate) {
                    Visibility::Crate
                } else if self.match_token(TokenKind::Super) {
                    Visibility::Super
                } else if self.match_token(TokenKind::SelfValue) {
                    Visibility::Self_
                } else if self.check(TokenKind::Ident("")) {
                    let path = self.parse_path()?;
                    Visibility::In(path)
                } else {
                    Visibility::Public
                };
                self.expect(TokenKind::RParen, "Expected )")?;
                Ok(vis)
            } else {
                Ok(Visibility::Public)
            }
        } else {
            Ok(Visibility::Private)
        }
    }

    // ... Continue with parsing functions for each item type ...

    fn parse_use_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let tree = self.parse_use_tree()?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Item::Use(UseDecl { tree, span: self.previous_span() }))
    }

    fn parse_use_tree(&mut self) -> Result<UseTree, Diagnostic> {
        if self.match_token(TokenKind::SelfValue) {
            return Ok(UseTree::Self_);
        }
        if self.match_token(TokenKind::Super) {
            return Ok(UseTree::Super);
        }
        if self.match_token(TokenKind::Crate) {
            return Ok(UseTree::Crate);
        }

        let path = self.parse_path()?;

        if self.match_token(TokenKind::ColonColon) {
            if self.match_token(TokenKind::LBrace) {
                let mut trees = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        trees.push(self.parse_use_tree()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBrace, "Expected }")?;
                return Ok(UseTree::Group(path, trees));
            } else if self.match_token(TokenKind::Star) {
                return Ok(UseTree::Glob(path));
            }
        }

        Ok(UseTree::Simple(path))
    }

    fn parse_module_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let body = if self.match_token(TokenKind::Semi) {
            None
        } else {
            self.expect(TokenKind::LBrace, "Expected {")?;
            let mut items = Vec::new();
            while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                match self.parse_item() {
                    Ok(item) => items.push(item),
                    Err(e) => {
                        self.session.diagnostics.push(e);
                        self.synchronize();
                    }
                }
            }
            self.expect(TokenKind::RBrace, "Expected }")?;
            Some(items)
        };
        Ok(Item::Module(ModuleDecl { visibility, name, body, span: self.previous_span() }))
    }

    fn parse_extern_crate(&mut self, attrs: Vec<Attribute>) -> Result<Item, Diagnostic> {
        self.expect(TokenKind::Crate, "Expected crate")?;
        let name = self.parse_ident()?;
        let alias = if self.match_token(TokenKind::As) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Item::ExternCrate(ExternCrateDecl { name, alias, span: self.previous_span() }))
    }

    fn parse_function_decl(
        &mut self,
        attrs: Vec<Attribute>,
        visibility: Visibility,
        constness: Constness,
        asyncness: Asyncness,
        unsafety: Unsafety,
    ) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(TokenKind::LParen, "Expected (")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                params.push(self.parse_param()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "Expected )")?;

        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let where_clause = self.parse_where_clause()?;

        let body = if self.match_token(TokenKind::Semi) {
            None
        } else {
            Some(self.parse_block()?)
        };

        Ok(Item::Function(FunctionDecl {
            attrs,
            visibility,
            constness,
            asyncness,
            unsafety,
            name,
            generics,
            params,
            return_type,
            where_clause,
            body,
            span: self.previous_span(),
        }))
    }

    fn parse_param(&mut self) -> Result<Param, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let mutability = if self.match_token(TokenKind::Mut) {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        };
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        Ok(Param { attrs, mutability, name, ty, span: self.previous_span() })
    }

    fn parse_struct_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;

        let kind = if self.match_token(TokenKind::LBrace) {
            let mut fields = Vec::new();
            if !self.check(TokenKind::RBrace) {
                loop {
                    fields.push(self.parse_struct_field()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBrace, "Expected }")?;
            StructKind::Record(fields)
        } else if self.match_token(TokenKind::LParen) {
            let mut types = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            StructKind::Tuple(types)
        } else {
            self.expect(TokenKind::Semi, "Expected ;")?;
            StructKind::Unit
        };

        Ok(Item::Struct(StructDecl {
            attrs,
            visibility,
            name,
            generics,
            where_clause,
            kind,
            span: self.previous_span(),
        }))
    }

    fn parse_struct_field(&mut self) -> Result<StructField, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let visibility = self.parse_visibility()?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        Ok(StructField { attrs, visibility, name, ty, span: self.previous_span() })
    }

    fn parse_enum_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;

        let mut variants = Vec::new();
        if !self.check(TokenKind::RBrace) {
            loop {
                variants.push(self.parse_enum_variant()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RBrace, "Expected }")?;

        Ok(Item::Enum(EnumDecl {
            attrs,
            visibility,
            name,
            generics,
            where_clause,
            variants,
            span: self.previous_span(),
        }))
    }

    fn parse_enum_variant(&mut self) -> Result<EnumVariant, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let name = self.parse_ident()?;

        let kind = if self.match_token(TokenKind::LParen) {
            let mut types = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            VariantKind::Tuple(types)
        } else if self.match_token(TokenKind::LBrace) {
            let mut fields = Vec::new();
            if !self.check(TokenKind::RBrace) {
                loop {
                    fields.push(self.parse_struct_field()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBrace, "Expected }")?;
            VariantKind::Record(fields)
        } else {
            VariantKind::Unit
        };

        Ok(EnumVariant { attrs, name, kind, span: self.previous_span() })
    }

    fn parse_type_alias(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        self.expect(TokenKind::Assign, "Expected =")?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Item::TypeAlias(TypeAliasDecl {
            attrs,
            visibility,
            name,
            generics,
            ty,
            span: self.previous_span(),
        }))
    }

    fn parse_trait_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let unsafety = if self.match_token(TokenKind::Unsafe) {
            Unsafety::Unsafe
        } else {
            Unsafety::NotUnsafe
        };
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;

        let mut items = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            items.push(self.parse_trait_item()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;

        Ok(Item::Trait(TraitDecl {
            attrs,
            visibility,
            unsafety,
            name,
            generics,
            where_clause,
            items,
            span: self.previous_span(),
        }))
    }

    fn parse_trait_item(&mut self) -> Result<TraitItem, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let visibility = self.parse_visibility()?;

        if self.match_token(TokenKind::Fn) {
            let constness = if self.match_token(TokenKind::Const) { Constness::Const } else { Constness::NotConst };
            let asyncness = if self.match_token(TokenKind::Async) { Asyncness::Async } else { Asyncness::NotAsync };
            let name = self.parse_ident()?;
            let generics = self.parse_generics()?;
            self.expect(TokenKind::LParen, "Expected (")?;
            let mut params = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    params.push(self.parse_param()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            let return_type = if self.match_token(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let where_clause = self.parse_where_clause()?;
            let body = if self.match_token(TokenKind::Semi) {
                None
            } else {
                Some(self.parse_block()?)
            };
            Ok(TraitItem::Function(TraitFn {
                attrs,
                visibility,
                constness,
                asyncness,
                name,
                generics,
                params,
                return_type,
                where_clause,
                body,
                span: self.previous_span(),
            }))
        } else if self.match_token(TokenKind::Type) {
            let name = self.parse_ident()?;
            let generics = self.parse_generics()?;
            let bounds = if self.match_token(TokenKind::Colon) {
                Some(self.parse_generic_bounds()?)
            } else {
                None
            };
            let default = if self.match_token(TokenKind::Assign) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(TokenKind::Semi, "Expected ;")?;
            Ok(TraitItem::Type(TraitType {
                attrs,
                visibility,
                name,
                generics,
                bounds,
                default,
                span: self.previous_span(),
            }))
        } else if self.match_token(TokenKind::Const) {
            let name = self.parse_ident()?;
            self.expect(TokenKind::Colon, "Expected :")?;
            let ty = self.parse_type()?;
            let default = if self.match_token(TokenKind::Assign) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            self.expect(TokenKind::Semi, "Expected ;")?;
            Ok(TraitItem::Const(TraitConst {
                attrs,
                visibility,
                name,
                ty,
                default,
                span: self.previous_span(),
            }))
        } else {
            Err(self.error("Expected trait item".to_string()))
        }
    }

    fn parse_impl_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let unsafety = if self.match_token(TokenKind::Unsafe) {
            Unsafety::Unsafe
        } else {
            Unsafety::NotUnsafe
        };
        let generics = self.parse_generics()?;

        let trait_ref = if self.check(TokenKind::Ident("")) && !self.check_keyword("for") {
            // Could be trait ref followed by "for"
            let checkpoint = self.tokens.len();
            let path = self.parse_path()?;
            let generics = self.parse_generic_args()?;
            if self.match_token(TokenKind::Ident("for")) {
                // This is a trait impl
                let trait_ref = TraitRef { path, generics, span: self.previous_span() };
                let self_ty = self.parse_type()?;
                let where_clause = self.parse_where_clause()?;
                self.expect(TokenKind::LBrace, "Expected {")?;
                let mut items = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                    items.push(self.parse_impl_item()?);
                }
                self.expect(TokenKind::RBrace, "Expected }")?;
                return Ok(Item::Impl(ImplDecl {
                    attrs,
                    visibility,
                    unsafety,
                    generics,
                    trait_ref: Some(trait_ref),
                    self_ty,
                    where_clause,
                    items,
                    span: self.previous_span(),
                }));
            } else {
                // Not a trait impl, backtrack
                self.tokens.truncate(checkpoint);
            }
        } else {
            None
        };

        // Inherent impl
        let self_ty = self.parse_type()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut items = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            items.push(self.parse_impl_item()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;

        Ok(Item::Impl(ImplDecl {
            attrs,
            visibility,
            unsafety,
            generics,
            trait_ref,
            self_ty,
            where_clause,
            items,
            span: self.previous_span(),
        }))
    }

    fn parse_impl_item(&mut self) -> Result<ImplItem, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let visibility = self.parse_visibility()?;

        if self.match_token(TokenKind::Fn) {
            let constness = if self.match_token(TokenKind::Const) { Constness::Const } else { Constness::NotConst };
            let asyncness = if self.match_token(TokenKind::Async) { Asyncness::Async } else { Asyncness::NotAsync };
            let unsafety = if self.match_token(TokenKind::Unsafe) { Unsafety::Unsafe } else { Unsafety::NotUnsafe };
            let name = self.parse_ident()?;
            let generics = self.parse_generics()?;
            self.expect(TokenKind::LParen, "Expected (")?;
            let mut params = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    params.push(self.parse_param()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            let return_type = if self.match_token(TokenKind::Arrow) {
                Some(self.parse_type()?)
            } else {
                None
            };
            let where_clause = self.parse_where_clause()?;
            let body = self.parse_block()?;
            Ok(ImplItem::Function(FunctionDecl {
                attrs,
                visibility,
                constness,
                asyncness,
                unsafety,
                name,
                generics,
                params,
                return_type,
                where_clause,
                body: Some(body),
                span: self.previous_span(),
            }))
        } else if self.match_token(TokenKind::Type) {
            let name = self.parse_ident()?;
            self.expect(TokenKind::Assign, "Expected =")?;
            let ty = self.parse_type()?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            Ok(ImplItem::Type(TypeAliasDecl {
                attrs,
                visibility,
                name,
                generics: Generics { params: Vec::new(), span: self.previous_span() },
                ty,
                span: self.previous_span(),
            }))
        } else if self.match_token(TokenKind::Const) {
            let name = self.parse_ident()?;
            self.expect(TokenKind::Colon, "Expected :")?;
            let ty = self.parse_type()?;
            self.expect(TokenKind::Assign, "Expected =")?;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            Ok(ImplItem::Const(ConstDecl {
                attrs,
                visibility,
                name,
                ty,
                value,
                span: self.previous_span(),
            }))
        } else {
            Err(self.error("Expected impl item".to_string()))
        }
    }

    fn parse_const_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Assign, "Expected =")?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Item::Const(ConstDecl {
            attrs,
            visibility,
            name,
            ty,
            value,
            span: self.previous_span(),
        }))
    }

    fn parse_static_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let mutability = if self.match_token(TokenKind::Mut) {
            Mutability::Mutable
        } else {
            Mutability::Immutable
        };
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Assign, "Expected =")?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Item::Static(StaticDecl {
            attrs,
            visibility,
            mutability,
            name,
            ty,
            value,
            span: self.previous_span(),
        }))
    }

    fn parse_macro_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        self.expect(TokenKind::LParen, "Expected (")?;
        let mut params = Vec::new();
        if !self.check(TokenKind::RParen) {
            loop {
                params.push(self.parse_macro_param()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RParen, "Expected )")?;
        self.expect(TokenKind::FatArrow, "Expected =>")?;
        let rules = self.parse_macro_rules()?;
        Ok(Item::Macro(MacroDecl {
            attrs,
            visibility,
            name,
            params,
            rules,
            span: self.previous_span(),
        }))
    }

    fn parse_macro_rules_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        self.expect(TokenKind::Bang, "Expected !")?;
        let name = self.parse_ident()?;
        let rules = self.parse_macro_rules()?;
        Ok(Item::MacroRules(MacroRulesDecl {
            attrs,
            visibility,
            name,
            rules,
            span: self.previous_span(),
        }))
    }

    // ========================================================================
    // NOVA Unique Features - Item Parsing
    // ========================================================================

    fn parse_effect_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut operations = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            operations.push(self.parse_effect_operation()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        let span = self.previous_span();
        Ok(Item::Effect(EffectDecl {
            attrs,
            visibility,
            name,
            generics,
            operations,
            span,
        }))
    }

    fn parse_effect_operation(&mut self) -> Result<EffectOperation, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::LParen, "Expected (")?;
        let params = self.parse_fn_params()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(EffectOperation {
            attrs,
            name,
            params,
            return_type,
            span: self.previous_span(),
        })
    }

    fn parse_effect_handler_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let effect = self.parse_path()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut handlers = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            handlers.push(self.parse_effect_handler_arm()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        let span = self.previous_span();
        Ok(Item::EffectHandler(EffectHandlerDecl {
            attrs,
            visibility,
            effect,
            generics,
            handlers,
            span,
        }))
    }

    fn parse_effect_handler_arm(&mut self) -> Result<EffectHandlerArm, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let operation = self.parse_ident()?;
        self.expect(TokenKind::LParen, "Expected (")?;
        let params = self.parse_fn_params()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        let body = self.parse_block()?;
        Ok(EffectHandlerArm {
            attrs,
            operation,
            params,
            body,
            span: self.previous_span(),
        })
    }

    fn parse_actor_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut supervision = None;
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if self.check(TokenKind::Pub) || self.check(TokenKind::Ident("pub".to_string())) {
                // Could be field or method
                if self.peek(1).map(|t| matches!(t.kind, TokenKind::Ident(_))).unwrap_or(false) {
                    let next = self.peek(2);
                    if next.map(|t| matches!(t.kind, TokenKind::Colon)).unwrap_or(false) {
                        fields.push(self.parse_struct_field()?);
                        continue;
                    }
                }
            }
            if self.match_token(TokenKind::Ident("supervise".to_string())) {
                supervision = Some(self.parse_supervision_spec()?);
                continue;
            }
            methods.push(self.parse_function_decl(Vec::new(), visibility, Constness::NotConst, Asyncness::NotAsync, Unsafety::NotUnsafe)?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        let span = self.previous_span();
        Ok(Item::Actor(ActorDecl {
            attrs,
            visibility,
            name,
            generics,
            fields,
            methods,
            supervision,
            span,
        }))
    }

    fn parse_supervision_spec(&mut self) -> Result<SupervisionSpec, Diagnostic> {
        self.expect(TokenKind::LParen, "Expected (")?;
        let strategy = self.parse_supervision_strategy()?;
        let mut max_restarts = None;
        let mut time_window = None;
        while self.match_token(TokenKind::Comma) {
            if self.match_token(TokenKind::Ident("max_restarts".to_string())) {
                self.expect(TokenKind::Assign, "Expected =")?;
                let token = self.advance().ok_or_else(|| self.error("Expected integer".to_string()))?;
                if let TokenKind::Integer(s) = token.kind {
                    max_restarts = Some(s.parse().unwrap_or(0));
                }
            } else if self.match_token(TokenKind::Ident("time_window".to_string())) {
                self.expect(TokenKind::Assign, "Expected =")?;
                time_window = Some(self.parse_duration_expr()?);
            }
        }
        self.expect(TokenKind::RParen, "Expected )")?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(SupervisionSpec {
            strategy,
            max_restarts,
            time_window,
            span: self.previous_span(),
        })
    }

    fn parse_supervision_strategy(&mut self) -> Result<SupervisionStrategy, Diagnostic> {
        let token = self.advance().ok_or_else(|| self.error("Expected supervision strategy".to_string()))?;
        match token.kind {
            TokenKind::Ident(s) => Ok(match s.as_str() {
                "one_for_one" => SupervisionStrategy::OneForOne,
                "one_for_all" => SupervisionStrategy::OneForAll,
                "rest_for_one" => SupervisionStrategy::RestForOne,
                _ => SupervisionStrategy::OneForOne,
            }),
            _ => Err(self.error("Expected supervision strategy".to_string())),
        }
    }

    fn parse_duration_expr(&mut self) -> Result<DurationExpr, Diagnostic> {
        let token = self.advance().ok_or_else(|| self.error("Expected duration".to_string()))?;
        let value = if let TokenKind::Integer(s) = token.kind {
            s.parse().unwrap_or(0)
        } else {
            0
        };
        let unit_token = self.advance().ok_or_else(|| self.error("Expected time unit".to_string()))?;
        let unit = match unit_token.kind {
            TokenKind::Ident(s) => match s.as_str() {
                "ns" => TimeUnit::Nanoseconds,
                "us" => TimeUnit::Microseconds,
                "ms" => TimeUnit::Milliseconds,
                "s" => TimeUnit::Seconds,
                "m" => TimeUnit::Minutes,
                "h" => TimeUnit::Hours,
                _ => TimeUnit::Milliseconds,
            },
            _ => TimeUnit::Milliseconds,
        };
        Ok(DurationExpr {
            value,
            unit,
            span: self.previous_span(),
        })
    }

    fn parse_struct_field(&mut self) -> Result<StructField, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let visibility = self.parse_visibility()?;
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let ty = self.parse_type_expr()?;
        let default = if self.match_token(TokenKind::Assign) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(StructField {
            attrs,
            visibility,
            name,
            ty,
            default,
            span: self.previous_span(),
        })
    }

    fn parse_comptime_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let span_start = self.current_span();
        let kind = if self.check(TokenKind::LBrace) {
            let block = self.parse_block()?;
            ComptimeKind::Block(block)
        } else if self.check(TokenKind::Fn) {
            let func = self.parse_function_decl(attrs.clone(), visibility, Constness::NotConst, Asyncness::NotAsync, Unsafety::NotUnsafe)?;
            if let Item::Function(f) = func {
                ComptimeKind::Function(f)
            } else {
                ComptimeKind::Block(Block { stmts: Vec::new(), expr: None, span: span_start, labels: Vec::new() })
            }
        } else if self.check(TokenKind::Const) {
            let ident = self.parse_ident()?;
            self.expect(TokenKind::Colon, "Expected :")?;
            let ty = self.parse_type_expr()?;
            self.expect(TokenKind::Assign, "Expected =")?;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            ComptimeKind::Const(ident, ty, value)
        } else if self.check(TokenKind::Static) {
            let ident = self.parse_ident()?;
            self.expect(TokenKind::Colon, "Expected :")?;
            let ty = self.parse_type_expr()?;
            self.expect(TokenKind::Assign, "Expected =")?;
            let value = self.parse_expr()?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            ComptimeKind::Static(ident, ty, value)
        } else {
            let block = self.parse_block()?;
            ComptimeKind::Block(block)
        };
        Ok(Item::Comptime(ComptimeDecl {
            attrs,
            visibility,
            kind,
            span: self.previous_span(),
        }))
    }

    fn parse_pipeline_decl(&mut self, attrs: Vec<Attribute>, visibility: Visibility) -> Result<Item, Diagnostic> {
        let name = self.parse_ident()?;
        let generics = self.parse_generics()?;
        let where_clause = self.parse_where_clause()?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut stages = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            stages.push(self.parse_pipeline_stage()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        let span = self.previous_span();
        Ok(Item::Pipeline(PipelineDecl {
            attrs,
            visibility,
            name,
            generics,
            stages,
            span,
        }))
    }

    fn parse_pipeline_stage(&mut self) -> Result<PipelineStage, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let kind = if self.match_token(TokenKind::Ident("filter".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Filter(expr)
        } else if self.match_token(TokenKind::Ident("map".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Map(expr)
        } else if self.match_token(TokenKind::Ident("flat_map".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::FlatMap(expr)
        } else if self.match_token(TokenKind::Ident("fold".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let init = self.parse_expr()?;
            self.expect(TokenKind::Comma, "Expected ,")?;
            let func = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Fold(init, func)
        } else if self.match_token(TokenKind::Ident("reduce".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let func = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Reduce(func)
        } else if self.match_token(TokenKind::Ident("collect".to_string())) {
            let ty = if self.match_token(TokenKind::LParen) {
                if self.check(TokenKind::RBracket) || self.check(TokenKind::Ident("".to_string())) {
                    let ty = self.parse_type_expr()?;
                    self.expect(TokenKind::RParen, "Expected )")?;
                    Some(ty)
                } else {
                    self.expect(TokenKind::RParen, "Expected )")?;
                    None
                }
            } else {
                None
            };
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Collect(ty)
        } else if self.match_token(TokenKind::Ident("take".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Take(expr)
        } else if self.match_token(TokenKind::Ident("skip".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Skip(expr)
        } else {
            let ident = self.parse_ident()?;
            self.expect(TokenKind::LParen, "Expected (")?;
            let args = self.parse_expr_list()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            self.expect(TokenKind::Semi, "Expected ;")?;
            PipelineStageKind::Custom(ident, args)
        };
        Ok(PipelineStage {
            attrs,
            kind,
            span: self.previous_span(),
        })
    }

    fn parse_macro_param(&mut self) -> Result<MacroParam, Diagnostic> {
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let fragment = self.parse_macro_fragment_spec()?;
        Ok(MacroParam { name, fragment, span: self.previous_span() })
    }

    fn parse_macro_fragment_spec(&mut self) -> Result<MacroFragmentSpec, Diagnostic> {
        let token = self.advance().ok_or_else(|| self.error("Expected fragment specifier".to_string()))?;
        match token.kind {
            TokenKind::Ident(s) => Ok(match s.as_str() {
                "expr" => MacroFragmentSpec::Expr,
                "stmt" => MacroFragmentSpec::Stmt,
                "item" => MacroFragmentSpec::Item,
                "type" | "ty" => MacroFragmentSpec::Type,
                "pat" => MacroFragmentSpec::Pat,
                "path" => MacroFragmentSpec::Path,
                "ident" => MacroFragmentSpec::Ident,
                "literal" => MacroFragmentSpec::Literal,
                "block" => MacroFragmentSpec::Block,
                "meta" => MacroFragmentSpec::Meta,
                "tt" => MacroFragmentSpec::Tt,
                "vis" => MacroFragmentSpec::Vis,
                _ => MacroFragmentSpec::Expr,
            }),
            _ => Err(self.error("Expected fragment specifier".to_string())),
        }
    }

    fn parse_macro_rules(&mut self) -> Result<MacroRules, Diagnostic> {
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut rules = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            rules.push(self.parse_macro_rule()?);
            self.match_token(TokenKind::Semi);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        Ok(MacroRules { rules, span: self.previous_span() })
    }

    fn parse_macro_rule(&mut self) -> Result<MacroRule, Diagnostic> {
        self.expect(TokenKind::LParen, "Expected (")?;
        let pattern = self.parse_token_tree_sequence()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        self.expect(TokenKind::FatArrow, "Expected =>")?;
        self.expect(TokenKind::LParen, "Expected (")?;
        let body = self.parse_token_tree_sequence()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        Ok(MacroRule { pattern, body, span: self.previous_span() })
    }

    fn parse_token_tree_sequence(&mut self) -> Result<Vec<TokenTree>, Diagnostic> {
        let mut trees = Vec::new();
        while !self.check(TokenKind::RParen) && !self.check(TokenKind::RBracket) && !self.check(TokenKind::RBrace) && !self.is_at_end() {
            trees.push(self.parse_token_tree()?);
        }
        Ok(trees)
    }

    fn parse_token_tree(&mut self) -> Result<TokenTree, Diagnostic> {
        // Simplified - in reality this would parse full token trees
        let token = self.advance().ok_or_else(|| self.error("Unexpected EOF".to_string()))?;
        match token.kind {
            TokenKind::Ident(s) => Ok(TokenTree::Ident(Ident { name: s, raw: false, span: token.span })),
            TokenKind::Integer(s) => Ok(TokenTree::Literal(Literal { kind: LiteralKind::Integer(s), suffix: None, span: token.span })),
            TokenKind::Float(s) => Ok(TokenTree::Literal(Literal { kind: LiteralKind::Float(s), suffix: None, span: token.span })),
            TokenKind::Char(c) => Ok(TokenTree::Literal(Literal { kind: LiteralKind::Char(c), suffix: None, span: token.span })),
            TokenKind::String(s) => Ok(TokenTree::Literal(Literal { kind: LiteralKind::String(s), suffix: None, span: token.span })),
            TokenKind::Bool(b) => Ok(TokenTree::Literal(Literal { kind: LiteralKind::Bool(b), suffix: None, span: token.span })),
            TokenKind::Unit => Ok(TokenTree::Literal(Literal { kind: LiteralKind::Unit, suffix: None, span: token.span })),
            TokenKind::LParen => {
                let mut trees = Vec::new();
                while !self.check(TokenKind::RParen) && !self.is_at_end() {
                    trees.push(self.parse_token_tree()?);
                }
                self.expect(TokenKind::RParen, "Expected )")?;
                Ok(TokenTree::Group(Delimiter::Paren, trees))
            }
            TokenKind::LBracket => {
                let mut trees = Vec::new();
                while !self.check(TokenKind::RBracket) && !self.is_at_end() {
                    trees.push(self.parse_token_tree()?);
                }
                self.expect(TokenKind::RBracket, "Expected ]")?;
                Ok(TokenTree::Group(Delimiter::Bracket, trees))
            }
            TokenKind::LBrace => {
                let mut trees = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                    trees.push(self.parse_token_tree()?);
                }
                self.expect(TokenKind::RBrace, "Expected }")?;
                Ok(TokenTree::Group(Delimiter::Brace, trees))
            }
            _ => Ok(TokenTree::Punct('?')),
        }
    }

    // ========================================================================
    // Type parsing
    // ========================================================================

    fn parse_type(&mut self) -> Result<TypeExpr, Diagnostic> {
        let start = self.current_span();
        let kind = self.parse_type_inner()?;
        Ok(TypeExpr { kind, span: Span::new(self.file_id, start.start, self.previous_span().end) })
    }

    fn parse_type_inner(&mut self) -> Result<TypeExprKind, Diagnostic> {
        if self.match_token(TokenKind::LParen) {
            if self.check(TokenKind::RParen) {
                self.expect(TokenKind::RParen, "Expected )")?;
                return Ok(TypeExprKind::Tuple(Vec::new()));
            }
            let first = self.parse_type()?;
            if self.match_token(TokenKind::Comma) {
                let mut types = vec![first];
                loop {
                    types.push(self.parse_type()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen, "Expected )")?;
                Ok(TypeExprKind::Tuple(types))
            } else {
                self.expect(TokenKind::RParen, "Expected )")?;
                Ok(TypeExprKind::Paren(Box::new(first)))
            }
        } else if self.match_token(TokenKind::LBracket) {
            let elem = self.parse_type()?;
            if self.match_token(TokenKind::Semi) {
                let len = self.parse_expr()?;
                self.expect(TokenKind::RBracket, "Expected ]")?;
                Ok(TypeExprKind::Array(Box::new(elem), Box::new(len)))
            } else {
                self.expect(TokenKind::RBracket, "Expected ]")?;
                Ok(TypeExprKind::Slice(Box::new(elem)))
            }
        } else if self.match_token(TokenKind::Amp) {
            let mutability = if self.match_token(TokenKind::Mut) {
                Mutability::Mutable
            } else {
                Mutability::Immutable
            };
            let ty = self.parse_type()?;
            Ok(TypeExprKind::Ref(mutability, Box::new(ty)))
        } else if self.match_token(TokenKind::Box) {
            let ty = self.parse_type()?;
            Ok(TypeExprKind::Box(Box::new(ty)))
        } else if self.match_token(TokenKind::Fn) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let mut params = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    params.push(self.parse_type()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            let return_type = if self.match_token(TokenKind::Arrow) {
                self.parse_type()?
            } else {
                TypeExpr { kind: TypeExprKind::Tuple(Vec::new()), span: self.previous_span() }
            };
            Ok(TypeExprKind::Fn(params, Box::new(return_type)))
        } else if self.match_token(TokenKind::Dyn) {
            let bounds = self.parse_generic_bounds()?;
            Ok(TypeExprKind::TraitObject(bounds))
        } else if self.match_token(TokenKind::Impl) {
            let bounds = self.parse_generic_bounds()?;
            Ok(TypeExprKind::ImplTrait(bounds))
        } else if self.match_token(TokenKind::Bang) {
            Ok(TypeExprKind::Never)
        } else if self.match_token(TokenKind::Underscore) {
            Ok(TypeExprKind::Infer)
        } else {
            // Path type
            let path = self.parse_path()?;
            Ok(TypeExprKind::Path(path))
        }
    }

    // ========================================================================
    // Expression parsing (precedence climbing)
    // ========================================================================

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        self.parse_expr_prec(Precedence::Lowest)
    }

    fn parse_expr_prec(&mut self, min_prec: Precedence) -> Result<Expr, Diagnostic> {
        let mut lhs = self.parse_prefix()?;

        while !self.is_at_end() {
            let prec = self.current_precedence();
            if prec < min_prec {
                break;
            }

            let op_token = self.advance().unwrap();
            let op = self.token_to_binop(&op_token)?;

            let next_prec = if op.is_right_associative() {
                prec
            } else {
                Precedence(prec.0 + 1)
            };

            let rhs = self.parse_expr_prec(next_prec)?;
            let span = Span::new(self.file_id, lhs.span.start, rhs.span.end);

            lhs = Expr {
                kind: ExprKind::Binary(Box::new(lhs), op, Box::new(rhs)),
                span,
                attrs: Vec::new(),
                node_id: self.next_node_id(),
            };
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.current().cloned().ok_or_else(|| self.error("Unexpected EOF".to_string()))?;

        match token.kind {
            TokenKind::Minus | TokenKind::Plus | TokenKind::Not | TokenKind::Tilde |
            TokenKind::Star | TokenKind::Amp | TokenKind::Try | TokenKind::Box => {
                self.advance();
                let op = self.token_to_unop(&token)?;
                let expr = self.parse_prefix()?;
                let span = Span::new(self.file_id, token.span.start, expr.span.end);
                Ok(Expr {
                    kind: ExprKind::Unary(op, Box::new(expr)),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenKind::Dot) {
                if self.check(TokenKind::Ident("await")) {
                    self.advance(); // consume await
                    let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                    expr = Expr {
                        kind: ExprKind::Await(Box::new(expr)),
                        span,
                        attrs: Vec::new(),
                        node_id: self.next_node_id(),
                    };
                } else {
                    let ident = self.parse_ident()?;
                    let generics = self.parse_generic_args()?;
                    if self.match_token(TokenKind::LParen) {
                        let mut args = Vec::new();
                        if !self.check(TokenKind::RParen) {
                            loop {
                                args.push(self.parse_expr()?);
                                if !self.match_token(TokenKind::Comma) {
                                    break;
                                }
                            }
                        }
                        self.expect(TokenKind::RParen, "Expected )")?;
                        let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                        expr = Expr {
                            kind: ExprKind::MethodCall(Box::new(expr), ident, generics, args),
                            span,
                            attrs: Vec::new(),
                            node_id: self.next_node_id(),
                        };
                    } else {
                        let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                        expr = Expr {
                            kind: ExprKind::Field(Box::new(expr), ident),
                            span,
                            attrs: Vec::new(),
                            node_id: self.next_node_id(),
                        };
                    }
                }
            } else if self.match_token(TokenKind::LParen) {
                let mut args = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen, "Expected )")?;
                let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                expr = Expr {
                    kind: ExprKind::Call(Box::new(expr), args),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else if self.match_token(TokenKind::LBracket) {
                let index = self.parse_expr()?;
                self.expect(TokenKind::RBracket, "Expected ]")?;
                let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                expr = Expr {
                    kind: ExprKind::Index(Box::new(expr), Box::new(index)),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else if self.match_token(TokenKind::Question) {
                let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                expr = Expr {
                    kind: ExprKind::TryOperator(Box::new(expr)),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else if self.match_token(TokenKind::DotDot) {
                let inclusive = self.match_token(TokenKind::Assign);
                let end = self.parse_expr()?;
                let span = Span::new(self.file_id, expr.span.start, end.span.end);
                expr = Expr {
                    kind: ExprKind::Range(Box::new(expr), if inclusive { RangeEnd::Inclusive } else { RangeEnd::Exclusive }, Box::new(end)),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else if self.match_token(TokenKind::ColonColon) {
                let ident = self.parse_ident()?;
                let generics = self.parse_generic_args()?;
                let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                expr = Expr {
                    kind: ExprKind::PathSegment(Box::new(expr), ident, generics),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else if self.match_token(TokenKind::As) {
                let ty = self.parse_type()?;
                let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                expr = Expr {
                    kind: ExprKind::Cast(Box::new(expr), ty),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else if self.match_token(TokenKind::PipeRight) {
                // Pipeline operator: expr |> stage1 |> stage2
                let mut stages = Vec::new();
                loop {
                    let stage = self.parse_pipeline_stage_expr()?;
                    stages.push(stage);
                    if !self.match_token(TokenKind::PipeRight) {
                        break;
                    }
                }
                let span = Span::new(self.file_id, expr.span.start, self.previous_span().end);
                expr = Expr {
                    kind: ExprKind::Pipeline(Box::new(expr), stages),
                    span,
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance().ok_or_else(|| self.error("Unexpected EOF".to_string()))?;
        let span = token.span;

        match token.kind {
            TokenKind::Integer(s) => Ok(self.mk_lit(LiteralKind::Integer(s), span)),
            TokenKind::Float(s) => Ok(self.mk_lit(LiteralKind::Float(s), span)),
            TokenKind::Char(c) => Ok(self.mk_lit(LiteralKind::Char(c), span)),
            TokenKind::String(s) => Ok(self.mk_lit(LiteralKind::String(s), span)),
            TokenKind::RawString(s) => Ok(self.mk_lit(LiteralKind::RawString(s), span)),
            TokenKind::ByteString(b) => Ok(self.mk_lit(LiteralKind::ByteString(b), span)),
            TokenKind::RawByteString(b) => Ok(self.mk_lit(LiteralKind::RawByteString(b), span)),
            TokenKind::Bool(b) => Ok(self.mk_lit(LiteralKind::Bool(b), span)),
            TokenKind::Unit => Ok(self.mk_lit(LiteralKind::Unit, span)),
            TokenKind::Ident(name) => {
                // Check for keywords that can start expressions
                match name.as_str() {
                    "if" => self.parse_if_expr(span),
                    "match" => self.parse_match_expr(span),
                    "loop" => self.parse_loop_expr(span),
                    "while" => self.parse_while_expr(span),
                    "for" => self.parse_for_expr(span),
                    "try" => self.parse_try_expr(span),
                    "return" => self.parse_return_expr(span),
                    "break" => self.parse_break_expr(span),
                    "continue" => self.parse_continue_expr(span),
                    "yield" => self.parse_yield_expr(span),
                    "async" => self.parse_async_block(span),
                    // NOVA Unique Features - Expression keywords
                    "perform" => self.parse_effect_perform_expr(span),
                    "handle" => self.parse_effect_handle_expr(span),
                    "spawn" => self.parse_actor_spawn_expr(span),
                    "supervise" => self.parse_actor_supervise_expr(span),
                    "comptime" => self.parse_comptime_expr(span),
                    "move" | "mut" | "ref" | "box" => {
                        // These are parsed as part of patterns, not expressions
                        // Fall through to path
                        self.parse_path_expr(name, span)
                    }
                    _ => self.parse_path_expr(name, span),
                }
            }
            TokenKind::LParen => {
                if self.check(TokenKind::RParen) {
                    self.advance();
                    return Ok(self.mk_lit(LiteralKind::Unit, span));
                }
                let expr = self.parse_expr()?;
                self.expect(TokenKind::RParen, "Expected )")?;
                Ok(Expr {
                    kind: ExprKind::Paren(Box::new(expr)),
                    span: Span::new(self.file_id, span.start, self.previous_span().end),
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                })
            }
            TokenKind::LBrace => {
                self.parse_block_expr(span)
            }
            TokenKind::LBracket => {
                self.parse_array_expr(span)
            }
            TokenKind::Pipe | TokenKind::PipePipe => {
                // Closure
                self.parse_closure_expr(span, false)
            }
            _ => Err(self.error(format!("Unexpected token in expression: {:?}", token.kind))),
        }
    }

    fn mk_lit(&mut self, kind: LiteralKind, span: Span) -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal { kind, suffix: None, span }),
            span,
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        }
    }

    fn parse_path_expr(&mut self, first: String, start_span: Span) -> Result<Expr, Diagnostic> {
        let mut segments = vec![PathSegment {
            ident: Ident { name: first, raw: false, span: start_span },
            generics: self.parse_generic_args()?,
            span: start_span,
        }];

        while self.match_token(TokenKind::ColonColon) {
            let ident = self.parse_ident()?;
            let generics = self.parse_generic_args()?;
            segments.push(PathSegment { ident, generics, span: self.previous_span() });
        }

        let path = Path { segments, span: Span::new(self.file_id, start_span.start, self.previous_span().end), global: false };

        // Check for struct construction: Path { fields }
        if self.match_token(TokenKind::LBrace) {
            let mut fields = Vec::new();
            if !self.check(TokenKind::RBrace) {
                loop {
                    fields.push(self.parse_field_expr()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBrace, "Expected }")?;
            Ok(Expr {
                kind: ExprKind::Struct(path, fields),
                span: Span::new(self.file_id, start_span.start, self.previous_span().end),
                attrs: Vec::new(),
                node_id: self.next_node_id(),
            })
        } else {
            Ok(Expr {
                kind: ExprKind::Path(path),
                span: Span::new(self.file_id, start_span.start, self.previous_span().end),
                attrs: Vec::new(),
                node_id: self.next_node_id(),
            })
        }
    }

    // ... Expression parsing continues with if, match, loop, etc.

    fn parse_if_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let else_branch = if self.match_token(TokenKind::Else) {
            if self.check(TokenKind::If) {
                Some(Box::new(self.parse_if_expr(self.current_span())?))
            } else {
                Some(Box::new(Expr {
                    kind: ExprKind::Block(self.parse_block()?),
                    span: self.previous_span(),
                    attrs: Vec::new(),
                    node_id: self.next_node_id(),
                }))
            }
        } else {
            None
        };

        Ok(Expr {
            kind: ExprKind::If(Box::new(cond), then_branch, else_branch),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_match_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let scrutinee = self.parse_expr()?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);
            self.match_token(TokenKind::Comma);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        Ok(Expr {
            kind: ExprKind::Match(Box::new(scrutinee), arms),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, Diagnostic> {
        let pattern = self.parse_pattern()?;
        let guard = if self.match_token(TokenKind::If) {
            Some(Box::new(self.parse_expr()?))
        } else {
            None
        };
        self.expect(TokenKind::FatArrow, "Expected =>")?;
        let body = self.parse_expr()?;
        Ok(MatchArm { pattern, guard, body: Box::new(body), span: self.previous_span() })
    }

    fn parse_loop_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let label = if self.check(TokenKind::Ident("")) && self.peek(1).map(|t| matches!(t.kind, TokenKind::Colon)).unwrap_or(false) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        if label.is_some() {
            self.expect(TokenKind::Colon, "Expected :")?;
        }
        let body = self.parse_block()?;
        Ok(Expr {
            kind: ExprKind::Loop(body, label),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_while_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let label = if self.check(TokenKind::Ident("")) && self.peek(1).map(|t| matches!(t.kind, TokenKind::Colon)).unwrap_or(false) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        if label.is_some() {
            self.expect(TokenKind::Colon, "Expected :")?;
        }
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Expr {
            kind: ExprKind::While(Box::new(cond), body, label),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_for_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let label = if self.check(TokenKind::Ident("")) && self.peek(1).map(|t| matches!(t.kind, TokenKind::Colon)).unwrap_or(false) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        if label.is_some() {
            self.expect(TokenKind::Colon, "Expected :")?;
        }
        let pattern = self.parse_pattern()?;
        self.expect(TokenKind::In, "Expected in")?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Expr {
            kind: ExprKind::For(pattern, Box::new(iter), body, label),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_try_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let body = self.parse_block()?;
        let mut catches = Vec::new();
        while self.match_token(TokenKind::Catch) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            let catch_body = self.parse_block()?;
            catches.push(CatchClause { pattern, body: catch_body, span: self.previous_span() });
        }
        let finally = if self.match_token(TokenKind::Finally) {
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(Expr {
            kind: ExprKind::Try(Box::new(Expr {
                kind: ExprKind::Block(body),
                span: body.span,
                attrs: Vec::new(),
                node_id: self.next_node_id(),
            }), catches, finally),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_return_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let value = if self.check(TokenKind::Semi) || self.check(TokenKind::RBrace) || self.is_at_end() {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        Ok(Expr {
            kind: ExprKind::Return(value),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_break_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let label = if self.check(TokenKind::Ident("")) && !self.peek(1).map(|t| matches!(t.kind, TokenKind::Colon)).unwrap_or(false) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        let value = if self.check(TokenKind::Semi) || self.check(TokenKind::RBrace) || self.is_at_end() {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        Ok(Expr {
            kind: ExprKind::Break(label, value),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_continue_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let label = if self.check(TokenKind::Ident("")) && !self.peek(1).map(|t| matches!(t.kind, TokenKind::Colon)).unwrap_or(false) {
            Some(self.parse_ident()?)
        } else {
            None
        };
        Ok(Expr {
            kind: ExprKind::Continue(label),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_yield_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let value = if self.check(TokenKind::Semi) || self.check(TokenKind::RBrace) || self.is_at_end() {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        Ok(Expr {
            kind: ExprKind::Yield(value),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_async_block(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let block = self.parse_block()?;
        Ok(Expr {
            kind: ExprKind::Async(block),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    // ========================================================================
    // NOVA Unique Features - Expression Parsing
    // ========================================================================

    fn parse_effect_perform_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        // perform Effect::operation(args)
        let path = self.parse_path()?;
        self.expect(TokenKind::LParen, "Expected (")?;
        let args = self.parse_expr_list()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        Ok(Expr {
            kind: ExprKind::EffectPerform(path, args),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_effect_handle_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        // handle expr with { ... }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Ident("with".to_string()), "Expected 'with'")?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut handlers = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            handlers.push(self.parse_effect_handler_arm()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        Ok(Expr {
            kind: ExprKind::EffectHandle(Box::new(expr), handlers),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_actor_spawn_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        // spawn(expr)
        self.expect(TokenKind::LParen, "Expected (")?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        Ok(Expr {
            kind: ExprKind::ActorSpawn(Box::new(expr)),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_actor_supervise_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        // supervise(expr, spec)
        self.expect(TokenKind::LParen, "Expected (")?;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Comma, "Expected ,")?;
        let spec = self.parse_supervision_spec()?;
        self.expect(TokenKind::RParen, "Expected )")?;
        Ok(Expr {
            kind: ExprKind::ActorSupervise(Box::new(expr), spec),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_comptime_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        // comptime { ... }
        let block = self.parse_block()?;
        Ok(Expr {
            kind: ExprKind::Comptime(Box::new(Expr {
                kind: ExprKind::Block(block),
                span: self.previous_span(),
                attrs: Vec::new(),
                node_id: self.next_node_id(),
            })),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_pipeline_stage_expr(&mut self) -> Result<PipelineStage, Diagnostic> {
        let attrs = Vec::new();
        let kind = if self.match_token(TokenKind::Ident("filter".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Filter(expr)
        } else if self.match_token(TokenKind::Ident("map".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Map(expr)
        } else if self.match_token(TokenKind::Ident("flat_map".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::FlatMap(expr)
        } else if self.match_token(TokenKind::Ident("fold".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let init = self.parse_expr()?;
            self.expect(TokenKind::Comma, "Expected ,")?;
            let func = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Fold(init, func)
        } else if self.match_token(TokenKind::Ident("reduce".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let func = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Reduce(func)
        } else if self.match_token(TokenKind::Ident("collect".to_string())) {
            let ty = if self.match_token(TokenKind::LParen) {
                if self.check(TokenKind::RBracket) || self.check(TokenKind::Ident("".to_string())) {
                    let ty = self.parse_type_expr()?;
                    self.expect(TokenKind::RParen, "Expected )")?;
                    Some(ty)
                } else {
                    self.expect(TokenKind::RParen, "Expected )")?;
                    None
                }
            } else {
                None
            };
            PipelineStageKind::Collect(ty)
        } else if self.match_token(TokenKind::Ident("take".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Take(expr)
        } else if self.match_token(TokenKind::Ident("skip".to_string())) {
            self.expect(TokenKind::LParen, "Expected (")?;
            let expr = self.parse_expr()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Skip(expr)
        } else {
            let ident = self.parse_ident()?;
            self.expect(TokenKind::LParen, "Expected (")?;
            let args = self.parse_expr_list()?;
            self.expect(TokenKind::RParen, "Expected )")?;
            PipelineStageKind::Custom(ident, args)
        };
        Ok(PipelineStage {
            attrs,
            kind,
            span: self.previous_span(),
        })
    }

    fn parse_block_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let block = self.parse_block_inner(start_span)?;
        Ok(Expr {
            kind: ExprKind::Block(block),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_block(&mut self) -> Result<Block, Diagnostic> {
        self.parse_block_inner(self.current_span())
    }

    fn parse_block_inner(&mut self, start_span: Span) -> Result<Block, Diagnostic> {
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut stmts = Vec::new();
        let mut expr = None;
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            if self.check(TokenKind::Semi) {
                self.advance();
                continue;
            }
            // Check if this is the last expression (no semicolon)
            let is_last = self.peek(1).map(|t| matches!(t.kind, TokenKind::RBrace)).unwrap_or(false);
            let stmt = self.parse_stmt()?;
            if is_last && matches!(stmt.kind, StmtKind::Expr(_)) {
                expr = Some(Box::new(match stmt.kind {
                    StmtKind::Expr(e) => e,
                    _ => unreachable!(),
                }));
            } else {
                stmts.push(stmt);
            }
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        Ok(Block {
            stmts,
            expr,
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            labels: Vec::new(),
        })
    }

    fn parse_array_expr(&mut self, start_span: Span) -> Result<Expr, Diagnostic> {
        let mut elems = Vec::new();
        if !self.check(TokenKind::RBracket) {
            loop {
                elems.push(self.parse_expr()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RBracket, "Expected ]")?;
        Ok(Expr {
            kind: ExprKind::Array(elems),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_field_expr(&mut self) -> Result<FieldExpr, Diagnostic> {
        let name = self.parse_ident()?;
        self.expect(TokenKind::Colon, "Expected :")?;
        let value = self.parse_expr()?;
        Ok(FieldExpr { name, value, span: self.previous_span() })
    }

    fn parse_closure_expr(&mut self, start_span: Span, is_async: bool) -> Result<Expr, Diagnostic> {
        // |params| body
        let mut params = Vec::new();
        if !self.check(TokenKind::Pipe) {
            loop {
                params.push(self.parse_closure_param()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::Pipe, "Expected |")?;
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = self.parse_block()?;
        Ok(Expr {
            kind: ExprKind::Closure(ClosureExpr {
                params,
                return_type,
                body,
                asyncness: if is_async { Asyncness::Async } else { Asyncness::NotAsync },
                movability: Movability::Move,
                span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            }),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs: Vec::new(),
            node_id: self.next_node_id(),
        })
    }

    fn parse_closure_param(&mut self) -> Result<ClosureParam, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let name = self.parse_ident()?;
        let ty = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        Ok(ClosureParam { attrs, name, ty, span: self.previous_span() })
    }

    // ========================================================================
    // Pattern parsing
    // ========================================================================

    fn parse_pattern(&mut self) -> Result<Pattern, Diagnostic> {
        if self.match_token(TokenKind::Underscore) {
            return Ok(Pattern::Wildcard(self.previous_span()));
        }

        // Check for literal
        if let Some(token) = self.current().cloned() {
            match token.kind {
                TokenKind::Integer(s) => { self.advance(); return Ok(Pattern::Literal(Literal { kind: LiteralKind::Integer(s), suffix: None, span: token.span })); }
                TokenKind::Float(s) => { self.advance(); return Ok(Pattern::Literal(Literal { kind: LiteralKind::Float(s), suffix: None, span: token.span })); }
                TokenKind::Char(c) => { self.advance(); return Ok(Pattern::Literal(Literal { kind: LiteralKind::Char(c), suffix: None, span: token.span })); }
                TokenKind::String(s) => { self.advance(); return Ok(Pattern::Literal(Literal { kind: LiteralKind::String(s), suffix: None, span: token.span })); }
                TokenKind::Bool(b) => { self.advance(); return Ok(Pattern::Literal(Literal { kind: LiteralKind::Bool(b), suffix: None, span: token.span })); }
                TokenKind::Unit => { self.advance(); return Ok(Pattern::Literal(Literal { kind: LiteralKind::Unit, suffix: None, span: token.span })); }
                _ => {}
            }
        }

        // Check for ref pattern
        if self.match_token(TokenKind::Amp) {
            let mutability = if self.match_token(TokenKind::Mut) { Mutability::Mutable } else { Mutability::Immutable };
            let pattern = self.parse_pattern()?;
            return Ok(Pattern::Ref(mutability, Box::new(pattern)));
        }

        // Check for binding @ pattern
        if self.check(TokenKind::Ident("")) && self.peek(1).map(|t| matches!(t.kind, TokenKind::At)).unwrap_or(false) {
            let ident = self.parse_ident()?;
            self.expect(TokenKind::At, "Expected @")?;
            let sub = self.parse_pattern()?;
            return Ok(Pattern::Binding(ident, Some(Box::new(sub))));
        }

        // Path-based patterns (struct, enum, const)
        if self.check(TokenKind::Ident("")) {
            let path = self.parse_path()?;
            if self.match_token(TokenKind::LBrace) {
                // Struct pattern
                let mut fields = Vec::new();
                if !self.check(TokenKind::RBrace) {
                    loop {
                        fields.push(self.parse_field_pattern()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RBrace, "Expected }")?;
                return Ok(Pattern::Struct(path, fields));
            } else if self.match_token(TokenKind::LParen) {
                // Tuple struct / enum variant
                let mut patterns = Vec::new();
                if !self.check(TokenKind::RParen) {
                    loop {
                        patterns.push(self.parse_pattern()?);
                        if !self.match_token(TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(TokenKind::RParen, "Expected )")?;
                // Heuristic: if path has :: and last segment starts with uppercase, treat as enum
                let is_enum = path.segments.last().map(|s| s.ident.name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)).unwrap_or(false);
                if is_enum {
                    let variant = path.segments.pop().unwrap().ident;
                    return Ok(Pattern::Enum(path, variant, Some(patterns)));
                } else {
                    return Ok(Pattern::Tuple(patterns));
                }
            } else if self.check(TokenKind::LBrace) || self.check(TokenKind::LParen) {
                // Enum variant without fields
                let variant = path.segments.pop().unwrap().ident;
                return Ok(Pattern::Enum(path, variant, None));
            } else {
                // Could be const pattern or binding
                let ident = path.segments.pop().unwrap().ident;
                if path.segments.is_empty() {
                    // Simple identifier - binding pattern
                    return Ok(Pattern::Binding(ident, None));
                } else {
                    // Qualified path - const pattern
                    return Ok(Pattern::Literal(Literal { kind: LiteralKind::String(path.segments.last().unwrap().ident.name.clone()), suffix: None, span: self.previous_span() }));
                }
            }
        }

        // Tuple pattern
        if self.match_token(TokenKind::LParen) {
            let mut patterns = Vec::new();
            if !self.check(TokenKind::RParen) {
                loop {
                    patterns.push(self.parse_pattern()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RParen, "Expected )")?;
            return Ok(Pattern::Tuple(patterns));
        }

        // Slice pattern
        if self.match_token(TokenKind::LBracket) {
            let mut prefix = Vec::new();
            let mut rest = None;
            if !self.check(TokenKind::RBracket) {
                loop {
                    if self.match_token(TokenKind::DotDot) {
                        if !self.check(TokenKind::RBracket) {
                            rest = Some(Box::new(self.parse_pattern()?));
                        }
                        break;
                    }
                    prefix.push(self.parse_pattern()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RBracket, "Expected ]")?;
            return Ok(Pattern::Slice(prefix, rest));
        }

        // Or pattern
        let mut first = self.parse_pattern_atom()?;
        if self.match_token(TokenKind::Pipe) {
            let mut patterns = vec![first];
            loop {
                patterns.push(self.parse_pattern_atom()?);
                if !self.match_token(TokenKind::Pipe) {
                    break;
                }
            }
            return Ok(Pattern::Or(patterns));
        }

        Ok(first)
    }

    fn parse_pattern_atom(&mut self) -> Result<Pattern, Diagnostic> {
        // Handle the atomic patterns that don't start with special chars
        if self.match_token(TokenKind::Underscore) {
            return Ok(Pattern::Wildcard(self.previous_span()));
        }
        // ... simplified for brevity
        Err(self.error("Unexpected pattern".to_string()))
    }

    fn parse_field_pattern(&mut self) -> Result<FieldPattern, Diagnostic> {
        let name = self.parse_ident()?;
        let pattern = if self.match_token(TokenKind::Colon) {
            Some(self.parse_pattern()?)
        } else {
            None
        };
        Ok(FieldPattern { name, pattern, span: self.previous_span() })
    }

    // ========================================================================
    // Statement parsing
    // ========================================================================

    fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let start_span = self.current_span();

        // NOVA Unique Features - Statements
        if self.match_token(TokenKind::Ident("handle".to_string())) {
            return self.parse_effect_handle_stmt(attrs, start_span);
        }
        if self.match_token(TokenKind::Ident("spawn".to_string())) {
            return self.parse_actor_spawn_stmt(attrs, start_span);
        }
        if self.match_token(TokenKind::Ident("comptime".to_string())) {
            return self.parse_comptime_stmt(attrs, start_span);
        }

        if self.match_token(TokenKind::Let) {
            return self.parse_let_stmt(attrs, start_span);
        }

        if self.check(TokenKind::Fn) || self.check(TokenKind::Struct) || self.check(TokenKind::Enum) ||
           self.check(TokenKind::Mod) || self.check(TokenKind::Const) || self.check(TokenKind::Static) ||
           self.check(TokenKind::Trait) || self.check(TokenKind::Impl) || self.check(TokenKind::Type) ||
           self.check(TokenKind::Use) || self.check(TokenKind::Macro) || self.check(TokenKind::Pub) ||
           self.check(TokenKind::Async) || self.check(TokenKind::Unsafe) || self.check(TokenKind::Const) ||
           self.check(TokenKind::Extern) || self.check(TokenKind::Effect) || self.check(TokenKind::Handle) ||
           self.check(TokenKind::Actor) || self.check(TokenKind::Comptime) || self.check(TokenKind::Pipeline) {
            let item = self.parse_item()?;
            return Ok(Stmt { kind: StmtKind::Item(item), span: start_span, attrs, node_id: self.next_node_id() });
        }

        if self.match_token(TokenKind::Semi) {
            return Ok(Stmt { kind: StmtKind::Empty, span: start_span, attrs, node_id: self.next_node_id() });
        }

        // Expression statement
        let expr = self.parse_expr()?;
        if !self.match_token(TokenKind::Semi) && !self.check(TokenKind::RBrace) && !self.is_at_end() {
            // Could be a let without semicolon in block tail position
            // For now, require semicolon for statements
        }
        Ok(Stmt { kind: StmtKind::Expr(expr), span: Span::new(self.file_id, start_span.start, self.previous_span().end), attrs, node_id: self.next_node_id() })
    }

    fn parse_let_stmt(&mut self, attrs: Vec<Attribute>, start_span: Span) -> Result<Stmt, Diagnostic> {
        let mutability = if self.match_token(TokenKind::Mut) { Mutability::Mutable } else { Mutability::Immutable };
        let pattern = self.parse_pattern()?;
        let ty = if self.match_token(TokenKind::Colon) { Some(self.parse_type()?) } else { None };
        let initializer = if self.match_token(TokenKind::Assign) { Some(self.parse_expr()?) } else { None };
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Stmt {
            kind: StmtKind::Let(LetStmt { mutability, pattern, ty, initializer, span: Span::new(self.file_id, start_span.start, self.previous_span().end) }),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs,
            node_id: self.next_node_id(),
        })
    }

    // NOVA Unique Features - Statement Parsing
    fn parse_effect_handle_stmt(&mut self, attrs: Vec<Attribute>, start_span: Span) -> Result<Stmt, Diagnostic> {
        // handle expr with { ... }
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Ident("with".to_string()), "Expected 'with'")?;
        self.expect(TokenKind::LBrace, "Expected {")?;
        let mut handlers = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            handlers.push(self.parse_effect_handler_arm()?);
        }
        self.expect(TokenKind::RBrace, "Expected }")?;
        Ok(Stmt {
            kind: StmtKind::EffectHandle(Box::new(expr), handlers),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs,
            node_id: self.next_node_id(),
        })
    }

    fn parse_actor_spawn_stmt(&mut self, attrs: Vec<Attribute>, start_span: Span) -> Result<Stmt, Diagnostic> {
        // spawn expr;
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Stmt {
            kind: StmtKind::ActorSpawn(Box::new(expr)),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs,
            node_id: self.next_node_id(),
        })
    }

    fn parse_comptime_stmt(&mut self, attrs: Vec<Attribute>, start_span: Span) -> Result<Stmt, Diagnostic> {
        // comptime { ... };
        let block = self.parse_block()?;
        self.expect(TokenKind::Semi, "Expected ;")?;
        Ok(Stmt {
            kind: StmtKind::Comptime(block),
            span: Span::new(self.file_id, start_span.start, self.previous_span().end),
            attrs,
            node_id: self.next_node_id(),
        })
    }

    // ========================================================================
    // Generics and where clauses
    // ========================================================================

    fn parse_generics(&mut self) -> Result<Generics, Diagnostic> {
        if !self.match_token(TokenKind::Lt) {
            return Ok(Generics { params: Vec::new(), span: self.previous_span() });
        }

        let mut params = Vec::new();
        if !self.check(TokenKind::Gt) {
            loop {
                params.push(self.parse_generic_param()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::Gt, "Expected >")?;
        Ok(Generics { params, span: self.previous_span() })
    }

    fn parse_generic_param(&mut self) -> Result<GenericParam, Diagnostic> {
        let attrs = self.parse_attributes()?;
        let name = self.parse_ident()?;
        let kind = if self.match_token(TokenKind::Colon) {
            let bounds = self.parse_generic_bounds()?;
            let default = if self.match_token(TokenKind::Assign) {
                Some(self.parse_type()?)
            } else {
                None
            };
            GenericParamKind::Type { bounds: Some(bounds), default }
        } else if self.match_token(TokenKind::Assign) {
            GenericParamKind::Type { bounds: None, default: Some(self.parse_type()?) }
        } else {
            GenericParamKind::Type { bounds: None, default: None }
        };
        Ok(GenericParam { attrs, name, kind, span: self.previous_span() })
    }

    fn parse_generic_bounds(&mut self) -> Result<GenericBounds, Diagnostic> {
        let mut bounds = Vec::new();
        loop {
            let trait_ref = self.parse_trait_ref()?;
            bounds.push(GenericBound { trait_ref, span: self.previous_span() });
            if !self.match_token(TokenKind::Plus) {
                break;
            }
        }
        Ok(GenericBounds { bounds, span: self.previous_span() })
    }

    fn parse_trait_ref(&mut self) -> Result<TraitRef, Diagnostic> {
        let path = self.parse_path()?;
        let generics = self.parse_generic_args()?;
        Ok(TraitRef { path, generics, span: self.previous_span() })
    }

    fn parse_generic_args(&mut self) -> Result<Option<GenericArgs>, Diagnostic> {
        if !self.match_token(TokenKind::Lt) {
            return Ok(None);
        }
        let mut args = Vec::new();
        if !self.check(TokenKind::Gt) {
            loop {
                if self.check(TokenKind::Ident("lifetime")) || self.check(TokenKind::Ident("'")) {
                    // Lifetime
                    let name = self.parse_ident()?;
                    args.push(GenericArg::Lifetime(Lifetime { name, span: self.previous_span() }));
                } else if self.check(TokenKind::Integer("")) {
                    // Const generic
                    let expr = self.parse_expr()?;
                    args.push(GenericArg::Const(expr));
                } else {
                    // Type
                    let ty = self.parse_type()?;
                    args.push(GenericArg::Type(ty));
                }
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::Gt, "Expected >")?;
        Ok(Some(GenericArgs { args, span: self.previous_span() }))
    }

    fn parse_where_clause(&mut self) -> Result<Option<WhereClause>, Diagnostic> {
        if !self.match_token(TokenKind::Where) {
            return Ok(None);
        }
        let mut predicates = Vec::new();
        loop {
            let ty = self.parse_type()?;
            self.expect(TokenKind::Colon, "Expected :")?;
            let bounds = self.parse_generic_bounds()?;
            predicates.push(WherePredicate { ty, bounds, span: self.previous_span() });
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(Some(WhereClause { predicates, span: self.previous_span() }))
    }

    // ========================================================================
    // Helper parsing functions
    // ========================================================================

    fn parse_ident(&mut self) -> Result<Ident, Diagnostic> {
        let token = self.advance().ok_or_else(|| self.error("Expected identifier".to_string()))?;
        match token.kind {
            TokenKind::Ident(name) => Ok(Ident { name, raw: false, span: token.span }),
            TokenKind::RawIdent(name) => Ok(Ident { name, raw: true, span: token.span }),
            _ => Err(self.error("Expected identifier".to_string())),
        }
    }

    fn parse_path(&mut self) -> Result<Path, Diagnostic> {
        let global = self.match_token(TokenKind::ColonColon);
        let mut segments = Vec::new();
        loop {
            let ident = self.parse_ident()?;
            let generics = self.parse_generic_args()?;
            segments.push(PathSegment { ident, generics, span: self.previous_span() });
            if !self.match_token(TokenKind::ColonColon) {
                break;
            }
        }
        Ok(Path { segments, span: self.previous_span(), global })
    }

    fn token_to_literal(&self, token: Token) -> Result<Literal, Diagnostic> {
        Ok(match token.kind {
            TokenKind::Integer(s) => Literal { kind: LiteralKind::Integer(s), suffix: None, span: token.span },
            TokenKind::Float(s) => Literal { kind: LiteralKind::Float(s), suffix: None, span: token.span },
            TokenKind::Char(c) => Literal { kind: LiteralKind::Char(c), suffix: None, span: token.span },
            TokenKind::String(s) => Literal { kind: LiteralKind::String(s), suffix: None, span: token.span },
            TokenKind::RawString(s) => Literal { kind: LiteralKind::RawString(s), suffix: None, span: token.span },
            TokenKind::ByteString(b) => Literal { kind: LiteralKind::ByteString(b), suffix: None, span: token.span },
            TokenKind::RawByteString(b) => Literal { kind: LiteralKind::RawByteString(b), suffix: None, span: token.span },
            TokenKind::Bool(b) => Literal { kind: LiteralKind::Bool(b), suffix: None, span: token.span },
            TokenKind::Unit => Literal { kind: LiteralKind::Unit, suffix: None, span: token.span },
            _ => return Err(self.error("Expected literal".to_string())),
        })
    }

    fn token_to_binop(&self, token: &Token) -> Result<BinOp, Diagnostic> {
        Ok(match &token.kind {
            TokenKind::Plus => BinOp::Add,
            TokenKind::Minus => BinOp::Sub,
            TokenKind::Star => BinOp::Mul,
            TokenKind::Slash => BinOp::Div,
            TokenKind::Percent => BinOp::Rem,
            TokenKind::Amp => BinOp::BitAnd,
            TokenKind::Pipe => BinOp::BitOr,
            TokenKind::Caret => BinOp::BitXor,
            TokenKind::Shl => BinOp::Shl,
            TokenKind::Shr => BinOp::Shr,
            TokenKind::EqEq => BinOp::Eq,
            TokenKind::Ne => BinOp::Ne,
            TokenKind::Lt => BinOp::Lt,
            TokenKind::Le => BinOp::Le,
            TokenKind::Gt => BinOp::Gt,
            TokenKind::Ge => BinOp::Ge,
            TokenKind::AndAnd => BinOp::And,
            TokenKind::OrOr => BinOp::Or,
            TokenKind::As => BinOp::As,
            TokenKind::Is => BinOp::Is,
            TokenKind::In => BinOp::In,
            _ => return Err(self.error(format!("Not a binary operator: {:?}", token.kind))),
        })
    }

    fn token_to_unop(&self, token: &Token) -> Result<UnOp, Diagnostic> {
        Ok(match &token.kind {
            TokenKind::Minus => UnOp::Neg,
            TokenKind::Not => UnOp::Not,
            TokenKind::Tilde => UnOp::BitNot,
            TokenKind::Star => UnOp::Deref,
            TokenKind::Amp => UnOp::Ref,
            TokenKind::Mut => UnOp::RefMut, // &mut handled separately
            TokenKind::Box => UnOp::Box,
            TokenKind::Try => UnOp::Try,
            _ => return Err(self.error(format!("Not a unary operator: {:?}", token.kind))),
        })
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        self.current().map(|t| matches!(t.kind, TokenKind::Ident(ref s) if s == keyword)).unwrap_or(false)
    }

    // Precedence handling
    fn current_precedence(&self) -> Precedence {
        self.current().map(|t| self.precedence_of(&t.kind)).unwrap_or(Precedence::Lowest)
    }

    fn precedence_of(&self, kind: &TokenKind) -> Precedence {
        use BinOp::*;
        match kind {
            TokenKind::OrOr => Precedence(1),      // ||
            TokenKind::AndAnd => Precedence(2),    // &&
            TokenKind::Pipe => Precedence(3),      // |
            TokenKind::Caret => Precedence(4),     // ^
            TokenKind::Amp => Precedence(5),       // &
            TokenKind::EqEq | TokenKind::Ne => Precedence(6), // == !=
            TokenKind::Lt | TokenKind::Le | TokenKind::Gt | TokenKind::Ge => Precedence(7), // < <= > >=
            TokenKind::Shl | TokenKind::Shr => Precedence(8),   // << >>
            TokenKind::Plus | TokenKind::Minus => Precedence(9), // + -
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence(10), // * / %
            TokenKind::As | TokenKind::Is | TokenKind::In => Precedence(11), // as is in
            _ => Precedence::Lowest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Precedence(u8);

impl Precedence {
    const Lowest: Self = Precedence(0);
    const Highest: Self = Precedence(255);
}

impl BinOp {
    fn is_right_associative(&self) -> bool {
        matches!(self, BinOp::As | BinOp::Is | BinOp::In)
    }
}

/// Parse function for external use
pub fn parse(session: &mut Session, file_id: FileId, tokens: &[Token]) -> Result<Program, Vec<Diagnostic>> {
    let mut parser = Parser::new(session, tokens.to_vec(), file_id);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompileConfig, Target, OptLevel, Edition};

    #[test]
    fn parse_simple_function() {
        let mut session = Session::new(CompileConfig {
            target: Target::Native,
            opt_level: OptLevel::None,
            debug_info: false,
            test_mode: false,
            doc_mode: false,
            edition: Edition::Nova2026,
        });

        let source = r#"
fn main() {
    let x = 42;
    println!("Hello, {}!", x);
}
"#;

        let file_id = session.add_file("test.nova", source.to_string());
        let tokens = crate::lexer::lex(&mut session, file_id, source).unwrap();
        let program = parse(&mut session, file_id, &tokens).unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Function(f) => {
                assert_eq!(f.name.name, "main");
                assert_eq!(f.params.len(), 0);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn parse_struct() {
        let mut session = Session::new(CompileConfig {
            target: Target::Native,
            opt_level: OptLevel::None,
            debug_info: false,
            test_mode: false,
            doc_mode: false,
            edition: Edition::Nova2026,
        });

        let source = r#"
struct Point {
    x: f64,
    y: f64,
}
"#;

        let file_id = session.add_file("test.nova", source.to_string());
        let tokens = crate::lexer::lex(&mut session, file_id, source).unwrap();
        let program = parse(&mut session, file_id, &tokens).unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name.name, "Point");
                match &s.kind {
                    StructKind::Record(fields) => {
                        assert_eq!(fields.len(), 2);
                        assert_eq!(fields[0].name.name, "x");
                        assert_eq!(fields[1].name.name, "y");
                    }
                    _ => panic!("Expected record struct"),
                }
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn parse_enum() {
        let mut session = Session::new(CompileConfig {
            target: Target::Native,
            opt_level: OptLevel::None,
            debug_info: false,
            test_mode: false,
            doc_mode: false,
            edition: Edition::Nova2026,
        });

        let source = r#"
enum Option<T> {
    Some(T),
    None,
}
"#;

        let file_id = session.add_file("test.nova", source.to_string());
        let tokens = crate::lexer::lex(&mut session, file_id, source).unwrap();
        let program = parse(&mut session, file_id, &tokens).unwrap();

        assert_eq!(program.items.len(), 1);
        match &program.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.name.name, "Option");
                assert_eq!(e.generics.params.len(), 1);
                assert_eq!(e.variants.len(), 2);
            }
            _ => panic!("Expected enum"),
        }
    }
}