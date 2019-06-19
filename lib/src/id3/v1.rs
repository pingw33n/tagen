use std::io::prelude::*;
use std::io::{ErrorKind, Result, SeekFrom};

use super::string::*;

const LEN: usize = 128;
const EXT_LEN: usize = 227;
const FULL_LEN: usize = LEN + EXT_LEN;

const GENRES: &[&str] = &[
    /* 0 */ "Blues",
    /* 1 */ "Classic Rock",
    /* 2 */ "Country",
    /* 3 */ "Dance",
    /* 4 */ "Disco",
    /* 5 */ "Funk",
    /* 6 */ "Grunge",
    /* 7 */ "Hip-Hop",
    /* 8 */ "Jazz",
    /* 9 */ "Metal",
    /* 10 */ "New Age",
    /* 11 */ "Oldies",
    /* 12 */ "Other",
    /* 13 */ "Pop",
    /* 14 */ "R&B",
    /* 15 */ "Rap",
    /* 16 */ "Reggae",
    /* 17 */ "Rock",
    /* 18 */ "Techno",
    /* 19 */ "Industrial",
    /* 20 */ "Alternative",
    /* 21 */ "Ska",
    /* 22 */ "Death Metal",
    /* 23 */ "Pranks",
    /* 24 */ "Soundtrack",
    /* 25 */ "Euro-Techno",
    /* 26 */ "Ambient",
    /* 27 */ "Trip-Hop",
    /* 28 */ "Vocal",
    /* 29 */ "Jazz+Funk",
    /* 30 */ "Fusion",
    /* 31 */ "Trance",
    /* 32 */ "Classical",
    /* 33 */ "Instrumental",
    /* 34 */ "Acid",
    /* 35 */ "House",
    /* 36 */ "Game",
    /* 37 */ "Sound Clip",
    /* 38 */ "Gospel",
    /* 39 */ "Noise",
    /* 40 */ "Alt. Rock",
    /* 41 */ "Bass",
    /* 42 */ "Soul",
    /* 43 */ "Punk",
    /* 44 */ "Space",
    /* 45 */ "Meditative",
    /* 46 */ "Instrumental Pop",
    /* 47 */ "Instrumental Rock",
    /* 48 */ "Ethnic",
    /* 49 */ "Gothic",
    /* 50 */ "Darkwave",
    /* 51 */ "Techno-Industrial",
    /* 52 */ "Electronic",
    /* 53 */ "Pop-Folk",
    /* 54 */ "Eurodance",
    /* 55 */ "Dream",
    /* 56 */ "Southern Rock",
    /* 57 */ "Comedy",
    /* 58 */ "Cult",
    /* 59 */ "Gangsta Rap",
    /* 60 */ "Top 40",
    /* 61 */ "Christian Rap",
    /* 62 */ "Pop/Funk",
    /* 63 */ "Jungle",
    /* 64 */ "Native American",
    /* 65 */ "Cabaret",
    /* 66 */ "New Wave",
    /* 67 */ "Psychedelic",
    /* 68 */ "Rave",
    /* 69 */ "Showtunes",
    /* 70 */ "Trailer",
    /* 71 */ "Lo-Fi",
    /* 72 */ "Tribal",
    /* 73 */ "Acid Punk",
    /* 74 */ "Acid Jazz",
    /* 75 */ "Polka",
    /* 76 */ "Retro",
    /* 77 */ "Musical",
    /* 78 */ "Rock & Roll",
    /* 79 */ "Hard Rock",
    /* 80 */ "Folk",
    /* 81 */ "Folk-Rock",
    /* 82 */ "National Folk",
    /* 83 */ "Swing",
    /* 84 */ "Fast-Fusion",
    /* 85 */ "Bebop",
    /* 86 */ "Latin",
    /* 87 */ "Revival",
    /* 88 */ "Celtic",
    /* 89 */ "Bluegrass",
    /* 90 */ "Avantgarde",
    /* 91 */ "Gothic Rock",
    /* 92 */ "Progressive Rock",
    /* 93 */ "Psychedelic Rock",
    /* 94 */ "Symphonic Rock",
    /* 95 */ "Slow Rock",
    /* 96 */ "Big Band",
    /* 97 */ "Chorus",
    /* 98 */ "Easy Listening",
    /* 99 */ "Acoustic",
    /* 100 */ "Humour",
    /* 101 */ "Speech",
    /* 102 */ "Chanson",
    /* 103 */ "Opera",
    /* 104 */ "Chamber Music",
    /* 105 */ "Sonata",
    /* 106 */ "Symphony",
    /* 107 */ "Booty Bass",
    /* 108 */ "Primus",
    /* 109 */ "Porn Groove",
    /* 110 */ "Satire",
    /* 111 */ "Slow Jam",
    /* 112 */ "Club",
    /* 113 */ "Tango",
    /* 114 */ "Samba",
    /* 115 */ "Folklore",
    /* 116 */ "Ballad",
    /* 117 */ "Power Ballad",
    /* 118 */ "Rhythmic Soul",
    /* 119 */ "Freestyle",
    /* 120 */ "Duet",
    /* 121 */ "Punk Rock",
    /* 122 */ "Drum Solo",
    /* 123 */ "A Cappella",
    /* 124 */ "Euro-House",
    /* 125 */ "Dance Hall",
    /* 126 */ "Goa",
    /* 127 */ "Drum & Bass",
    /* 128 */ "Club-House",
    /* 129 */ "Hardcore",
    /* 130 */ "Terror",
    /* 131 */ "Indie",
    /* 132 */ "BritPop",
    /* 133 */ "Afro-Punk",
    /* 134 */ "Polsk Punk",
    /* 135 */ "Beat",
    /* 136 */ "Christian Gangsta Rap",
    /* 137 */ "Heavy Metal",
    /* 138 */ "Black Metal",
    /* 139 */ "Crossover",
    /* 140 */ "Contemporary Christian",
    /* 141 */ "Christian Rock",
    /* 142 */ "Merengue",
    /* 143 */ "Salsa",
    /* 144 */ "Thrash Metal",
    /* 145 */ "Anime",
    /* 146 */ "JPop",
    /* 147 */ "Synthpop",
    /* 148 */ "Abstract",
    /* 149 */ "Art Rock",
    /* 150 */ "Baroque",
    /* 151 */ "Bhangra",
    /* 152 */ "Big Beat",
    /* 153 */ "Breakbeat",
    /* 154 */ "Chillout",
    /* 155 */ "Downtempo",
    /* 156 */ "Dub",
    /* 157 */ "EBM",
    /* 158 */ "Eclectic",
    /* 159 */ "Electro",
    /* 160 */ "Electroclash",
    /* 161 */ "Emo",
    /* 162 */ "Experimental",
    /* 163 */ "Garage",
    /* 164 */ "Global",
    /* 165 */ "IDM",
    /* 166 */ "Illbient",
    /* 167 */ "Industro-Goth",
    /* 168 */ "Jam Band",
    /* 169 */ "Krautrock",
    /* 170 */ "Leftfield",
    /* 171 */ "Lounge",
    /* 172 */ "Math Rock",
    /* 173 */ "New Romantic",
    /* 174 */ "Nu-Breakz",
    /* 175 */ "Post-Punk",
    /* 176 */ "Post-Rock",
    /* 177 */ "Psytrance",
    /* 178 */ "Shoegaze",
    /* 179 */ "Space Rock",
    /* 180 */ "Trop Rock",
    /* 181 */ "World Music",
    /* 182 */ "Neoclassical",
    /* 183 */ "Audiobook",
    /* 184 */ "Audio Theatre",
    /* 185 */ "Neue Deutsche Welle",
    /* 186 */ "Podcast",
    /* 187 */ "Indie Rock",
    /* 188 */ "G-Funk",
    /* 189 */ "Dubstep",
    /* 190 */ "Garage Rock",
    /* 191 */ "Psybient",
];

pub fn genre_str(genre: u8) -> Option<&'static str> {
    GENRES.get(genre as usize).map(|v| *v)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Tag {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: Option<u8>,
    pub genre: Option<u8>,
    pub ext: Option<ExtTag>,
}

impl Tag {
    pub fn best_title(&self) -> &str {
        self.ext.as_ref().map(|e| &e.title).unwrap_or(&self.title)
    }

    pub fn best_artist(&self) -> &str {
        self.ext.as_ref().map(|e| &e.artist).unwrap_or(&self.artist)
    }

    pub fn best_album(&self) -> &str {
        self.ext.as_ref().map(|e| &e.album).unwrap_or(&self.album)
    }

    pub fn len(&self) -> u32 {
        (if self.ext.is_some() { FULL_LEN } else { LEN }) as u32
    }

    pub(crate) fn read(mut rd: impl Read + Seek) -> Result<Option<Self>> {
        let mut data = [0; FULL_LEN];
        for &len in &[FULL_LEN, LEN] {
            if let Err(e) = rd.seek(SeekFrom::End(-(len as i64))) {
                if e.kind() == ErrorKind::InvalidInput {
                    continue;
                }
                return Err(e);
            }
            let data = &mut data[..len];
            if let Err(e) = rd.read_exact(data) {
                if e.kind() == ErrorKind::UnexpectedEof {
                    continue;
                } else {
                    return Err(e);
                }
            }
            let r = Self::decode(&data);
            if r.is_some() {
                return Ok(r);
            }
        }
        Ok(None)
    }

    fn decode(buf: &[u8]) -> Option<Self> {
        if buf.len() < LEN || &buf[..3] != b"TAG" {
            return None;
        }

        let ext = if buf[4] == b'+' {
            if buf.len() < LEN + EXT_LEN || &buf[EXT_LEN..EXT_LEN + 3] != b"TAG" {
                return None;
            }
            Some(ExtTag::decode(buf))
        } else {
            None
        };

        let buf = if ext.is_some() {
            &buf[EXT_LEN..]
        } else {
            buf
        };

        Some(Self::decode0(buf, ext))
    }

    fn decode0(buf: &[u8], ext: Option<ExtTag>) -> Self {
        let title = decode_str(&buf[3..33]);
        let artist = decode_str(&buf[33..63]);
        let album = decode_str(&buf[63..93]);
        let year = decode_str(&buf[93..97]);

        let (comment_bytes, track) = if buf[125] == 0 && buf[126] != 0 {
            (&buf[97..125], Some(buf[126]))
        }  else {
            (&buf[97..127], None)
        };
        let comment = decode_str(comment_bytes);
        let genre = if buf[127] != 255 {
            Some(buf[127])
        } else {
            None
        };

        Self {
            title,
            artist,
            album,
            year,
            comment,
            track,
            genre,
            ext,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExtTag {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub speed: Option<u8>,
    pub genre: String,
    pub start_time: String,
    pub end_time: String,
}

impl ExtTag {
    fn decode(buf: &[u8]) -> Self {
        let title = decode_str(&buf[4..64]);
        let artist = decode_str(&buf[64..124]);
        let album = decode_str(&buf[124..184]);
        let speed = if buf[184] == 0 { None } else { Some(buf[184]) };
        let genre = decode_str(&buf[185..215]);
        let start_time = decode_str(&buf[185..215]);
        let end_time = decode_str(&buf[185..215]);
        Self {
            title,
            artist,
            album,
            speed,
            genre,
            start_time,
            end_time,
        }
    }
}

fn decode_str(buf: &[u8]) -> String {
    Decoder::new(Encoding::Latin1).decode_maybe_null_terminated(buf).unwrap()
}