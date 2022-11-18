use std::str::FromStr;

use regex::Regex;
type Rule = (Regex, bool);
#[derive(Debug)]
pub struct Filter {
    rules: Vec<Rule>,
}

impl Filter {
    pub fn is_match(&self, s: &str) -> bool {
        for rule in &self.rules {
            if rule.0.is_match(s) != rule.1 {
                return false;
            }
        }
        true
    }
}

impl FromStr for Filter {
    type Err = regex::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rules: Result<Vec<Rule>, Self::Err> = s
            .split(',')
            .map(|mut x| {
                let exclude = x.starts_with('!');
                if exclude || x.starts_with("\\!") {
                    x = &x[1..]
                }
                let r = Regex::new(x)?;
                Ok((r, !exclude))
            })
            .collect();
        Ok(Filter { rules: rules? })
    }
}
