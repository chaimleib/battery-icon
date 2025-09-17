use crate::tag;
use std::error::Error;

pub trait StackMatcher {
    fn matches(&self, stack: &Vec<tag::Tag>) -> bool;
}

pub struct IdMatcher {
    pub id: String,
}

impl StackMatcher for IdMatcher {
    fn matches(&self, stack: &Vec<tag::Tag>) -> bool {
        let Some(last) = stack.last() else {
            return false;
        };
        last.id == self.id
    }
}

pub struct NameMatcher {
    pub name: String,
}

impl StackMatcher for NameMatcher {
    fn matches(&self, stack: &Vec<tag::Tag>) -> bool {
        let Some(last) = stack.last() else {
            return false;
        };
        last.name == self.name
    }
}

pub struct AndMatcher {
    pub matchers: Vec<Box<dyn StackMatcher>>,
}

impl StackMatcher for AndMatcher {
    fn matches(&self, stack: &Vec<tag::Tag>) -> bool {
        for m in &self.matchers {
            if !m.matches(stack) {
                return false;
            }
        }
        true
    }
}

pub fn new_tag_matcher(spec: &str) -> Result<AndMatcher, Box<dyn Error>> {
    let (name, id) = spec.split_once("#").unwrap_or((spec, ""));
    let mut result = AndMatcher {
        matchers: Vec::new(),
    };
    if name != "" {
        result.matchers.push(Box::new(NameMatcher {
            name: name.to_string(),
        }));
    }
    if id != "" {
        result
            .matchers
            .push(Box::new(IdMatcher { id: id.to_string() }));
    }
    if result.matchers.len() == 0 {
        return Err("new_tag_matcher: failed to parse spec".into());
    }
    Ok(result)
}
