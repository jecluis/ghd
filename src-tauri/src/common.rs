// Copyright 2023 Joao Eduardo Luis <joao@abysmo.io>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use chrono::TimeZone;

pub fn datetime_to_ts(datetime: &str) -> i64 {
    if datetime.is_empty() {
        panic!("Provided datetime is empty!");
    }

    let parsed = match chrono::DateTime::parse_from_rfc3339(datetime) {
        Ok(res) => res,
        Err(err) => {
            panic!("Error parsing datetime '{}': {}", datetime, err);
        }
    };
    return parsed.timestamp();
}

/// Transforms an optional `chrono::DateTime` to an optional `i64` timestamp.
///
pub fn dt_opt_to_ts(dt: &Option<chrono::DateTime<chrono::Utc>>) -> Option<i64> {
    match &dt {
        None => None,
        Some(v) => Some(v.timestamp()),
    }
}

pub fn ts_to_datetime(ts: i64) -> Result<chrono::DateTime<chrono::Utc>, ()> {
    if !ts.is_positive() {
        return Err(());
    }

    Ok(chrono::Utc.timestamp_opt(ts, 0).unwrap())
}

pub fn has_expired(t: &chrono::DateTime<chrono::Utc>, secs: i64) -> bool {
    let now = chrono::Utc::now();
    let dt = match t.checked_add_signed(chrono::Duration::seconds(secs)) {
        Some(v) => v,
        None => now,
    };
    return dt < now;
}
