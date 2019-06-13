use std::fmt;

macro_rules! frame_id {
    ($s:expr) => {
        FrameId::new([$s[0], $s[1], $s[2], $s[3]])
    };
}

macro_rules! frame_id_short {
    ($s:expr) => {
        FrameId::new_short([$s[0], $s[1], $s[2]])
    };
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FrameId(u32);

impl FrameId {
    pub const NULL: Self = Self::new([0; 4]);

    // Text frames.

    pub const BPM: Self = frame_id!(b"TBPM");
    pub const ALBUM: Self = frame_id!(b"TALB");
    pub const COMPOSER: Self = frame_id!(b"TCOM");
    pub const CONTENT_TYPE: Self = frame_id!(b"TCON");
    pub const COPYRIGHT: Self = frame_id!(b"TCOP");
    pub const ENCODED_TS: Self = frame_id!(b"TDEN");
    pub const DELAY: Self = frame_id!(b"TDLY");
    pub const ORIGINAL_RELEASE_TS: Self = frame_id!(b"TDOR");

    /// `TDRC`. While it is called "recording timestamp" in the RFC, de-facto it's used as release
    /// timestamp everywhere.
    pub const RELEASE_TS: Self = frame_id!(b"TDRC");

    pub const RELEASE_TS_RFC: Self = frame_id!(b"TDRL");
    pub const TAGGING_TS: Self = frame_id!(b"TDRL");
    pub const ENCODED_BY: Self = frame_id!(b"TENC");
    pub const LYRICIST: Self = frame_id!(b"TEXT");
    pub const FILE_TYPE: Self = frame_id!(b"TFLT");
    pub const GROUP_TITLE: Self = frame_id!(b"TIT1");
    pub const TITLE: Self = frame_id!(b"TIT2");
    pub const SUBTITLE: Self = frame_id!(b"TIT3");
    pub const PEOPLE: Self = frame_id!(b"TIPL");
    pub const INITIAL_KEY: Self = frame_id!(b"TKEY");
    pub const LANGUAGE: Self = frame_id!(b"TLAN");
    pub const LENGTH: Self = frame_id!(b"TLEN");
    pub const PERFORMERS: Self = frame_id!(b"TMCL");
    pub const MEDIA_TYPE: Self = frame_id!(b"TMED");
    pub const MOOD: Self = frame_id!(b"TMOO");
    pub const ORIGINAL_ALBUM: Self = frame_id!(b"TOAL");
    pub const ORIGINAL_FILENAME: Self = frame_id!(b"TOFN");
    pub const ORIGINAL_LYRICIST: Self = frame_id!(b"TOLY");
    pub const ORIGINAL_ARTIST: Self = frame_id!(b"TOPE");
    /// v2.3
    pub const ORIGINAL_RELEASE_YEAR: Self = frame_id!(b"TORY");
    pub const OWNER: Self = frame_id!(b"TOWN");
    pub const ARTIST: Self = frame_id!(b"TPE1");
    pub const ALBUM_ARTIST: Self = frame_id!(b"TPE2");
    pub const CONDUCTOR: Self = frame_id!(b"TPE3");
    pub const DISC: Self = frame_id!(b"TPOS");
    pub const PRODUCED_NOTICE: Self = frame_id!(b"TPRO");
    pub const PUBLISHER: Self = frame_id!(b"TPUB");
    pub const RADIO_STATION: Self = frame_id!(b"TRSN");
    pub const RADIO_STATION_NAME: Self = frame_id!(b"TRSO");
    pub const ISRC: Self = frame_id!(b"TSRC");
    pub const DISC_SUBTITLE: Self = frame_id!(b"TSST");
    pub const TRACK: Self = frame_id!(b"TRCK");
    pub const ALBUM_SORT_ORDER: Self = frame_id!(b"TSOA");
    pub const ARTIST_SORT_ORDER: Self = frame_id!(b"TSOP");
    pub const TITLE_SORT_ORDER: Self = frame_id!(b"TSOT");
    pub const ENCODER_SETTINGS: Self = frame_id!(b"TSSE");
    pub const USER_TEXT: Self = frame_id!(b"TXXX");

    // URL frames.

    pub const USER_WEB_LINK: Self = frame_id!(b"WXXX");

    // Misc frames.

    pub const CHAPTER: Self = frame_id!(b"CHAP");
    pub const COMMENT: Self = frame_id!(b"COMM");
    pub const COMMENT_SHORT: Self = frame_id_short!(b"COM");

    pub const fn new(v: [u8; 4]) -> Self {
        Self((v[0] as u32) << 24 |
            (v[1] as u32) << 16 |
            (v[2] as u32) << 8 |
            (v[3] as u32) << 0)
    }

    pub const fn new_short(v: [u8; 3]) -> Self {
        Self::new([v[0], v[1], v[2], 0])
    }

    pub fn to_bytes(&self) -> [u8; 4] {
        [
            (self.0 >> 24) as u8,
            (self.0 >> 16) as u8,
            (self.0 >> 8) as u8,
            (self.0 >> 0) as u8,
        ]
    }

    pub fn is_short(&self) -> bool {
        self.0 & 0xff == 0
    }

    pub fn text(sub_id: [u8; 3]) -> Self {
        Self::prefixed(b'T', sub_id)
    }

    pub fn is_text(&self) -> bool {
        self.is_prefixed(b'T') && *self != Self::USER_TEXT
    }

    pub fn web_link(sub_id: [u8; 3]) -> Self {
        Self::prefixed(b'W', sub_id)
    }

    pub fn is_web_link(&self) -> bool {
        self.is_prefixed(b'W') && *self != Self::USER_WEB_LINK
    }

    fn prefixed(prefix: u8, sub_id: [u8; 3]) -> Self {
        Self::new([prefix, sub_id[0], sub_id[1], sub_id[2]])
    }

    pub fn is_prefixed(&self, prefix: u8) -> bool {
        self.to_bytes()[0] == prefix
    }
}

impl Into<[u8; 4]> for FrameId {
    fn into(self) -> [u8; 4] {
        self.to_bytes()
    }
}

impl fmt::Debug for FrameId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = if self.is_short() { 3 } else { 4 };
        let bytes = self.to_bytes();
        write!(f, "FrameId(")?;
        for &c in &bytes[..len] {
            if c >= 0x20 && c <= 0x7f {
                write!(f, "{}", c as char)?;
            } else {
                write!(f, "\\x{:x}", c)?;
            }
        }
        write!(f, ")")
    }
}