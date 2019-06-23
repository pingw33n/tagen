#![allow(dead_code)]
#![deny(non_snake_case)]
#![deny(unused_imports)]
#![deny(unused_must_use)]

use clap::{Arg, App};
use humansize::{FileSize, file_size_opts};
use humantime::format_duration;
use memmap::Mmap;
use std::fs::File;
use std::fmt;
use std::io::{self, Cursor, Result, Write};

use tagen::id3;
use tagen::mpeg::{Mpeg, Vbr};
use tagen::meta::*;

struct WithUnit<V, U> {
    v: V,
    unit: U,
}

impl<V, U> WithUnit<V, U> {
    fn new(v: V, unit: U) -> Self {
        Self {
            v,
            unit,
        }
    }
}

impl<V: fmt::Display, U: fmt::Display> fmt::Display for WithUnit<V, U> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.v, self.unit)
    }
}

fn print_line(name: &str, v: impl fmt::Display) {
    println!("{}: {}", name, v);
}

fn print_opt_line<T: fmt::Display>(name: &str, v: Option<T>) {
    if let Some(v) = v {
        print_line(name, v);
    }
}

fn print_sep_line() {
    println!("----------------------------------------");
}

fn print_file(filename: &str) -> Result<()> {
    let file = File::open(filename)?;
    let file_len = file.metadata()?.len();
    let rd = Cursor::new(unsafe { Mmap::map(&file)? });
    let meta = Meta::read(rd)?.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData,
        "unknown format"))?;
    let format = meta.format();

    print_line("File", filename);
    print_line("File Length", file_len.file_size(file_size_opts::CONVENTIONAL).unwrap());

    println!();
    println!("Audio");
    print_sep_line();
    print_line("Format", &format);
    print_line("Duration", format_duration(meta.duration()));
    print_line("Channels", meta.channel_count());
    print_line("Sample Rate", WithUnit::new(meta.samples_per_sec() as f64 / 1000.0, "kHz"));
    print_opt_line("Sample Size", meta.bits_per_sample());
    print_line("Bitrate", WithUnit::new(meta.bits_per_sec() as f64 / 1000.0, "kb/s"));

    println!();
    println!("General tags");
    print_sep_line();
    let tags = meta.tags();

    let mut avail_tags = Vec::new();
    if tags.id3v1.is_some() {
        avail_tags.push("ID3v1".to_owned());
    }
    if let Some(v) = tags.id3v2 {
        avail_tags.push(format!("ID3v{}", v.header().version));
    }
    print_line("Available", avail_tags.join(", "));

    print_opt_line("Title", tags.title());
    print_opt_line("Artist", tags.artist());
    print_opt_line("Album", tags.album());
    print_opt_line("Genre", tags.genre());
    print_opt_line("Date", tags.date());

    match format {
        FormatRef::Mpeg(v) => print_mpeg(&v),
        FormatRef::__Nonexhaustive => unreachable!(),
    }

    Ok(())
}

fn print_mpeg(mpeg: &Mpeg) {
    let h = mpeg.header();
    println!();
    println!("MPEG");
    print_sep_line();
    print_line("Channel Mode", h.channel_mode);
    print_line("CRC Protected", h.crc_protected);
    print_line("Copyrighted", h.copyrighted);
    print_line("Original", h.original);
    print_line("Emphasis", h.emphasis);
    if let Some(v) = mpeg.vbr() {
        match v {
            Vbr::Xing(v) => {
                print_line("XING Header", true);
                print_opt_line("XING Stream Length (frames)", v.stream_len_frames);
                print_opt_line("XING Stream Length (bytes)", v.stream_len_bytes);
                print_opt_line("XING Quality", v.quality);
                print_opt_line("LAME Version", v.lame_version);
                print_line("LAME Header", v.lame.is_some());
            }
            Vbr::Vbri(v) => {
                print_line("VBRI Header", true);
                print_line("VBRI Version", v.version);
                print_line("VBRI Stream Length (frames)", v.stream_len_frames);
                print_line("VBRI Stream Length (bytes)", v.stream_len_bytes);
                print_line("VBRI Quality", v.quality);
            }
        }
    }

    if let Some(v) = mpeg.tags().id3v2 {
        println!();
        println!("ID3v{}", v.header().version);
        print_sep_line();
        print_opt_line("Title", v.title());
        print_opt_line("Artist", v.artist());
        print_opt_line("Album", v.album());
        print_opt_line("Genre", v.genre());
        print_opt_line("Release Date", v.release_date());

        for p in v.pictures() {
            print_line(&format!("Picture [{}]", p.descr),
                format_args!("{}, {}, {}",
                    p.picture_kind,
                    p.content_type,
                    p.data.len().file_size(file_size_opts::CONVENTIONAL).unwrap()));
        }

    }

    if let Some(v) = mpeg.tags().id3v1 {
        println!();
        println!("ID3v1");
        print_sep_line();
        print_opt_line("Title", non_blank(&v.title));
        print_opt_line("Title (ext)", v.ext.as_ref().map(|v| &v.title).and_then(non_blank));
        print_opt_line("Artist", non_blank(&v.artist));
        print_opt_line("Artist (ext)", v.ext.as_ref().map(|v| &v.artist).and_then(non_blank));
        print_opt_line("Album", non_blank(&v.album));
        print_opt_line("Album (ext)", v.ext.as_ref().map(|v| &v.album).and_then(non_blank));
        print_opt_line("Year", non_blank(&v.year));
        print_opt_line("Comment", non_blank(&v.comment));
        print_opt_line("Track", v.track.as_ref());
        if let Some(v) = v.genre {
            print_line("Genre", format_args!("{} ({})", id3::v1::genre_str(v).unwrap_or("?"), v));
        }
        print_opt_line("Genre (ext)", v.ext.as_ref().map(|v| &v.genre).and_then(non_blank));
        if let Some(v) = &v.ext {
            print_opt_line("Speed", v.speed);
            print_opt_line("Start Time", non_blank(&v.start_time));
            print_opt_line("End Time", non_blank(&v.end_time));
        }
    }
}

fn non_blank<T: AsRef<str>>(s: T) -> Option<T> {
    if s.as_ref().trim().is_empty() {
        None
    } else {
        Some(s)
    }
}

fn main() {
    let args = App::new("Tagen")
        .arg(Arg::with_name("input")
            .value_name("INPUT")
            .help("File to use")
            .required(true)
            .multiple(true))
        .get_matches();

    let inputs = args.values_of("input").unwrap();
    for (i, input) in inputs.into_iter().enumerate() {
        if i > 0 {
            println!();
            println!("================================================================================");
        }
        if let Err(e) = print_file(&input) {
            eprintln!("Error analyzing `{}`: {} ({:?})", input, e, e.kind());
        }
    }
}
