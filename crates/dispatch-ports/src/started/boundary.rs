use std::fmt;
use std::str::FromStr;

use grammar::lstart::Stamp;
use named_derive::Named;
use newtype::Spelled;

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Weekday {
    #[name("Mon")]
    Monday,
    #[name("Tue")]
    Tuesday,
    #[name("Wed")]
    Wednesday,
    #[name("Thu")]
    Thursday,
    #[name("Fri")]
    Friday,
    #[name("Sat")]
    Saturday,
    #[name("Sun")]
    Sunday,
}

#[derive(Named, Clone, Copy, Debug, PartialEq, Eq)]
enum Month {
    #[name("Jan")]
    January,
    #[name("Feb")]
    February,
    #[name("Mar")]
    March,
    #[name("Apr")]
    April,
    #[name("May")]
    May,
    #[name("Jun")]
    June,
    #[name("Jul")]
    July,
    #[name("Aug")]
    August,
    #[name("Sep")]
    September,
    #[name("Oct")]
    October,
    #[name("Nov")]
    November,
    #[name("Dec")]
    December,
}

#[derive(Spelled, Clone, Debug, PartialEq, Eq)]
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

impl Started {
    fn read(text: &str) -> Option<Started> {
        let stamp = Stamp::read(text)?;
        let [weekday, month] = stamp.words();
        let [day, hour, minute, second, year] = stamp.numbers();
        Some(Started {
            weekday: weekday.parse().ok()?,
            month: month.parse().ok()?,
            day: u8::try_from(day).ok()?,
            hour: u8::try_from(hour).ok()?,
            minute: u8::try_from(minute).ok()?,
            second: u8::try_from(second).ok()?,
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
