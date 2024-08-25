use super::error::Result;
use std::borrow::Cow;
use std::collections::HashMap;

pub type Segment<'a> = Cow<'a, str>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Argument {
    StringArg(String),
    BoolArg(bool),
    IntArg(i32),
}
#[derive(Debug)]
pub struct QueryBuilder<'a> {
    segments: Vec<Segment<'a>>,
    params: HashMap<String, Argument>,
}

impl<'a> QueryBuilder<'a> {
    pub fn new() -> Self {
        QueryBuilder {
            segments: Vec::new(),
            params: HashMap::new(),
        }
    }

    pub fn create<T: Into<Segment<'a>>>(mut self, node: T) -> Self {
        self.add_segment_p("CREATE", node);

        self
    }

    pub fn select<T: Into<Segment<'a>>>(mut self, node: T) -> Self {
        self.add_segment_p("SELECT", node);

        self
    }

    pub fn from<T: Into<Segment<'a>>>(mut self, node: T, param: Argument) -> Self {
        self.add_segment_p("FROM", node);
        self.param(param);

        self
    }

    pub fn filter<T: Into<Segment<'a>>>(mut self, condition: T, param: Argument) -> Self {
        self.add_segment_p("WHERE", condition);;
        self.param(param);

        self
    }

    pub fn and<T: Into<Segment<'a>>>(mut self, condition: T, param: Argument) -> Self {
        self.add_segment_p("AND", condition);
        self.param(param);

        self
    }

    fn add_segment_p<T: Into<Segment<'a>>>(&mut self, prefix: &'a str, segment: T) -> &mut Self {
        self.add_segment(prefix).add_segment(segment)
    }

    pub fn add_segment<T: Into<Segment<'a>>>(&mut self, segment: T) -> &mut Self {
        let into = segment.into();

        if into.is_empty() {
            return self;
        }

        // match (&self.insert_exceptions, into.as_ref()) {
        //   // if the previous segment is already a OR or an AND and the new one is
        //   // one of the two again, the new one replaces the old one:
        //   (QueryBuilderInsertExceptions::AndOr, "AND" | "OR") => {
        //     if let Some(last) = self.segments.last_mut() {
        //       *last = into;
        //     }

        //     return self;
        //   }
        //   (_, "AND" | "OR") => {
        //     self.insert_exceptions = QueryBuilderInsertExceptions::AndOr;
        //   }
        //   _ => {
        //     self.insert_exceptions = QueryBuilderInsertExceptions::None;
        //   }
        // };

        self.segments.push(into);

        self
    }

    fn generate_key(&self) -> String {
        format!("${}", self.params.len() + 1)
    }

    pub fn param(&mut self, value: Argument) -> &mut Self {
        self.params.insert(self.generate_key(), value);

        self
    }

    pub fn build(self) -> Result<(String, usize)> {
        let mut output = self.segments.join(" ");

        let count_placeholders = Self::count_placeholders(&output);
        Ok((output, count_placeholders))
        // if count_placeholders == self.params.len() {
        //     // for (i, arg) in args.into_iter().enumerate() {
        //     //     let arg_to_replace = format!("${}", i + 1);
        //     //     output = output.replace(&arg_to_replace, arg);
        //     // }
        //
        //     Ok(output)
        // } else {
        //     println!("{:?}", self.params.len());
        //
        //     Err(Error::Placeholder(
        //         "Number of placeholders does not match the number of arguments".to_string(),
        //     ))
        // }

        // for (key, value) in self.params {
        //     let key_size = key.len();

        //     while let Some(index) = output.find(key) {
        //         output.replace_range(index..index + key_size, value);
        //     }
    }

    fn count_placeholders(query: &str) -> usize {
        query.matches("$").count()
    }
}

impl From<String> for Argument {
    fn from(s: String) -> Self {
        Argument::StringArg(s)
    }
}

impl From<&str> for Argument {
    fn from(s: &str) -> Self {
        Argument::StringArg(s.to_string())
    }
}

impl From<bool> for Argument {
    fn from(b: bool) -> Self {
        Argument::BoolArg(b)
    }
}

impl From<i32> for Argument {
    fn from(i: i32) -> Self { Argument::IntArg(i) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut qb = QueryBuilder::new()
            .select("*")
            .from("type::table($table)")
            .filter("app = $app")
            .and("app = $app");

        let qb = qb.param(
            "table",
            "$table",
        );
        let qb = qb.param("app", "$app");
        let qb = qb.param("app1", "$app1");

        let res = qb.build();

        match res {
            Ok((query, args)) => {
                println!("{:?}", query);
                println!("{:?}", args);

                let mut test_quety = query;

                for (i, arg) in args.into_iter().enumerate() {
                    let arg_to_replace = arg.0;
                    test_quety = test_quety.replace(arg_to_replace, "Test");
                }
                println!("{:?}", test_quety);
            }
            Err(e) => println!("{:?}", e),
        }
    }
}
