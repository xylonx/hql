use std::fmt::Debug;

use tracing::info;

use crate::{
    html::{ElementOrTextRef, Html},
    selector::{self, Rule, Selector, SelectorEnum},
};

#[derive(Debug)]
pub struct Querier {
    pub selectors: Vec<SelectorEnum>,
}

impl Querier {
    #[allow(clippy::result_large_err)]
    pub fn try_parse(hql: &str) -> Result<Self, pest::error::Error<Rule>> {
        Ok(Self {
            selectors: selector::try_parse_hql(hql)?,
        })
    }

    pub fn new(selectors: Vec<SelectorEnum>) -> Self {
        Self { selectors }
    }

    pub fn add_selector(&mut self, s: SelectorEnum) {
        self.selectors.push(s);
    }

    pub fn query_document<'a, 'b: 'a>(&'b self, doc: &'a Html) -> Vec<ElementOrTextRef<'a>> {
        let mut nodes = vec![doc.root()];

        for s in &self.selectors {
            info!("apply selector: {:?}", s);
            nodes = nodes
                .into_iter()
                .flat_map(|n| s.select(n))
                .collect::<Vec<_>>();
        }

        nodes
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_parse() {}
}
