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
    pub const BLUES: Self = Self(0);
    pub const CLASSIC_ROCK: Self = Self(1);
    pub const COUNTRY: Self = Self(2);
    pub const DANCE: Self = Self(3);
    pub const DISCO: Self = Self(4);
    pub const FUNK: Self = Self(5);
    pub const GRUNGE: Self = Self(6);
    pub const HIP_HOP: Self = Self(7);
    pub const JAZZ: Self = Self(8);
    pub const METAL: Self = Self(9);
    pub const NEW_AGE: Self = Self(10);
    pub const OLDIES: Self = Self(11);
    pub const OTHER: Self = Self(12);
    pub const POP: Self = Self(13);
    pub const R_B: Self = Self(14);
    pub const RAP: Self = Self(15);
    pub const REGGAE: Self = Self(16);
    pub const ROCK: Self = Self(17);
    pub const TECHNO: Self = Self(18);
    pub const INDUSTRIAL: Self = Self(19);
    pub const ALTERNATIVE: Self = Self(20);
    pub const SKA: Self = Self(21);
    pub const DEATH_METAL: Self = Self(22);
    pub const PRANKS: Self = Self(23);
    pub const SOUNDTRACK: Self = Self(24);
    pub const EURO_TECHNO: Self = Self(25);
    pub const AMBIENT: Self = Self(26);
    pub const TRIP_HOP: Self = Self(27);
    pub const VOCAL: Self = Self(28);
    pub const JAZZ_FUNK: Self = Self(29);
    pub const FUSION: Self = Self(30);
    pub const TRANCE: Self = Self(31);
    pub const CLASSICAL: Self = Self(32);
    pub const INSTRUMENTAL: Self = Self(33);
    pub const ACID: Self = Self(34);
    pub const HOUSE: Self = Self(35);
    pub const GAME: Self = Self(36);
    pub const SOUND_CLIP: Self = Self(37);
    pub const GOSPEL: Self = Self(38);
    pub const NOISE: Self = Self(39);
    pub const ALT_ROCK: Self = Self(40);
    pub const BASS: Self = Self(41);
    pub const SOUL: Self = Self(42);
    pub const PUNK: Self = Self(43);
    pub const SPACE: Self = Self(44);
    pub const MEDITATIVE: Self = Self(45);
    pub const INSTRUMENTAL_POP: Self = Self(46);
    pub const INSTRUMENTAL_ROCK: Self = Self(47);
    pub const ETHNIC: Self = Self(48);
    pub const GOTHIC: Self = Self(49);
    pub const DARKWAVE: Self = Self(50);
    pub const TECHNO_INDUSTRIAL: Self = Self(51);
    pub const ELECTRONIC: Self = Self(52);
    pub const POP_FOLK: Self = Self(53);
    pub const EURODANCE: Self = Self(54);
    pub const DREAM: Self = Self(55);
    pub const SOUTHERN_ROCK: Self = Self(56);
    pub const COMEDY: Self = Self(57);
    pub const CULT: Self = Self(58);
    pub const GANGSTA_RAP: Self = Self(59);
    pub const TOP_40: Self = Self(60);
    pub const CHRISTIAN_RAP: Self = Self(61);
    pub const POP_FUNK: Self = Self(62);
    pub const JUNGLE: Self = Self(63);
    pub const NATIVE_AMERICAN: Self = Self(64);
    pub const CABARET: Self = Self(65);
    pub const NEW_WAVE: Self = Self(66);
    pub const PSYCHEDELIC: Self = Self(67);
    pub const RAVE: Self = Self(68);
    pub const SHOWTUNES: Self = Self(69);
    pub const TRAILER: Self = Self(70);
    pub const LO_FI: Self = Self(71);
    pub const TRIBAL: Self = Self(72);
    pub const ACID_PUNK: Self = Self(73);
    pub const ACID_JAZZ: Self = Self(74);
    pub const POLKA: Self = Self(75);
    pub const RETRO: Self = Self(76);
    pub const MUSICAL: Self = Self(77);
    pub const ROCK_N_ROLL: Self = Self(78);
    pub const HARD_ROCK: Self = Self(79);
    pub const FOLK: Self = Self(80);
    pub const FOLK_ROCK: Self = Self(81);
    pub const NATIONAL_FOLK: Self = Self(82);
    pub const SWING: Self = Self(83);
    pub const FAST_FUSION: Self = Self(84);
    pub const BEBOP: Self = Self(85);
    pub const LATIN: Self = Self(86);
    pub const REVIVAL: Self = Self(87);
    pub const CELTIC: Self = Self(88);
    pub const BLUEGRASS: Self = Self(89);
    pub const AVANTGARDE: Self = Self(90);
    pub const GOTHIC_ROCK: Self = Self(91);
    pub const PROGRESSIVE_ROCK: Self = Self(92);
    pub const PSYCHEDELIC_ROCK: Self = Self(93);
    pub const SYMPHONIC_ROCK: Self = Self(94);
    pub const SLOW_ROCK: Self = Self(95);
    pub const BIG_BAND: Self = Self(96);
    pub const CHORUS: Self = Self(97);
    pub const EASY_LISTENING: Self = Self(98);
    pub const ACOUSTIC: Self = Self(99);
    pub const HUMOUR: Self = Self(100);
    pub const SPEECH: Self = Self(101);
    pub const CHANSON: Self = Self(102);
    pub const OPERA: Self = Self(103);
    pub const CHAMBER_MUSIC: Self = Self(104);
    pub const SONATA: Self = Self(105);
    pub const SYMPHONY: Self = Self(106);
    pub const BOOTY_BASS: Self = Self(107);
    pub const PRIMUS: Self = Self(108);
    pub const PORN_GROOVE: Self = Self(109);
    pub const SATIRE: Self = Self(110);
    pub const SLOW_JAM: Self = Self(111);
    pub const CLUB: Self = Self(112);
    pub const TANGO: Self = Self(113);
    pub const SAMBA: Self = Self(114);
    pub const FOLKLORE: Self = Self(115);
    pub const BALLAD: Self = Self(116);
    pub const POWER_BALLAD: Self = Self(117);
    pub const RHYTHMIC_SOUL: Self = Self(118);
    pub const FREESTYLE: Self = Self(119);
    pub const DUET: Self = Self(120);
    pub const PUNK_ROCK: Self = Self(121);
    pub const DRUM_SOLO: Self = Self(122);
    pub const A_CAPPELLA: Self = Self(123);
    pub const EURO_HOUSE: Self = Self(124);
    pub const DANCE_HALL: Self = Self(125);
    pub const GOA: Self = Self(126);
    pub const DRUM_N_BASS: Self = Self(127);
    pub const CLUB_HOUSE: Self = Self(128);
    pub const HARDCORE: Self = Self(129);
    pub const TERROR: Self = Self(130);
    pub const INDIE: Self = Self(131);
    pub const BRIT_POP: Self = Self(132);
    pub const AFRO_PUNK: Self = Self(133);
    pub const POLSK_PUNK: Self = Self(134);
    pub const BEAT: Self = Self(135);
    pub const CHRISTIAN_GANGSTA_RAP: Self = Self(136);
    pub const HEAVY_METAL: Self = Self(137);
    pub const BLACK_METAL: Self = Self(138);
    pub const CROSSOVER: Self = Self(139);
    pub const CONTEMPORARY_CHRISTIAN: Self = Self(140);
    pub const CHRISTIAN_ROCK: Self = Self(141);
    pub const MERENGUE: Self = Self(142);
    pub const SALSA: Self = Self(143);
    pub const THRASH_METAL: Self = Self(144);
    pub const ANIME: Self = Self(145);
    pub const JPOP: Self = Self(146);
    pub const SYNTHPOP: Self = Self(147);
    pub const ABSTRACT: Self = Self(148);
    pub const ART_ROCK: Self = Self(149);
    pub const BAROQUE: Self = Self(150);
    pub const BHANGRA: Self = Self(151);
    pub const BIG_BEAT: Self = Self(152);
    pub const BREAKBEAT: Self = Self(153);
    pub const CHILLOUT: Self = Self(154);
    pub const DOWNTEMPO: Self = Self(155);
    pub const DUB: Self = Self(156);
    pub const EBM: Self = Self(157);
    pub const ECLECTIC: Self = Self(158);
    pub const ELECTRO: Self = Self(159);
    pub const ELECTROCLASH: Self = Self(160);
    pub const EMO: Self = Self(161);
    pub const EXPERIMENTAL: Self = Self(162);
    pub const GARAGE: Self = Self(163);
    pub const GLOBAL: Self = Self(164);
    pub const IDM: Self = Self(165);
    pub const ILLBIENT: Self = Self(166);
    pub const INDUSTRO_GOTH: Self = Self(167);
    pub const JAM_BAND: Self = Self(168);
    pub const KRAUTROCK: Self = Self(169);
    pub const LEFTFIELD: Self = Self(170);
    pub const LOUNGE: Self = Self(171);
    pub const MATH_ROCK: Self = Self(172);
    pub const NEW_ROMANTIC: Self = Self(173);
    pub const NU_BREAKZ: Self = Self(174);
    pub const POST_PUNK: Self = Self(175);
    pub const POST_ROCK: Self = Self(176);
    pub const PSYTRANCE: Self = Self(177);
    pub const SHOEGAZE: Self = Self(178);
    pub const SPACE_ROCK: Self = Self(179);
    pub const TROP_ROCK: Self = Self(180);
    pub const WORLD_MUSIC: Self = Self(181);
    pub const NEOCLASSICAL: Self = Self(182);
    pub const AUDIOBOOK: Self = Self(183);
    pub const AUDIO_THEATRE: Self = Self(184);
    pub const NEUE_DEUTSCHE_WELLE: Self = Self(185);
    pub const PODCAST: Self = Self(186);
    pub const INDIE_ROCK: Self = Self(187);
    pub const G_FUNK: Self = Self(188);
    pub const DUBSTEP: Self = Self(189);
    pub const GARAGE_ROCK: Self = Self(190);
    pub const PSYBIENT: Self = Self(191);

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
pub struct Id3v1 {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub comment: String,
    pub track: Option<u8>,
    pub genre: Option<Genre>,
    pub ext: Option<Ext>,
}

impl Id3v1 {
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
            Some(Ext::decode(buf))
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

    fn decode0(buf: &[u8], ext: Option<Ext>) -> Self {
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
pub struct Ext {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub speed: Option<u8>,
    pub genre: String,
    pub start_time: String,
    pub end_time: String,
}

impl Ext {
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