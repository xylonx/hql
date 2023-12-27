//! The variant selector implementations
//!
//! The Selector is a trait which accepts a `ElementOrTextRef` node and returns
//! a vector of ElementOrTextRef nodes. And for many selector implementations, Use
//! [enum_dispatch](https://docs.rs/enum_dispatch) trick instead of dynamic dispatch
//! for better performance.
//!
//! Each of the submodule contains a group of selector implementations, like selecting
//! by path like xpath or selecting by classes like css selector.
//!
//! Basically, the selectors is categorized as two types:
//! - filter or generate nodes based on some rules, with `@` prefix, like @path(`//div`)
//! - handle node inner text based on some rules, with `#` prefix, like `#text()`
//!
//! # Examples
//!
//! ## Manually
//!
//! ```
//! let selectors: Vec<SelectorEnum> = vec![
//!     PathSelector::new(vec![(Path::Travel, "div".into()), (Path::Single, "a".into())]).into(),
//!     FlatSelector::new().into(),
//! ]
//! ```
//!
//! ## Parse HQL
//!
//! ```
//! let selectors: Vec<SelectorEnum> =
//!     try_parse_hql("@path(`//div/a`) | @flat()").unwrap_or_else(|e| panic!("{}", e));
//! ```
//!
//! The full HQL grammar is define in [grammar.pest](https://github.com/xylonx/hql/tree/master/src/selector/grammar.pest)

pub mod attr;
pub mod path;
pub mod text;

use enum_dispatch::enum_dispatch;
use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;

use crate::html::ElementOrTextRef;

use self::{attr::*, path::*, text::*};

#[enum_dispatch]
#[derive(Debug, PartialEq)]
pub enum SelectorEnum {
    PathSelector,

    AttrSelector,
    ClassSelector,
    IDSelector,

    FlatSelector,

    TextSelector,
    TrimSelector,
    TrimPrefixSelector,
    TrimSuffixSelector,
    NthChildSelector,
    ExtractAttrSelector,
}

#[enum_dispatch(SelectorEnum)]
pub trait Selector: PartialEq {
    /// TODO(xylonx): use iterator tricks instead of Vec here to avoid intermediate memory consumption
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>>;
}

#[derive(Debug, Parser)]
#[grammar = "selector/grammar.pest"]
struct HqlParser;

impl HqlParser {
    fn parse_path(pair: Pair<'_, Rule>) -> (Path, String) {
        let mut pairs = pair.into_inner();

        let p_node = match pairs.next().unwrap().as_rule() {
            Rule::singlePath => Path::Single,
            Rule::travelPath => Path::Travel,
            _ => unreachable!(),
        };

        let t = pairs.next().unwrap();
        let tag = match t.as_rule() {
            Rule::tag => t.as_str().to_string(),
            _ => unreachable!(),
        };

        (p_node, tag)
    }

    // quotedPath
    fn parse_paths(pairs: Pairs<'_, Rule>) -> SelectorEnum {
        PathSelector::new(
            pairs
                .into_iter()
                .next()
                .unwrap()
                .into_inner()
                .map(Self::parse_path)
                .collect(),
        )
        .into()
        // .into()
    }

    fn parse_attr(mut pairs: Pairs<'_, Rule>) -> SelectorEnum {
        let name = pairs.next().unwrap().into_inner().next().unwrap();
        let name_str = match name.as_rule() {
            Rule::attrField => name.as_str().to_string(),
            _ => unreachable!(),
        };

        match pairs.next() {
            Some(v) => {
                AttrSelector::new(&name_str, Some(v.into_inner().next().unwrap().as_str())).into()
            }
            None => AttrSelector::new(&name_str, None).into(),
        }
    }

    /// parse pairs into IDSelector, with case sensitive as default
    fn parse_id(mut pairs: Pairs<'_, Rule>) -> SelectorEnum {
        let id = pairs.next().unwrap().into_inner().next().unwrap();
        let id_str = match id.as_rule() {
            Rule::attrField => id.as_str().to_string(),
            _ => unreachable!(),
        };

        let case_sensitive = pairs.next();

        if let Some(c) = case_sensitive {
            if matches!(c.as_rule(), Rule::caseSensitiveOpt) && c.as_str() == "0" {
                return IDSelector::new(id_str, false).into();
            }
        }

        IDSelector::new(id_str, true).into()
    }

    /// parse pairs into ClassSelector, with case sensitive as default
    fn parse_class(mut pairs: Pairs<'_, Rule>) -> SelectorEnum {
        let class = pairs.next().unwrap().into_inner().next().unwrap();
        let class_str = match class.as_rule() {
            Rule::attrField => class.as_str().to_string(),
            _ => unreachable!(),
        };

        let case_sensitive = pairs.next();

        if let Some(c) = case_sensitive {
            if matches!(c.as_rule(), Rule::caseSensitiveOpt) && c.as_str() == "0" {
                return ClassSelector::new(class_str, false).into();
            }
        }

        ClassSelector::new(class_str, true).into()
    }

    fn parse_child(mut pairs: Pairs<'_, Rule>) -> SelectorEnum {
        let n_str = pairs.next().unwrap().as_str();

        // grammar ensures n_str contains at least one characters
        let (neg_sign, n) = match &n_str[0..=0] {
            "-" => (true, n_str[1..=n_str.len() - 1].parse::<usize>().unwrap()),
            _ => (false, n_str.parse::<usize>().unwrap()),
        };

        if neg_sign && n > 0 {
            return NthChildSelector::new(n - 1, true).into();
        }
        NthChildSelector::new(n, false).into()
    }

    fn parse_expr(pair: Pair<'_, Rule>) -> SelectorEnum {
        match pair.as_rule() {
            Rule::childExpr => Self::parse_child(pair.into_inner()),
            Rule::flatExpr => FlatSelector::new().into(),
            Rule::pathExpr => Self::parse_paths(pair.into_inner()),
            Rule::attrExpr => Self::parse_attr(pair.into_inner()),
            Rule::idExpr => Self::parse_id(pair.into_inner()),
            Rule::classExpr => Self::parse_class(pair.into_inner()),
            Rule::textExpr => TextSelector::new().into(),
            Rule::trimExpr => TrimSelector::new().into(),
            Rule::trimPrefixExpr => TrimPrefixSelector::new(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_string(),
            )
            .into(),
            Rule::trimSuffixExpr => TrimSuffixSelector::new(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str()
                    .to_string(),
            )
            .into(),
            Rule::extractAttrExpr => ExtractAttrSelector::new(
                pair.into_inner()
                    .next()
                    .unwrap()
                    .into_inner()
                    .next()
                    .unwrap()
                    .as_str(),
            )
            .into(),
            _ => unreachable!(),
        }
    }

    fn parse_stmt(pairs: Pairs<'_, Rule>) -> Vec<SelectorEnum> {
        pairs
            .into_iter()
            .filter_map(|n| match n.as_rule() {
                Rule::EOI => None,
                _ => Some(Self::parse_expr(n)),
            })
            .collect()
    }
}

/// Parse input as hql defined in [grammar.pest](https://github.com/xylonx/hql/tree/master/src/selector/grammar.pest)
/// and return a series of Selectors.
///
/// Throw pest::error::Error when input does not follow the grammar. It implements Display trait with
/// more readable error like below
///
/// ```
/// --> 1:1
/// |
/// 1 | #child(2)
/// | ^---
/// |
/// = expected flatExpr, pathExpr, attrExpr, idExpr, classExpr, or helperExpr
/// ```
#[allow(clippy::result_large_err)]
pub fn try_parse_hql(input: &str) -> Result<Vec<SelectorEnum>, pest::error::Error<Rule>> {
    Ok(HqlParser::parse_stmt(HqlParser::parse(Rule::hql, input)?))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse() {
        #[rustfmt::skip]
        let cases = vec![
            ("@flat()", vec![FlatSelector::new().into()]),

            ("@path(`/body//div/a`)", vec![PathSelector::new(vec![(Path::Single, "body".into()), (Path::Travel, "div".into()), (Path::Single, "a".into())]).into()]),

            ("@attr(`target`, `_blank`)", vec![AttrSelector::new("target", Some("_blank")).into()]),
            ("@attr(`href`)", vec![AttrSelector::new("href", None).into()]),

            ("@id(`main`)", vec![IDSelector::new("main".into(), true).into()]),
            ("@id(`main`, 1)", vec![IDSelector::new("main".into(), true).into()]),
            ("@id(`main`, 0)", vec![IDSelector::new("main".into(), false).into()]),

            ("@class(`content-body`)", vec![ClassSelector::new("content-body".into(), true).into()]),
            ("@class(`content-body`, 1)", vec![ClassSelector::new("content-body".into(), true).into()]),
            ("@class(`content-body`, 0)", vec![ClassSelector::new("content-body".into(), false).into()]),

            ("#text()", vec![TextSelector::new().into()]),
            ("#trim()", vec![TrimSelector::new().into()]),
            ("#trimPrefix(`hello`)", vec![TrimPrefixSelector::new("hello".into()).into()]),
            ("#trimSuffix(`world`)", vec![TrimSuffixSelector::new("world".into()).into()]),

            ("@child(0)", vec![NthChildSelector::new(0, false).into()]),
            ("@child(-0)", vec![NthChildSelector::new(0, false).into()]),
            ("@child(2)", vec![NthChildSelector::new(2, false).into()]),
            ("@child(-2)", vec![NthChildSelector::new(1, true).into()]),

            ("@flat() | @path(`/body//div/a`) | @attr(`href`) | #text() | #trim()", vec![
                FlatSelector::new().into(),
                PathSelector::new(vec![(Path::Single, "body".into()), (Path::Travel, "div".into()), (Path::Single, "a".into())]).into(),
                AttrSelector::new("href", None).into(),
                TextSelector::new().into(),
                TrimSelector::new().into(),
            ]),
        ];

        for (hql, selectors) in cases {
            let pairs = HqlParser::parse(Rule::hql, hql).unwrap_or_else(|e| panic!("{}", e));
            assert_eq!(HqlParser::parse_stmt(pairs), selectors)
        }
    }
}
