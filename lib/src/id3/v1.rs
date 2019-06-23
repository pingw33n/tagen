use std::fmt;
use std::io::prelude::*;
use std::io::{self, ErrorKind, SeekFrom};

use super::string::*;
use crate::error::*;
use crate::timestamp::Timestamp;

const LEN: usize = 128;
const EXT_LEN: usize = 227;
const FULL_LEN: usize = LEN + EXT_LEN;

const GENRES: &[&str] = &[
    /* 0 */   "Blues",
    /* 1 */   "Classic Rock",
    /* 2 */   "Country",
    /* 3 */   "Dance",
    /* 4 */   "Disco",
    /* 5 */   "Funk",
    /* 6 */   "Grunge",
    /* 7 */   "Hip-Hop",
    /* 8 */   "Jazz",
    /* 9 */   "Metal",
    /* 10 */  "New Age",
    /* 11 */  "Oldies",
    /* 12 */  "Other",
    /* 13 */  "Pop",
    /* 14 */  "R&B",
    /* 15 */  "Rap",
    /* 16 */  "Reggae",
    /* 17 */  "Rock",
    /* 18 */  "Techno",
    /* 19 */  "Industrial",
    /* 20 */  "Alternative",
    /* 21 */  "Ska",
    /* 22 */  "Death Metal",
    /* 23 */  "Pranks",
    /* 24 */  "Soundtrack",
    /* 25 */  "Euro-Techno",
    /* 26 */  "Ambient",
    /* 27 */  "Trip-Hop",
    /* 28 */  "Vocal",
    /* 29 */  "Jazz+Funk",
    /* 30 */  "Fusion",
    /* 31 */  "Trance",
    /* 32 */  "Classical",
    /* 33 */  "Instrumental",
    /* 34 */  "Acid",
    /* 35 */  "House",
    /* 36 */  "Game",
    /* 37 */  "Sound Clip",
    /* 38 */  "Gospel",
    /* 39 */  "Noise",
    /* 40 */  "Alt. Rock",
    /* 41 */  "Bass",
    /* 42 */  "Soul",
    /* 43 */  "Punk",
    /* 44 */  "Space",
    /* 45 */  "Meditative",
    /* 46 */  "Instrumental Pop",
    /* 47 */  "Instrumental Rock",
    /* 48 */  "Ethnic",
    /* 49 */  "Gothic",
    /* 50 */  "Darkwave",
    /* 51 */  "Techno-Industrial",
    /* 52 */  "Electronic",
    /* 53 */  "Pop-Folk",
    /* 54 */  "Eurodance",
    /* 55 */  "Dream",
    /* 56 */  "Southern Rock",
    /* 57 */  "Comedy",
    /* 58 */  "Cult",
    /* 59 */  "Gangsta Rap",
    /* 60 */  "Top 40",
    /* 61 */  "Christian Rap",
    /* 62 */  "Pop/Funk",
    /* 63 */  "Jungle",
    /* 64 */  "Native American",
    /* 65 */  "Cabaret",
    /* 66 */  "New Wave",
    /* 67 */  "Psychedelic",
    /* 68 */  "Rave",
    /* 69 */  "Showtunes",
    /* 70 */  "Trailer",
    /* 71 */  "Lo-Fi",
    /* 72 */  "Tribal",
    /* 73 */  "Acid Punk",
    /* 74 */  "Acid Jazz",
    /* 75 */  "Polka",
    /* 76 */  "Retro",
    /* 77 */  "Musical",
    /* 78 */  "Rock & Roll",
    /* 79 */  "Hard Rock",
    /* 80 */  "Folk",
    /* 81 */  "Folk-Rock",
    /* 82 */  "National Folk",
    /* 83 */  "Swing",
    /* 84 */  "Fast-Fusion",
    /* 85 */  "Bebop",
    /* 86 */  "Latin",
    /* 87 */  "Revival",
    /* 88 */  "Celtic",
    /* 89 */  "Bluegrass",
    /* 90 */  "Avantgarde",
    /* 91 */  "Gothic Rock",
    /* 92 */  "Progressive Rock",
    /* 93 */  "Psychedelic Rock",
    /* 94 */  "Symphonic Rock",
    /* 95 */  "Slow Rock",
    /* 96 */  "Big Band",
    /* 97 */  "Chorus",
    /* 98 */  "Easy Listening",
    /* 99 */  "Acoustic",
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

const GENRES_DBG: &[&str] = &[
    /* 0 */   "BLUES",
    /* 1 */   "CLASSIC_ROCK",
    /* 2 */   "COUNTRY",
    /* 3 */   "DANCE",
    /* 4 */   "DISCO",
    /* 5 */   "FUNK",
    /* 6 */   "GRUNGE",
    /* 7 */   "HIP_HOP",
    /* 8 */   "JAZZ",
    /* 9 */   "METAL",
    /* 10 */  "NEW_AGE",
    /* 11 */  "OLDIES",
    /* 12 */  "OTHER",
    /* 13 */  "POP",
    /* 14 */  "R_B",
    /* 15 */  "RAP",
    /* 16 */  "REGGAE",
    /* 17 */  "ROCK",
    /* 18 */  "TECHNO",
    /* 19 */  "INDUSTRIAL",
    /* 20 */  "ALTERNATIVE",
    /* 21 */  "SKA",
    /* 22 */  "DEATH_METAL",
    /* 23 */  "PRANKS",
    /* 24 */  "SOUNDTRACK",
    /* 25 */  "EURO_TECHNO",
    /* 26 */  "AMBIENT",
    /* 27 */  "TRIP_HOP",
    /* 28 */  "VOCAL",
    /* 29 */  "JAZZ_FUNK",
    /* 30 */  "FUSION",
    /* 31 */  "TRANCE",
    /* 32 */  "CLASSICAL",
    /* 33 */  "INSTRUMENTAL",
    /* 34 */  "ACID",
    /* 35 */  "HOUSE",
    /* 36 */  "GAME",
    /* 37 */  "SOUND_CLIP",
    /* 38 */  "GOSPEL",
    /* 39 */  "NOISE",
    /* 40 */  "ALT_ROCK",
    /* 41 */  "BASS",
    /* 42 */  "SOUL",
    /* 43 */  "PUNK",
    /* 44 */  "SPACE",
    /* 45 */  "MEDITATIVE",
    /* 46 */  "INSTRUMENTAL_POP",
    /* 47 */  "INSTRUMENTAL_ROCK",
    /* 48 */  "ETHNIC",
    /* 49 */  "GOTHIC",
    /* 50 */  "DARKWAVE",
    /* 51 */  "TECHNO_INDUSTRIAL",
    /* 52 */  "ELECTRONIC",
    /* 53 */  "POP_FOLK",
    /* 54 */  "EURODANCE",
    /* 55 */  "DREAM",
    /* 56 */  "SOUTHERN_ROCK",
    /* 57 */  "COMEDY",
    /* 58 */  "CULT",
    /* 59 */  "GANGSTA_RAP",
    /* 60 */  "TOP_40",
    /* 61 */  "CHRISTIAN_RAP",
    /* 62 */  "POP_FUNK",
    /* 63 */  "JUNGLE",
    /* 64 */  "NATIVE_AMERICAN",
    /* 65 */  "CABARET",
    /* 66 */  "NEW_WAVE",
    /* 67 */  "PSYCHEDELIC",
    /* 68 */  "RAVE",
    /* 69 */  "SHOWTUNES",
    /* 70 */  "TRAILER",
    /* 71 */  "LO_FI",
    /* 72 */  "TRIBAL",
    /* 73 */  "ACID_PUNK",
    /* 74 */  "ACID_JAZZ",
    /* 75 */  "POLKA",
    /* 76 */  "RETRO",
    /* 77 */  "MUSICAL",
    /* 78 */  "ROCK_N_ROLL",
    /* 79 */  "HARD_ROCK",
    /* 80 */  "FOLK",
    /* 81 */  "FOLK_ROCK",
    /* 82 */  "NATIONAL_FOLK",
    /* 83 */  "SWING",
    /* 84 */  "FAST_FUSION",
    /* 85 */  "BEBOP",
    /* 86 */  "LATIN",
    /* 87 */  "REVIVAL",
    /* 88 */  "CELTIC",
    /* 89 */  "BLUEGRASS",
    /* 90 */  "AVANTGARDE",
    /* 91 */  "GOTHIC_ROCK",
    /* 92 */  "PROGRESSIVE_ROCK",
    /* 93 */  "PSYCHEDELIC_ROCK",
    /* 94 */  "SYMPHONIC_ROCK",
    /* 95 */  "SLOW_ROCK",
    /* 96 */  "BIG_BAND",
    /* 97 */  "CHORUS",
    /* 98 */  "EASY_LISTENING",
    /* 99 */  "ACOUSTIC",
    /* 100 */ "HUMOUR",
    /* 101 */ "SPEECH",
    /* 102 */ "CHANSON",
    /* 103 */ "OPERA",
    /* 104 */ "CHAMBER_MUSIC",
    /* 105 */ "SONATA",
    /* 106 */ "SYMPHONY",
    /* 107 */ "BOOTY_BASS",
    /* 108 */ "PRIMUS",
    /* 109 */ "PORN_GROOVE",
    /* 110 */ "SATIRE",
    /* 111 */ "SLOW_JAM",
    /* 112 */ "CLUB",
    /* 113 */ "TANGO",
    /* 114 */ "SAMBA",
    /* 115 */ "FOLKLORE",
    /* 116 */ "BALLAD",
    /* 117 */ "POWER_BALLAD",
    /* 118 */ "RHYTHMIC_SOUL",
    /* 119 */ "FREESTYLE",
    /* 120 */ "DUET",
    /* 121 */ "PUNK_ROCK",
    /* 122 */ "DRUM_SOLO",
    /* 123 */ "A_CAPPELLA",
    /* 124 */ "EURO_HOUSE",
    /* 125 */ "DANCE_HALL",
    /* 126 */ "GOA",
    /* 127 */ "DRUM_N_BASS",
    /* 128 */ "CLUB_HOUSE",
    /* 129 */ "HARDCORE",
    /* 130 */ "TERROR",
    /* 131 */ "INDIE",
    /* 132 */ "BRIT_POP",
    /* 133 */ "AFRO_PUNK",
    /* 134 */ "POLSK_PUNK",
    /* 135 */ "BEAT",
    /* 136 */ "CHRISTIAN_GANGSTA_RAP",
    /* 137 */ "HEAVY_METAL",
    /* 138 */ "BLACK_METAL",
    /* 139 */ "CROSSOVER",
    /* 140 */ "CONTEMPORARY_CHRISTIAN",
    /* 141 */ "CHRISTIAN_ROCK",
    /* 142 */ "MERENGUE",
    /* 143 */ "SALSA",
    /* 144 */ "THRASH_METAL",
    /* 145 */ "ANIME",
    /* 146 */ "JPOP",
    /* 147 */ "SYNTHPOP",
    /* 148 */ "ABSTRACT",
    /* 149 */ "ART_ROCK",
    /* 150 */ "BAROQUE",
    /* 151 */ "BHANGRA",
    /* 152 */ "BIG_BEAT",
    /* 153 */ "BREAKBEAT",
    /* 154 */ "CHILLOUT",
    /* 155 */ "DOWNTEMPO",
    /* 156 */ "DUB",
    /* 157 */ "EBM",
    /* 158 */ "ECLECTIC",
    /* 159 */ "ELECTRO",
    /* 160 */ "ELECTROCLASH",
    /* 161 */ "EMO",
    /* 162 */ "EXPERIMENTAL",
    /* 163 */ "GARAGE",
    /* 164 */ "GLOBAL",
    /* 165 */ "IDM",
    /* 166 */ "ILLBIENT",
    /* 167 */ "INDUSTRO_GOTH",
    /* 168 */ "JAM_BAND",
    /* 169 */ "KRAUTROCK",
    /* 170 */ "LEFTFIELD",
    /* 171 */ "LOUNGE",
    /* 172 */ "MATH_ROCK",
    /* 173 */ "NEW_ROMANTIC",
    /* 174 */ "NU_BREAKZ",
    /* 175 */ "POST_PUNK",
    /* 176 */ "POST_ROCK",
    /* 177 */ "PSYTRANCE",
    /* 178 */ "SHOEGAZE",
    /* 179 */ "SPACE_ROCK",
    /* 180 */ "TROP_ROCK",
    /* 181 */ "WORLD_MUSIC",
    /* 182 */ "NEOCLASSICAL",
    /* 183 */ "AUDIOBOOK",
    /* 184 */ "AUDIO_THEATRE",
    /* 185 */ "NEUE_DEUTSCHE_WELLE",
    /* 186 */ "PODCAST",
    /* 187 */ "INDIE_ROCK",
    /* 188 */ "G_FUNK",
    /* 189 */ "DUBSTEP",
    /* 190 */ "GARAGE_ROCK",
    /* 191 */ "PSYBIENT",
];

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Genre(u8);

impl Genre {
    pub const BLUES: u8 = 0;
    pub const CLASSIC_ROCK: u8 = 1;
    pub const COUNTRY: u8 = 2;
    pub const DANCE: u8 = 3;
    pub const DISCO: u8 = 4;
    pub const FUNK: u8 = 5;
    pub const GRUNGE: u8 = 6;
    pub const HIP_HOP: u8 = 7;
    pub const JAZZ: u8 = 8;
    pub const METAL: u8 = 9;
    pub const NEW_AGE: u8 = 10;
    pub const OLDIES: u8 = 11;
    pub const OTHER: u8 = 12;
    pub const POP: u8 = 13;
    pub const R_B: u8 = 14;
    pub const RAP: u8 = 15;
    pub const REGGAE: u8 = 16;
    pub const ROCK: u8 = 17;
    pub const TECHNO: u8 = 18;
    pub const INDUSTRIAL: u8 = 19;
    pub const ALTERNATIVE: u8 = 20;
    pub const SKA: u8 = 21;
    pub const DEATH_METAL: u8 = 22;
    pub const PRANKS: u8 = 23;
    pub const SOUNDTRACK: u8 = 24;
    pub const EURO_TECHNO: u8 = 25;
    pub const AMBIENT: u8 = 26;
    pub const TRIP_HOP: u8 = 27;
    pub const VOCAL: u8 = 28;
    pub const JAZZ_FUNK: u8 = 29;
    pub const FUSION: u8 = 30;
    pub const TRANCE: u8 = 31;
    pub const CLASSICAL: u8 = 32;
    pub const INSTRUMENTAL: u8 = 33;
    pub const ACID: u8 = 34;
    pub const HOUSE: u8 = 35;
    pub const GAME: u8 = 36;
    pub const SOUND_CLIP: u8 = 37;
    pub const GOSPEL: u8 = 38;
    pub const NOISE: u8 = 39;
    pub const ALT_ROCK: u8 = 40;
    pub const BASS: u8 = 41;
    pub const SOUL: u8 = 42;
    pub const PUNK: u8 = 43;
    pub const SPACE: u8 = 44;
    pub const MEDITATIVE: u8 = 45;
    pub const INSTRUMENTAL_POP: u8 = 46;
    pub const INSTRUMENTAL_ROCK: u8 = 47;
    pub const ETHNIC: u8 = 48;
    pub const GOTHIC: u8 = 49;
    pub const DARKWAVE: u8 = 50;
    pub const TECHNO_INDUSTRIAL: u8 = 51;
    pub const ELECTRONIC: u8 = 52;
    pub const POP_FOLK: u8 = 53;
    pub const EURODANCE: u8 = 54;
    pub const DREAM: u8 = 55;
    pub const SOUTHERN_ROCK: u8 = 56;
    pub const COMEDY: u8 = 57;
    pub const CULT: u8 = 58;
    pub const GANGSTA_RAP: u8 = 59;
    pub const TOP_40: u8 = 60;
    pub const CHRISTIAN_RAP: u8 = 61;
    pub const POP_FUNK: u8 = 62;
    pub const JUNGLE: u8 = 63;
    pub const NATIVE_AMERICAN: u8 = 64;
    pub const CABARET: u8 = 65;
    pub const NEW_WAVE: u8 = 66;
    pub const PSYCHEDELIC: u8 = 67;
    pub const RAVE: u8 = 68;
    pub const SHOWTUNES: u8 = 69;
    pub const TRAILER: u8 = 70;
    pub const LO_FI: u8 = 71;
    pub const TRIBAL: u8 = 72;
    pub const ACID_PUNK: u8 = 73;
    pub const ACID_JAZZ: u8 = 74;
    pub const POLKA: u8 = 75;
    pub const RETRO: u8 = 76;
    pub const MUSICAL: u8 = 77;
    pub const ROCK_N_ROLL: u8 = 78;
    pub const HARD_ROCK: u8 = 79;
    pub const FOLK: u8 = 80;
    pub const FOLK_ROCK: u8 = 81;
    pub const NATIONAL_FOLK: u8 = 82;
    pub const SWING: u8 = 83;
    pub const FAST_FUSION: u8 = 84;
    pub const BEBOP: u8 = 85;
    pub const LATIN: u8 = 86;
    pub const REVIVAL: u8 = 87;
    pub const CELTIC: u8 = 88;
    pub const BLUEGRASS: u8 = 89;
    pub const AVANTGARDE: u8 = 90;
    pub const GOTHIC_ROCK: u8 = 91;
    pub const PROGRESSIVE_ROCK: u8 = 92;
    pub const PSYCHEDELIC_ROCK: u8 = 93;
    pub const SYMPHONIC_ROCK: u8 = 94;
    pub const SLOW_ROCK: u8 = 95;
    pub const BIG_BAND: u8 = 96;
    pub const CHORUS: u8 = 97;
    pub const EASY_LISTENING: u8 = 98;
    pub const ACOUSTIC: u8 = 99;
    pub const HUMOUR: u8 = 100;
    pub const SPEECH: u8 = 101;
    pub const CHANSON: u8 = 102;
    pub const OPERA: u8 = 103;
    pub const CHAMBER_MUSIC: u8 = 104;
    pub const SONATA: u8 = 105;
    pub const SYMPHONY: u8 = 106;
    pub const BOOTY_BASS: u8 = 107;
    pub const PRIMUS: u8 = 108;
    pub const PORN_GROOVE: u8 = 109;
    pub const SATIRE: u8 = 110;
    pub const SLOW_JAM: u8 = 111;
    pub const CLUB: u8 = 112;
    pub const TANGO: u8 = 113;
    pub const SAMBA: u8 = 114;
    pub const FOLKLORE: u8 = 115;
    pub const BALLAD: u8 = 116;
    pub const POWER_BALLAD: u8 = 117;
    pub const RHYTHMIC_SOUL: u8 = 118;
    pub const FREESTYLE: u8 = 119;
    pub const DUET: u8 = 120;
    pub const PUNK_ROCK: u8 = 121;
    pub const DRUM_SOLO: u8 = 122;
    pub const A_CAPPELLA: u8 = 123;
    pub const EURO_HOUSE: u8 = 124;
    pub const DANCE_HALL: u8 = 125;
    pub const GOA: u8 = 126;
    pub const DRUM_N_BASS: u8 = 127;
    pub const CLUB_HOUSE: u8 = 128;
    pub const HARDCORE: u8 = 129;
    pub const TERROR: u8 = 130;
    pub const INDIE: u8 = 131;
    pub const BRIT_POP: u8 = 132;
    pub const AFRO_PUNK: u8 = 133;
    pub const POLSK_PUNK: u8 = 134;
    pub const BEAT: u8 = 135;
    pub const CHRISTIAN_GANGSTA_RAP: u8 = 136;
    pub const HEAVY_METAL: u8 = 137;
    pub const BLACK_METAL: u8 = 138;
    pub const CROSSOVER: u8 = 139;
    pub const CONTEMPORARY_CHRISTIAN: u8 = 140;
    pub const CHRISTIAN_ROCK: u8 = 141;
    pub const MERENGUE: u8 = 142;
    pub const SALSA: u8 = 143;
    pub const THRASH_METAL: u8 = 144;
    pub const ANIME: u8 = 145;
    pub const JPOP: u8 = 146;
    pub const SYNTHPOP: u8 = 147;
    pub const ABSTRACT: u8 = 148;
    pub const ART_ROCK: u8 = 149;
    pub const BAROQUE: u8 = 150;
    pub const BHANGRA: u8 = 151;
    pub const BIG_BEAT: u8 = 152;
    pub const BREAKBEAT: u8 = 153;
    pub const CHILLOUT: u8 = 154;
    pub const DOWNTEMPO: u8 = 155;
    pub const DUB: u8 = 156;
    pub const EBM: u8 = 157;
    pub const ECLECTIC: u8 = 158;
    pub const ELECTRO: u8 = 159;
    pub const ELECTROCLASH: u8 = 160;
    pub const EMO: u8 = 161;
    pub const EXPERIMENTAL: u8 = 162;
    pub const GARAGE: u8 = 163;
    pub const GLOBAL: u8 = 164;
    pub const IDM: u8 = 165;
    pub const ILLBIENT: u8 = 166;
    pub const INDUSTRO_GOTH: u8 = 167;
    pub const JAM_BAND: u8 = 168;
    pub const KRAUTROCK: u8 = 169;
    pub const LEFTFIELD: u8 = 170;
    pub const LOUNGE: u8 = 171;
    pub const MATH_ROCK: u8 = 172;
    pub const NEW_ROMANTIC: u8 = 173;
    pub const NU_BREAKZ: u8 = 174;
    pub const POST_PUNK: u8 = 175;
    pub const POST_ROCK: u8 = 176;
    pub const PSYTRANCE: u8 = 177;
    pub const SHOEGAZE: u8 = 178;
    pub const SPACE_ROCK: u8 = 179;
    pub const TROP_ROCK: u8 = 180;
    pub const WORLD_MUSIC: u8 = 181;
    pub const NEOCLASSICAL: u8 = 182;
    pub const AUDIOBOOK: u8 = 183;
    pub const AUDIO_THEATRE: u8 = 184;
    pub const NEUE_DEUTSCHE_WELLE: u8 = 185;
    pub const PODCAST: u8 = 186;
    pub const INDIE_ROCK: u8 = 187;
    pub const G_FUNK: u8 = 188;
    pub const DUBSTEP: u8 = 189;
    pub const GARAGE_ROCK: u8 = 190;
    pub const PSYBIENT: u8 = 191;

    pub fn new(v: u8) -> Option<Self> {
        if v == 255 {
            None
        } else {
            Some(Self(v))
        }
    }

    pub fn description(&self) -> Option<&'static str> {
        GENRES.get(self.0 as usize).map(|v| *v)
    }
}

impl fmt::Display for Genre {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(s) = self.description() {
            write!(f, "{}", s)
        } else {
            write!(f, "Unknown ({})", self.0)
        }
    }
}

impl fmt::Debug for Genre {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(s) = GENRES_DBG.get(self.0 as usize) {
            write!(f, "{}", s)
        } else {
            write!(f, "Genre({})", self.0)
        }
    }
}

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
    pub genre: Option<Genre>,
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

    pub fn date(&self) -> Option<Timestamp> {
        let year = self.year.parse().ok()?;
        Timestamp::new_y(year)
    }

    pub fn len(&self) -> u32 {
        (if self.ext.is_some() { FULL_LEN } else { LEN }) as u32
    }

    pub(crate) fn read(mut rd: impl Read + Seek) -> io::Result<Self> {
        let mut data = [0; FULL_LEN];
        for &len in &[FULL_LEN, LEN] {
            let last_attempt = len == LEN;

            if let Err(e) = rd.seek(SeekFrom::End(-(len as i64))) {
                if e.kind() == ErrorKind::InvalidInput {
                    continue;
                }
                return Err(e);
            }
            let data = &mut data[..len];
            if let Err(e) = rd.read_exact(data) {
                if e.kind() == ErrorKind::UnexpectedEof && !last_attempt {
                    continue;
                } else {
                    return Err(e);
                }
            }
            match Self::decode(&data) {
                Ok(v) => return Ok(v),
                Err(e) => if last_attempt {
                    return Err(e.into_invalid_data_err())
                }
            }
        }
        unreachable!()
    }

    fn decode(buf: &[u8]) -> Result<Self> {
        if buf.len() < LEN || &buf[..3] != b"TAG" {
            return Err(Error("couldn't find ID3v1 magic"));
        }

        let ext = if buf[4] == b'+' {
            if buf.len() < LEN + EXT_LEN || &buf[EXT_LEN..EXT_LEN + 3] != b"TAG" {
                return Err(Error("couldn't find ID3v1 magic"));
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

        Ok(Self::decode0(buf, ext))
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
        let genre = Genre::new(buf[127]);

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