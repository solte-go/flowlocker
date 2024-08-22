use std::borrow::Cow;
use std::collections::HashMap;

pub type Segment<'a> = Cow<'a, str>;

#[derive(Debug)]
struct QueryBuilder<'a> {
    segments: Vec<Segment<'a>>,
    params:HashMap<&'a str,  &'a str>
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

  pub fn from<T: Into<Segment<'a>>>(mut self, node: T) -> Self {
    self.add_segment_p("FROM", node);

    self
  }

  pub fn filter<T: Into<Segment<'a>>>(mut self, condition: T) -> Self {
    self.add_segment_p("WHERE", condition);

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

  pub fn param(mut self, key: &'a str, value: &'a str) -> Self {
    self.params.insert(key, value);

    self
  }

  pub fn build(self) -> String {
    let mut output = self.segments.join(" ");

    for (key, value) in self.params {
      let key_size = key.len();

      while let Some(index) = output.find(key) {
        output.replace_range(index..index + key_size, value);
      }
    }

    output
  }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new(){
        let mut qb = QueryBuilder::new()
            .select("*")
            .from("test")
            .filter(condition)
            .build();

        println!("{:?}", qb)

    }
}
