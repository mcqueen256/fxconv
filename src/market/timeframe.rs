use std::fmt;


#[derive(PartialEq, Clone)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week
}

/// Display the TimeUnit in human readable terms.
impl fmt::Display for TimeUnit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match *self {
            TimeUnit::Second => "second",
            TimeUnit::Minute => "minute",
            TimeUnit::Hour => "hour",
            TimeUnit::Day => "day",
            TimeUnit::Week => "week"
        })
    }
}
#[derive(Clone)]
pub struct TimeFrame {
    unit: TimeUnit,
    length: usize
}

impl TimeFrame {
    pub fn new(length: usize, unit: TimeUnit) -> TimeFrame {
        TimeFrame { unit: unit, length: length }
    }

    pub fn unit(&self) -> &TimeUnit {
        &self.unit
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

impl fmt::Display for TimeFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TimeFrame( {} {} )", self.length, self.unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeunit_formatting() {
        assert_eq!("second", format!("{}", TimeUnit::Second));
        assert_eq!("minute", format!("{}", TimeUnit::Minute));
        assert_eq!("hour", format!("{}", TimeUnit::Hour));
        assert_eq!("day", format!("{}", TimeUnit::Day));
        assert_eq!("week", format!("{}", TimeUnit::Week));
    }

    #[test]
    fn timeframe_len() {
        let tf = TimeFrame::new(45, TimeUnit::Minute);
        assert_eq!(tf.len(), 45);
    }

    #[test]
    fn timeframe_unit() {
        let tf = TimeFrame::new(45, TimeUnit::Minute);
        assert!(*tf.unit() == TimeUnit::Minute);
    }
}
