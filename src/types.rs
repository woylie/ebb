use chrono::NaiveDate;
use std::collections::BTreeMap;

pub type Sickdays = BTreeMap<NaiveDate, String>;
