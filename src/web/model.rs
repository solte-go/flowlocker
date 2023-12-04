use std::borrow::Cow;
use serde::{Deserialize, Serialize, Serializer};
use chrono::{DateTime, Duration ,Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Operation {
    pub name: Cow<'static, str>,
    pub in_progress: bool,
    #[serde(with = "op_duration")]
    pub timeout: Duration,
    #[serde(with = "op_date_format")]
    pub started_at: DateTime<Utc>,
    #[serde(with = "op_date_format")]
    pub finished_at: DateTime<Utc>
}
#[derive(Serialize, Deserialize)]
pub struct OperationResponse{
    pub name: String,
    pub in_progress: bool,
    pub started_at: DateTime<Utc>,
    pub timeout: String,
}

// #[serde(serialize_with = "serialize_dt")]
// pub fn serialize_dt<S>(dt: &Option<Duration<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
// {
//     if let Some(dt) = dt {
//         dt.format("%m/%d/%Y %H:%M")
//             .to_string()
//             .serialize(serializer)
//     } else {
//         serializer.serialize_none()
//     }
// }

impl Operation {
  pub fn next_timeout(self) -> String {
        let timeout = self.timeout;
        let now = timeout;
        now.to_string()
    }
}

impl From<Operation> for OperationResponse{
    fn from(value: Operation) -> Self {
        Self {name: value.name.to_string(),
            in_progress: value.in_progress,
            started_at: value.started_at,
            timeout: value.next_timeout()
        }
    }
}

mod op_date_format {
    use chrono::{DateTime, Utc, NaiveDateTime};
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)?;
        Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
    }
}

mod op_duration {
    use chrono::Duration as ChronoDuration;
    use parse_duration;
    use serde::{self, Deserialize, Serializer, Deserializer};


    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(
        date: &ChronoDuration,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let s = format!("{}", date.num_seconds().to_string());
        println!("{:?}", date.num_seconds().to_string());
        
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<ChronoDuration, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        let std_duration = parse_duration::parse(s.as_str());
        match std_duration {
            Ok(d) => Ok(ChronoDuration::seconds(std_duration.unwrap().as_secs() as i64)),
            Err(_) => {
                let s: &'static str = "unknown error";
                Err(serde::de::Error::missing_field(s))
            }
            }
    }
}

