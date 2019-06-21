use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub struct TimestampParseError(());

impl fmt::Display for TimestampParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid timestamp syntax")
    }
}

impl std::error::Error for TimestampParseError {}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Timestamp(Inner);

impl Timestamp {
    pub(crate) fn new(y: u16, m: Option<u8>, d: Option<u8>,
        h: Option<u8>, mi: Option<u8>, s: Option<u8>) -> Option<Self>
    {
        let m = if let Some(m) = m {
            m
        } else {
            assert!(d.is_none() && h.is_none() && mi.is_none() && s.is_none());
            return Self::new_y(y);
        };

        let d = if let Some(d) = d {
            d
        } else {
            assert!(h.is_none() && mi.is_none() && s.is_none());
            return Self::new_ym(y, m);
        };

        let h = if let Some(h) = h {
            h
        } else {
            assert!(mi.is_none() && s.is_none());
            return Self::new_ymd(y, m, d);
        };


        let mi = if let Some(mi) = mi {
            mi
        } else {
            assert!(s.is_none());
            return Self::new_ymdh(y, m, d, h);
        };

        let s = if let Some(s) = s {
            s
        } else {
            return Self::new_ymdhm(y, m, d, h, mi);
        };

        Self::new_ymdhms(y, m, d, h, mi, s)
    }

    pub(crate) fn new_ymdhms(y: u16, m: u8, d: u8, h: u8, mi: u8, s: u8) -> Option<Self> {
        Some(Self(Inner::Ymdhms {
            y: Self::check_y(y)?,
            m: Self::check_m(m)?,
            d: Self::check_d(d)?,
            h: Self::check_h(h)?,
            mi: Self::check_mi(mi)?,
            s: Self::check_s(s)?,
        }))
    }

    pub(crate) fn new_ymdhm(y: u16, m: u8, d: u8, h: u8, mi: u8) -> Option<Self> {
        Some(Self(Inner::Ymdhm {
            y: Self::check_y(y)?,
            m: Self::check_m(m)?,
            d: Self::check_d(d)?,
            h: Self::check_h(h)?,
            mi: Self::check_mi(mi)?,
        }))
    }

    pub(crate) fn new_ymdh(y: u16, m: u8, d: u8, h: u8) -> Option<Self> {
        Some(Self(Inner::Ymdh {
            y: Self::check_y(y)?,
            m: Self::check_m(m)?,
            d: Self::check_d(d)?,
            h: Self::check_h(h)?,
        }))
    }

    pub(crate) fn new_ymd(y: u16, m: u8, d: u8) -> Option<Self> {
        Some(Self(Inner::Ymd {
            y: Self::check_y(y)?,
            m: Self::check_m(m)?,
            d: Self::check_d(d)?,
        }))
    }

    pub(crate) fn new_ym(y: u16, m: u8) -> Option<Self> {
        Some(Self(Inner::Ym {
            y: Self::check_y(y)?,
            m: Self::check_m(m)?,
        }))
    }

    pub(crate) fn new_y(y: u16) -> Option<Self> {
        Some(Self(Inner::Y {
            y: Self::check_y(y)?,
        }))
    }

    pub fn year(&self) -> u16 {
        use Inner::*;
        match self.0 {
            | Ymdhms { y, .. }
            | Ymdhm { y, .. }
            | Ymdh { y, .. }
            | Ymd { y, .. }
            | Ym { y, .. }
            | Y { y, .. }
            => y,
        }
    }

    pub fn month(&self) -> Option<u8> {
        use Inner::*;
        match self.0 {
            | Ymdhms { m, .. }
            | Ymdhm { m, .. }
            | Ymdh { m, .. }
            | Ymd { m, .. }
            | Ym { m, .. }
            => Some(m),
            | Y { .. }
            => None,
        }
    }

    pub fn day(&self) -> Option<u8> {
        use Inner::*;
        match self.0 {
            | Ymdhms { d, .. }
            | Ymdhm { d, .. }
            | Ymdh { d, .. }
            | Ymd { d, .. }
            => Some(d),
            | Ym { .. }
            | Y { .. }
            => None,
        }
    }

    pub fn hour(&self) -> Option<u8> {
        use Inner::*;
        match self.0 {
            | Ymdhms { h, .. }
            | Ymdhm { h, .. }
            | Ymdh { h, .. }
            => Some(h),
            | Ymd { .. }
            | Ym { .. }
            | Y { .. }
            => None,
        }
    }

    pub fn minute(&self) -> Option<u8> {
        use Inner::*;
        match self.0 {
            | Ymdhms { mi, .. }
            | Ymdhm { mi, .. }
            => Some(mi),
            | Ymdh { .. }
            | Ymd { .. }
            | Ym { .. }
            | Y { .. }
            => None,
        }
    }

    pub fn second(&self) -> Option<u8> {
        use Inner::*;
        match self.0 {
            | Ymdhms { s, .. }
            => Some(s),
            | Ymdhm { .. }
            | Ymdh { .. }
            | Ymd { .. }
            | Ym { .. }
            | Y { .. }
            => None,
        }
    }

    fn check_y(y: u16) -> Option<u16> {
        Some(y).filter(|&v| v <= 9999)
    }

    fn check_m(m: u8) -> Option<u8> {
        Some(m).filter(|&v| v >= 1 && v <= 12)
    }

    fn check_d(d: u8) -> Option<u8> {
        Some(d).filter(|&v| v >= 1 && v <= 31)
    }

    fn check_h(h: u8) -> Option<u8> {
        Some(h).filter(|&v| v <= 23)
    }

    fn check_mi(mi: u8) -> Option<u8> {
        Some(mi).filter(|&v| v <= 59)
    }

    fn check_s(s: u8) -> Option<u8> {
        Some(s).filter(|&v| v <= 59)
    }
}

impl FromStr for Timestamp {
    type Err = TimestampParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse<T>(s: Option<&str>) -> Result<Option<T>, TimestampParseError>
            where T: FromStr + Ord
        {
            if let Some(s) = s {
                s.parse::<T>().map(Some).map_err(|_| TimestampParseError(()))
            } else {
                Ok(None)
            }
        }

        let mut it = s.splitn(2, 'T');
        let date = it.next().unwrap();
        let time = it.next().filter(|s| !s.is_empty());

        let mut it = date.splitn(3, '-');
        let y = parse::<u16>(it.next())?.unwrap();
        let m = parse::<u8>(it.next())?;
        let d = parse::<u8>(it.next())?;

        let (h, mi, s) = if let Some(time) = time {
            let mut it = time.splitn(3, ':');
            (parse::<u8>(it.next())?,
                parse::<u8>(it.next())?,
                parse::<u8>(it.next())?)
        } else {
            (None, None, None)
        };

        Self::new(y, m, d, h, mi, s).ok_or(TimestampParseError(()))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Inner::*;
        match self.0 {
            Ymdhms { y, m, d, h, mi, s } => write!(f, "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
                y, m, d, h, mi, s),
            Ymdhm { y, m, d, h, mi }  => write!(f, "{:04}-{:02}-{:02}T{:02}:{:02}", y, m, d, h, mi),
            Ymdh { y, m, d, h }  => write!(f, "{:04}-{:02}-{:02}T{:02}", y, m, d, h),
            Ymd { y, m, d }  => write!(f, "{:04}-{:02}-{:02}", y, m, d),
            Ym { y, m }  => write!(f, "{:04}-{:02}", y, m),
            Y { y } => write!(f, "{:04}", y),
        }
    }
}

impl fmt::Debug for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Timestamp({})", self)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Inner {
    Ymdhms { y: u16, m: u8, d: u8, h: u8, mi: u8, s: u8 },
    Ymdhm { y: u16, m: u8, d: u8, h: u8, mi: u8 },
    Ymdh { y: u16, m: u8, d: u8, h: u8 },
    Ymd { y: u16, m: u8, d: u8 },
    Ym { y: u16, m: u8 },
    Y { y: u16 },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new() {
        assert!(Timestamp::new_y(0).is_some());
        assert!(Timestamp::new_y(9999).is_some());
        assert!(Timestamp::new_y(10000).is_none());

        assert!(Timestamp::new_ym(2019, 1).is_some());
        assert!(Timestamp::new_ym(2019, 12).is_some());
        assert!(Timestamp::new_ym(2019, 0).is_none());
        assert!(Timestamp::new_ym(2019, 13).is_none());

        assert!(Timestamp::new_ymd(2019, 2, 1).is_some());
        assert!(Timestamp::new_ymd(2019, 2, 31).is_some());
        assert!(Timestamp::new_ymd(2019, 2, 0).is_none());
        assert!(Timestamp::new_ymd(2019, 2, 32).is_none());

        assert!(Timestamp::new_ymdh(2019, 2, 1, 0).is_some());
        assert!(Timestamp::new_ymdh(2019, 2, 1, 23).is_some());
        assert!(Timestamp::new_ymdh(2019, 2, 1, 24).is_none());

        assert!(Timestamp::new_ymdhm(2019, 2, 1, 0, 0).is_some());
        assert!(Timestamp::new_ymdhm(2019, 2, 1, 0, 59).is_some());
        assert!(Timestamp::new_ymdhm(2019, 2, 1, 0, 60).is_none());

        assert!(Timestamp::new_ymdhms(2019, 2, 1, 0, 0, 0).is_some());
        assert!(Timestamp::new_ymdhms(2019, 2, 1, 0, 0, 59).is_some());
        assert!(Timestamp::new_ymdhms(2019, 2, 1, 0, 0, 60).is_none());
    }

    #[test]
    fn from_str() {
        fn p(s: &str) -> Result<Timestamp, TimestampParseError> {
            s.parse()
        }

        assert!(p("").is_err());
        assert!(p(" 0 ").is_err());
        assert!(p("0-0").is_err());
        assert!(p("0-1-0").is_err());
        assert!(p("0-1-1T24").is_err());
        assert!(p("0-1-1T23:60").is_err());
        assert!(p("0-1-1T23:59:60").is_err());

        assert_eq!(p("0").unwrap(), Timestamp::new_y(0).unwrap());
        assert_eq!(p("0-1").unwrap(), Timestamp::new_ym(0, 1).unwrap());
        assert_eq!(p("0-1-1").unwrap(), Timestamp::new_ymd(0, 1, 1).unwrap());
        assert_eq!(p("0-1-1T").unwrap(), Timestamp::new_ymd(0, 1, 1).unwrap());
        assert_eq!(p("0-1-1T0").unwrap(), Timestamp::new_ymdh(0, 1, 1, 0).unwrap());
        assert_eq!(p("0-1-1T0:0").unwrap(), Timestamp::new_ymdhm(0, 1, 1, 0, 0).unwrap());
        assert_eq!(p("0-1-1T0:0:0").unwrap(), Timestamp::new_ymdhms(0, 1, 1, 0, 0, 0).unwrap());
        assert_eq!(p("9999-12-31T23:59:59").unwrap(),
            Timestamp::new_ymdhms(9999, 12, 31, 23, 59, 59).unwrap());
        assert_eq!(p("9999-02-31T23:59:59").unwrap(),
            Timestamp::new_ymdhms(9999, 2, 31, 23, 59, 59).unwrap());
        assert_eq!(p("2019-06-20T11:06:01").unwrap(),
            Timestamp::new_ymdhms(2019, 6, 20, 11, 6, 1).unwrap());
    }

    #[test]
    fn display() {
        assert_eq!(Timestamp::new_ymdhms(2019, 6, 20, 11, 6, 1).unwrap().to_string(), "2019-06-20T11:06:01");
        assert_eq!(Timestamp::new_ymdh(2019, 6, 20, 11).unwrap().to_string(), "2019-06-20T11");
        assert_eq!(Timestamp::new_ymd(2019, 6, 20).unwrap().to_string(), "2019-06-20");
        assert_eq!(Timestamp::new_ym(2019, 6).unwrap().to_string(), "2019-06");
        assert_eq!(Timestamp::new_y(2019).unwrap().to_string(), "2019");
    }
}