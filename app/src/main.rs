use tagen::mpeg::Mpeg;

fn main() {
    let rd = std::fs::File::open("lib/testdata/mutagen/id3v1v2-combined.mp3").unwrap();
    let rd = std::io::Cursor::new(unsafe { memmap::Mmap::map(&rd).unwrap() });
    let mpeg = Mpeg::read(rd).unwrap().unwrap();
    let v2 = mpeg.id3v2().unwrap();
    println!("{:#?}", (v2.artist().unwrap_or("?"), v2.title().unwrap_or("?"), v2.album().unwrap_or("?")));
}
