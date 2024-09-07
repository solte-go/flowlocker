use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::borrow::Cow;
use std::collections::HashMap;

pub type Segment<'a> = Cow<'a, str>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Parameter {
    StringArg(String),
    BoolArg(bool),
    IntArg(i32),
}

pub enum Conditions {
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl Conditions {
    fn to_string(&self) -> Cow<'static, str> {
        match self {
            Conditions::Eq => Cow::Borrowed("="),
            Conditions::Gt => Cow::Borrowed(">"),
            Conditions::Gte => Cow::Borrowed(">="),
            Conditions::Lt => Cow::Borrowed("<"),
            Conditions::Lte => Cow::Borrowed("<="),
        }
    }
}

impl Serialize for Parameter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Parameter::StringArg(ref value) => serializer.serialize_str(value),
            Parameter::BoolArg(value) => serializer.serialize_bool(value),
            Parameter::IntArg(value) => serializer.serialize_i32(value),
        }
    }
}

#[derive(Debug)]
enum QueryBuilderInsertExceptions {
    None,
    AndOr,
}

#[derive(Debug)]
pub struct QueryBuilder<'a> {
    segments: Vec<Segment<'a>>,
    params: HashMap<String, Parameter>,
    insert_exceptions: QueryBuilderInsertExceptions,
    and_or_exceptions: bool,
}

impl<'a> Default for QueryBuilder<'a> {
    fn default() -> Self {
        QueryBuilder {
            segments: Vec::new(),
            params: HashMap::new(),
            insert_exceptions: QueryBuilderInsertExceptions::None,
            and_or_exceptions: false,
        }
    }
}
impl<'a> QueryBuilder<'a> {
    pub fn create<T: Into<Segment<'a>>>(mut self, node: T) -> Self {
        self.add_segment_p("CREATE", node);

        self
    }

    pub fn select<T: Into<Segment<'a>>>(mut self, node: T) -> Self {
        self.add_segment_p("SELECT", node);

        self
    }

    fn next_param_index(&self) -> usize {
        self.params.len() + 1
    }

    pub fn from<T>(mut self, param: T) -> Self
    where
        T: From<T>,
        Parameter: From<T>,
    {
        let t = format!("type::table(${})", self.next_param_index());
        let node: Segment = t.into();

        self.add_segment_p("FROM", node);
        self.param(param.into());

        self
    }

    pub fn filter<T, C>(mut self, column: C, conditions: Conditions, param: T) -> Self
    where
        T: From<T>,
        Parameter: From<T>,
        C: ToString,
    {
        let (new_condition, new_parameter) = self.condition(&column, conditions, param);

        let where_segment: Segment = "WHERE".into();

        if self.segments.contains(&where_segment) {
            self.add_segment_p("AND", new_condition);
            self.param(new_parameter);
        } else {
            self.add_segment_p("WHERE", new_condition);
            self.param(new_parameter);
        }

        self
    }

    pub fn and<T, C>(mut self, column: C, conditions: Conditions, param: T) -> Self
    where
        T: From<T>,
        Parameter: From<T>,
        C: ToString,
    {
        let (new_con, p) = self.condition(&column, conditions, param);

        self.add_segment_p("AND", new_con);
        self.param(p);

        self
    }

    pub fn and_or<T, C>(mut self, column: C, conditions: Conditions, param: T) -> Self
    where
        T: From<T>,
        Parameter: From<T>,
        C: ToString,
    {
        let (new_con, p) = self.condition(&column, conditions, param);

        if self.and_or_exceptions {
            self.add_segment_p("OR", new_con);
            self.param(p);
        } else {
            self.add_segment_p("AND", new_con);
            self.param(p);
            self.and_or_exceptions = true
        }

        self
    }

    pub fn or<T, C>(mut self, column: C, conditions: Conditions, param: T) -> Self
    where
        T: From<T>,
        Parameter: From<T>,
        C: ToString,
    {
        let (new_con, p) = self.condition(&column, conditions, param);

        self.add_segment_p("OR", new_con);
        self.param(p);

        self
        // todo checks
    }

    pub fn condition<T, C>(&self, c: &C, con: Conditions, p: T) -> (String, Parameter)
    where
        T: From<T>,
        Parameter: From<T>,
        C: ToString,
    {
        let parameter: Parameter = p.into();

        let placeholder = self.next_param_index();

        let condition = format!("{} {} ${}", c.to_string(), con.to_string(), placeholder);

        (condition, parameter)
    }

    fn add_segment_p<T: Into<Segment<'a>>>(&mut self, prefix: &'a str, segment: T) -> &mut Self {
        self.add_segment(prefix).add_segment(segment)
    }

    pub fn add_segment<T: Into<Segment<'a>>>(&mut self, segment: T) -> &mut Self {
        let into = segment.into();

        if into.is_empty() {
            return self;
        }

        match (&self.insert_exceptions, into.as_ref()) {
            // if the previous segment is already a OR or an AND and the new one is
            // one of the two again, the new one replaces the old one:
            (QueryBuilderInsertExceptions::AndOr, "AND" | "OR") => {
                if let Some(last) = self.segments.last_mut() {
                    *last = into;
                }

                return self;
            }
            (_, "AND" | "OR") => {
                self.insert_exceptions = QueryBuilderInsertExceptions::AndOr;
            }
            _ => {
                self.insert_exceptions = QueryBuilderInsertExceptions::None;
            }
        };

        self.segments.push(into);

        self
    }

    fn generate_key(&self) -> String {
        format!("${}", self.params.len() + 1)
    }

    pub fn param(&mut self, value: Parameter) -> &mut Self {
        self.params
            .insert(self.next_param_index().to_string(), value);

        self
    }

    pub fn build(self) -> super::error::Result<(String, HashMap<String, Parameter>)> {
        let mut output = self.segments.join(" ");

        Ok((output, self.params))
    }
}

impl From<String> for Parameter {
    fn from(s: String) -> Self {
        Parameter::StringArg(s)
    }
}

impl From<&str> for Parameter {
    fn from(s: &str) -> Self {
        Parameter::StringArg(s.to_string())
    }
}

impl From<bool> for Parameter {
    fn from(b: bool) -> Self {
        Parameter::BoolArg(b)
    }
}

impl From<i32> for Parameter {
    fn from(i: i32) -> Self {
        Parameter::IntArg(i)
    }
}

impl From<Parameter> for String {
    fn from(arg: Parameter) -> Self {
        match arg {
            Parameter::StringArg(s) => s,
            _ => panic!("Cannot convert Argument to String"),
        }
    }
}

impl From<Parameter> for bool {
    fn from(arg: Parameter) -> Self {
        match arg {
            Parameter::BoolArg(b) => b,
            _ => panic!("Cannot convert Argument to bool"),
        }
    }
}

impl From<Parameter> for i32 {
    fn from(arg: Parameter) -> Self {
        match arg {
            Parameter::IntArg(i) => i,
            _ => panic!("Cannot convert Argument to i32"),
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn test_new() {
    //     let mut qb = QueryBuilder::new()
    //         .select("*")
    //         .from("type::table($table)")
    //         .filter("app = $app")
    //         .and("app = $app");

    //     let qb = qb.param(
    //         "table",
    //         "$table",
    //     );
    //     let qb = qb.param("app", "$app");
    //     let qb = qb.param("app1", "$app1");

    //     let res = qb.build();

    //     match res {
    //         Ok((query, args)) => {
    //             println!("{:?}", query);
    //             println!("{:?}", args);

    //             let mut test_quety = query;

    //             for (i, arg) in args.into_iter().enumerate() {
    //                 let arg_to_replace = arg.0;
    //                 test_quety = test_quety.replace(arg_to_replace, "Test");
    //             }
    //             println!("{:?}", test_quety);
    //         }
    //         Err(e) => println!("{:?}", e),
    //     }
    // }
}
