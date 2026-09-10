//! Application-defined native editing rules; compilation precedes publication.
use crate::native::{CommandDefinition, ModuleDefinition};
use gpui::App;
use gpui_base::input::language_config::{
    AutoClosingPair, BracketPair, IndentationRules, LanguageConfig,
};
use std::sync::Arc;

#[crate::native_type]
#[derive(Clone)]
pub struct EditorBracketPair {
    pub open: String,
    pub close: String,
}
#[crate::native_type]
#[derive(Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum EditorSyntaxContext {
    String,
    Comment,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditorAutoClosingPair {
    pub open: String,
    pub close: String,
    #[serde(default)]
    pub not_in: Vec<EditorSyntaxContext>,
}
#[crate::native_type]
#[derive(Clone, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct EditorIndentationRules {
    pub increase: Option<String>,
    pub decrease: Option<String>,
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct EditorLanguageConfig {
    pub language: String,
    #[serde(default)]
    pub brackets: Option<Vec<EditorBracketPair>>,
    #[serde(default)]
    pub auto_closing_pairs: Option<Vec<EditorAutoClosingPair>>,
    #[serde(default)]
    pub auto_close_before: Option<String>,
    #[serde(default)]
    pub indentation: Option<EditorIndentationRules>,
}

fn delimiter(open: &str, close: &str) -> Result<(), String> {
    if open.is_empty() || close.is_empty() || open.len() > 64 || close.len() > 64 {
        return Err("editor delimiters must contain 1..64 bytes".into());
    }
    Ok(())
}
fn pattern(source: Option<String>) -> Result<Option<Arc<regex::Regex>>, String> {
    source
        .map(|source| {
            if source.len() > 4096 {
                return Err("indentation patterns must not exceed 4096 bytes".into());
            }
            regex::RegexBuilder::new(&source)
                .size_limit(1_048_576)
                .build()
                .map(Arc::new)
                .map_err(|error| format!("invalid indentation pattern: {error}"))
        })
        .transpose()
}
fn configure(request: EditorLanguageConfig, cx: &mut App) -> Result<(), String> {
    if request.language.trim().is_empty() || request.language.len() > 128 {
        return Err("editor language must contain 1..128 bytes".into());
    }
    let mut config = LanguageConfig::default();
    if let Some(pairs) = request.brackets {
        if pairs.len() > 64 {
            return Err("at most 64 structural pairs are supported".into());
        }
        for pair in &pairs {
            delimiter(&pair.open, &pair.close)?;
        }
        config = config.brackets(
            pairs
                .into_iter()
                .map(|pair| BracketPair::new(pair.open, pair.close)),
        );
    }
    if let Some(pairs) = request.auto_closing_pairs {
        if pairs.len() > 64 {
            return Err("at most 64 automatic pairs are supported".into());
        }
        for pair in &pairs {
            delimiter(&pair.open, &pair.close)?;
            if pair.not_in.len() > 2 {
                return Err("notIn accepts string and comment contexts".into());
            }
        }
        config = config.auto_closing_pairs(pairs.into_iter().map(|pair| {
            AutoClosingPair::new(pair.open, pair.close).not_in(pair.not_in.into_iter().map(
                |context| match context {
                    EditorSyntaxContext::String => gpui_base::input::SyntaxContext::String,
                    EditorSyntaxContext::Comment => gpui_base::input::SyntaxContext::Comment,
                },
            ))
        }));
    }
    if let Some(before) = request.auto_close_before {
        if before.len() > 4096 {
            return Err("autoCloseBefore must not exceed 4096 bytes".into());
        }
        config = config.auto_close_before(before);
    }
    if let Some(indentation) = request.indentation {
        let mut rules = IndentationRules::default();
        rules.increase_indent_pattern = pattern(indentation.increase)?;
        rules.decrease_indent_pattern = pattern(indentation.decrease)?;
        config = config.indentation_rules(rules);
    }
    gpui_component::input::set_language_config(request.language, config, cx);
    Ok(())
}
pub(super) fn native_module() -> ModuleDefinition {
    ModuleDefinition::new(
        "editor-language",
        vec![],
        vec![CommandDefinition::foreground(
            "configureEditorLanguage",
            |request: EditorLanguageConfig, _, cx| configure(request, cx),
        )],
    )
    .with_contract(include_str!("input_language.rs"))
}
