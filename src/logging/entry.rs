// Copyright (c) 2023 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use chrono::{DateTime, Utc};
use log::{Level, Record};
use serde::{Serialize, Serializer};

#[derive(Serialize)]
pub struct BriefEntry {
    #[serde(rename = "ts", serialize_with = "serialize_date_time")]
    time: DateTime<Utc>,
    #[serde(rename = "tgt")]
    target: String,
    #[serde(rename = "l", serialize_with = "serialize_level")]
    level: Level,
    #[serde(rename = "msg")]
    message: String,
}

impl BriefEntry {
    pub fn new(record: &Record) -> Self {
        Self {
            time: Utc::now(),
            target: String::from(record.target()),
            level: record.level(),
            message: record.args().to_string(),
        }
    }
}

#[derive(Serialize)]
pub struct DetailedEntry {
    #[serde(rename = "ts", serialize_with = "serialize_date_time")]
    time: DateTime<Utc>,
    #[serde(rename = "tgt")]
    target: String,
    #[serde(rename = "lev", serialize_with = "serialize_level")]
    level: Level,
    #[serde(rename = "msg")]
    message: String,
    #[serde(rename = "fn")]
    file: Option<String>,
    #[serde(rename = "ln")]
    line: Option<u32>,
}

impl DetailedEntry {
    pub fn new(record: &Record) -> Self {
        Self {
            time: Utc::now(),
            target: String::from(record.target()),
            level: record.level(),
            message: record.args().to_string(),
            file: record.file().map(str::to_string),
            line: record.line(),
        }
    }
}

fn serialize_date_time<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_rfc3339())
}

fn serialize_level<S>(value: &Level, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value.as_str())
}
