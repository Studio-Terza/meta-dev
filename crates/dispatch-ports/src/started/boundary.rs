use std::fmt;
use std::str::FromStr;

use grammar_lstart::Stamp;
use invariants::Ignoring;
use newtype::{Listed, Spelled};
use strum::{Display, IntoStaticStr, VariantArray};

use crate::Ignored;

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
enum Weekday {
    #[strum(serialize = "Mon")]
    Monday,
    #[strum(serialize = "Tue")]
    Tuesday,
    #[strum(serialize = "Wed")]
    Wednesday,
    #[strum(serialize = "Thu")]
    Thursday,
    #[strum(serialize = "Fri")]
    Friday,
    #[strum(serialize = "Sat")]
    Saturday,
    #[strum(serialize = "Sun")]
    Sunday,
}

#[derive(Display, IntoStaticStr, VariantArray, Listed, Clone, Copy, Debug, PartialEq, Eq)]
enum Month {
    #[strum(serialize = "Jan")]
    January,
    #[strum(serialize = "Feb")]
    February,
    #[strum(serialize = "Mar")]
    March,
    #[strum(serialize = "Apr")]
    April,
    #[strum(serialize = "May")]
    May,
    #[strum(serialize = "Jun")]
    June,
    #[strum(serialize = "Jul")]
    July,
    #[strum(serialize = "Aug")]
    August,
    #[strum(serialize = "Sep")]
    September,
    #[strum(serialize = "Oct")]
    October,
    #[strum(serialize = "Nov")]
    November,
    #[strum(serialize = "Dec")]
    December,
}

#[derive(Spelled, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(try_from = "String")]
pub struct Started {
    weekday: Weekday,
    month: Month,
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    year: u16,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{0:?} is not a process start time, which reads as in Fri Nov  6 01:16:00 2026")]
pub struct Unstarted(String);

impl TryFrom<String> for Started {
    type Error = Unstarted;

    fn try_from(text: String) -> Result<Started, Unstarted> {
        text.parse()
    }
}

impl Started {
    fn read(text: &str) -> Option<Started> {
        let stamp = Stamp::try_from(text).ignoring(Ignored::NotAStamp)?;
        let [weekday, month] = stamp.words();
        let [day, hour, minute, second, year] = stamp.numbers();
        Some(Started {
            weekday: weekday.parse().ignoring(Ignored::NotAStamp)?,
            month: month.parse().ignoring(Ignored::NotAStamp)?,
            day: u8::try_from(day).ignoring(Ignored::NotAStamp)?,
            hour: u8::try_from(hour).ignoring(Ignored::NotAStamp)?,
            minute: u8::try_from(minute).ignoring(Ignored::NotAStamp)?,
            second: u8::try_from(second).ignoring(Ignored::NotAStamp)?,
            year,
        })
    }
}

impl FromStr for Started {
    type Err = Unstarted;

    fn from_str(text: &str) -> Result<Started, Unstarted> {
        Started::read(text).ok_or_else(|| Unstarted(text.to_owned()))
    }
}

impl fmt::Display for Started {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} {:>2} {:02}:{:02}:{:02} {}",
            self.weekday, self.month, self.day, self.hour, self.minute, self.second, self.year
        )
    }
}
