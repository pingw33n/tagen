use std::fmt;

macro_rules! frame_id {
    ($s:expr) => {
        FrameId::new([$s[0], $s[1], $s[2], $s[3]])
    };
}

macro_rules! v22_frame_id {
    ($s:expr) => {
        FrameId::new_v22([$s[0], $s[1], $s[2]])
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
    pub const OWNER: Self = frame_id!(b"TOWN");
    pub const ARTIST: Self = frame_id!(b"TPE1");
    pub const ALBUM_ARTIST: Self = frame_id!(b"TPE2");
    pub const CONDUCTOR: Self = frame_id!(b"TPE3");
    pub const REMIXER: Self = frame_id!(b"TPE4");
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

    pub const COMMERCIAL_URL: Self = frame_id!(b"WCOM");
    pub const COPYRIGHT_URL: Self = frame_id!(b"WCOP");
    pub const AUDIO_FILE_URL: Self = frame_id!(b"WOAF");
    pub const ARTIST_URL: Self = frame_id!(b"WOAR");
    pub const AUDIO_SOURCE_URL: Self = frame_id!(b"WOAS");
    pub const INTERNET_RADIO_URL: Self = frame_id!(b"WORS");
    pub const PAYMENT_URL: Self = frame_id!(b"WPAY");
    pub const PUBLISHER_URL: Self = frame_id!(b"WPUB");
    pub const USER_URL: Self = frame_id!(b"WXXX");

    // Misc frames.

    pub const AUDIO_ENCRYPTION: Self = frame_id!(b"AENC");
    pub const ATTACHED_PICTURE: Self = frame_id!(b"APIC");
    pub const AUDIO_SEEK_POINT_INDEX: Self = frame_id!(b"ASPI");
    pub const CHAPTER: Self = frame_id!(b"CHAP");
    pub const COMMENT: Self = frame_id!(b"COMM");
    pub const COMMERCIAL: Self = frame_id!(b"COMR");
    pub const ENCRYPTION: Self = frame_id!(b"ENCR");
    pub const EQUALIZATION: Self = frame_id!(b"EQU2");
    pub const EVENT_TIMING_CODES: Self = frame_id!(b"ETCO");
    pub const GENERAL_ENCAP_OBJECT: Self = frame_id!(b"GEOB");
    pub const GROUP_ID: Self = frame_id!(b"GRID");
    pub const INVOLVED_PEOPLE_LIST: Self = frame_id!(b"IPLS"); // v2.3
    pub const LINK: Self = frame_id!(b"LINK");
    pub const MUSIC_CD_ID: Self = frame_id!(b"MCDI");
    pub const MPEG_LOCATION_LUT: Self = frame_id!(b"MLLT");
    pub const OWNERSHIP: Self = frame_id!(b"OWNE");
    pub const PRIVATE: Self = frame_id!(b"PRIV");
    pub const PLAY_COUNTER: Self = frame_id!(b"PCNT");
    pub const POPULARIMETER: Self = frame_id!(b"POPM");
    pub const POS_SYNC: Self = frame_id!(b"POSS");
    pub const RECOMMENDED_BUF: Self = frame_id!(b"RBUF");
    pub const RELATIVE_VOL_ADJUST: Self = frame_id!(b"RVA2");
    pub const REVERB: Self = frame_id!(b"RVRB");
    pub const SEEK: Self = frame_id!(b"SEEK");
    pub const SIGNATURE: Self = frame_id!(b"SIGN");
    pub const SYNC_LYRICS: Self = frame_id!(b"SYLT");
    pub const SYNC_TEMPO_CODES: Self = frame_id!(b"SYTC");
    pub const UNIQUE_FILE_ID: Self = frame_id!(b"UFID");
    pub const TERMS_OF_USE: Self = frame_id!(b"USER");
    pub const UNSYNC_LYRICS: Self = frame_id!(b"USLT");

    // Obsolete frames.

    pub const V22_ALBUM: Self = v22_frame_id!(b"TAL");
    pub const V22_BPM: Self = v22_frame_id!(b"TBP");
    pub const V22_COMPOSER: Self = v22_frame_id!(b"TCM");
    pub const V22_CONTENT_TYPE: Self = v22_frame_id!(b"TCO");
    pub const V22_COPYRIGHT: Self = v22_frame_id!(b"TCR");
    pub const V22_DELAY: Self = v22_frame_id!(b"TDY");
    pub const V22_ENCODED_BY: Self = v22_frame_id!(b"TEN");
    pub const V22_LYRICIST: Self = v22_frame_id!(b"TXT");
    pub const V22_FILE_TYPE: Self = v22_frame_id!(b"TFT");
    pub const V22_GROUP_TITLE: Self = v22_frame_id!(b"TT1");
    pub const V22_TITLE: Self = v22_frame_id!(b"TT2");
    pub const V22_SUBTITLE: Self = v22_frame_id!(b"TT3");
    pub const V22_PEOPLE: Self = v22_frame_id!(b"IPL");
    pub const V22_INITIAL_KEY: Self = v22_frame_id!(b"TKE");
    pub const V22_LANGUAGE: Self = v22_frame_id!(b"TLA");
    pub const V22_LENGTH: Self = v22_frame_id!(b"TLE");
    pub const V22_MEDIA_TYPE: Self = v22_frame_id!(b"TMT");
    pub const V22_ORIGINAL_ALBUM: Self = v22_frame_id!(b"TOT");
    pub const V22_ORIGINAL_FILENAME: Self = v22_frame_id!(b"TOF");
    pub const V22_ORIGINAL_LYRICIST: Self = v22_frame_id!(b"TOL");
    pub const V22_ORIGINAL_ARTIST: Self = v22_frame_id!(b"TOA");
    pub const V23_ORIGINAL_RELEASE_YEAR: Self = frame_id!(b"TORY");
    pub const V22_ARTIST: Self = v22_frame_id!(b"TP1");
    pub const V22_ALBUM_ARTIST: Self = v22_frame_id!(b"TP2");
    pub const V22_CONDUCTOR: Self = v22_frame_id!(b"TP3");
    pub const V22_REMIXER: Self = v22_frame_id!(b"TP4");
    pub const V22_DISC: Self = v22_frame_id!(b"TPA");
    pub const V22_PUBLISHER: Self = v22_frame_id!(b"TPB");
    pub const V22_ISRC: Self = v22_frame_id!(b"TRC");
    pub const V22_TRACK: Self = v22_frame_id!(b"TRK");
    pub const V22_ENCODER_SETTINGS: Self = v22_frame_id!(b"TSS");
    pub const V22_USER_TEXT: Self = v22_frame_id!(b"TXX");

    pub const V22_DATE: Self = v22_frame_id!(b"TDA");
    pub const V23_DATE: Self = frame_id!(b"TDAT");
    pub const V22_TIME: Self = v22_frame_id!(b"TIM");
    pub const V23_TIME: Self = frame_id!(b"TIME");
    pub const V22_RECORDING_DATES: Self = v22_frame_id!(b"TRD");
    pub const V23_RECORDING_DATES: Self = frame_id!(b"TRDA");
    pub const V22_SIZE: Self = v22_frame_id!(b"TSI");
    pub const V23_SIZE: Self = frame_id!(b"TSIZ");
    pub const V22_YEAR: Self = v22_frame_id!(b"TYE");
    pub const V23_YEAR: Self = frame_id!(b"TYER");

    pub const V22_COMMERCIAL_URL: Self = v22_frame_id!(b"WCM");
    pub const V22_COPYRIGHT_URL: Self = v22_frame_id!(b"WCP");
    pub const V22_AUDIO_FILE_URL: Self = v22_frame_id!(b"WAF");
    pub const V22_ARTIST_URL: Self = v22_frame_id!(b"WAR");
    pub const V22_AUDIO_SOURCE_URL: Self = v22_frame_id!(b"WAS");
    pub const V22_PUBLISHER_URL: Self = v22_frame_id!(b"WPB");
    pub const V22_USER_URL: Self = v22_frame_id!(b"WXXX");

    pub const V22_AUDIO_ENCRYPTION: Self = v22_frame_id!(b"CRA");
    pub const V22_ATTACHED_PICTURE: Self = v22_frame_id!(b"PIC");
    pub const V22_COMMENT: Self = v22_frame_id!(b"COM");
    pub const V22_EQUALIZATION: Self = v22_frame_id!(b"EQU");
    pub const V23_EQUALIZATION: Self = frame_id!(b"EQUA");
    pub const V22_EVENT_TIMING_CODES: Self = v22_frame_id!(b"ETC");
    pub const V22_GENERAL_ENCAP_OBJECT: Self = v22_frame_id!(b"GEO");
    pub const V22_LINK: Self = v22_frame_id!(b"LNK");
    pub const V22_MUSIC_CD_ID: Self = v22_frame_id!(b"MCI");
    pub const V22_MPEG_LOCATION_LUT: Self = v22_frame_id!(b"MLL");
    pub const V22_POPULARIMETER: Self = v22_frame_id!(b"POP");
    pub const V23_RELATIVE_VOL_ADJUST: Self = frame_id!(b"RVAD");
    pub const V22_RELATIVE_VOL_ADJUST: Self = v22_frame_id!(b"RVA");
    pub const V22_REVERB: Self = v22_frame_id!(b"REV");
    pub const V22_SYNC_LYRICS: Self = v22_frame_id!(b"SLT");
    pub const V22_SYNC_TEMPO_CODES: Self = v22_frame_id!(b"STC");
    pub const V22_UNIQUE_FILE_ID: Self = v22_frame_id!(b"UFI");
    pub const V22_UNSYNC_LYRICS: Self = v22_frame_id!(b"ULT");

    pub const fn new(v: [u8; 4]) -> Self {
        Self((v[0] as u32) << 24 |
            (v[1] as u32) << 16 |
            (v[2] as u32) << 8 |
            (v[3] as u32) << 0)
    }

    pub const fn new_v22(v: [u8; 3]) -> Self {
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

    pub fn is_v22(&self) -> bool {
        self.0 & 0xff == 0
    }

    pub fn text(sub_id: [u8; 3]) -> Self {
        Self::prefixed(b'T', sub_id)
    }

    pub fn is_text(&self) -> bool {
        self.is_prefixed(b'T') && *self != Self::USER_TEXT
    }

    pub fn url(sub_id: [u8; 3]) -> Self {
        Self::prefixed(b'W', sub_id)
    }

    pub fn is_url(&self) -> bool {
        self.is_prefixed(b'W') && *self != Self::USER_URL
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
        let len = if self.is_v22() { 3 } else { 4 };
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