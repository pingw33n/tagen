use std::borrow::Cow;

use crate::id3::v1::Id3v1;
use crate::id3::v2::Id3v2;
use crate::timestamp::Timestamp;
use crate::vcomment::Vcomment;

#[derive(Debug, Default)]
pub struct TagsRef<'a> {
    pub id3v1: Option<&'a Id3v1>,
    pub id3v2: Option<&'a Id3v2>,
    pub vcomment: Option<&'a Vcomment>,
}

impl<'a> TagsRef<'a> {
    pub fn title(&self) -> Option<Cow<str>> {
        self.choose::<&str, _, _>(|v| Some(v.best_title()), |v| v.title()).map(|v| v.into())
    }

    pub fn artist(&self) -> Option<Cow<str>> {
        self.choose::<&str, _, _>(|v| Some(v.best_artist()), |v| v.artist()).map(|v| v.into())
    }

    pub fn album(&self) -> Option<Cow<str>> {
        self.choose::<&str, _, _>(|v| Some(v.best_album()), |v| v.album()).map(|v| v.into())
    }

    pub fn genre(&self) -> Option<Cow<str>> {
        self.choose(
            |v| {
                Some(if let Some(ext) = &v.ext {
                    Cow::Borrowed(ext.genre.as_str())
                } else if let Some(g) = v.genre {
                    if let Some(s) = g.description() {
                        Cow::Borrowed(s)
                    } else {
                        Cow::Owned(g.to_string())
                    }
                } else {
                    return None;
                })
            },
            |v| v.genre().map(|v| v.into()))
    }

    pub fn date(&self) -> Option<Timestamp> {
        self.choose(|v| v.date(), |v| v.release_date())
    }

    fn choose<T, TId3v1, TId3v2>(&self,
        id3v1: TId3v1,
        id3v2: TId3v2,
    ) -> Option<T>
        where TId3v1: FnOnce(&'a Id3v1) -> Option<T>,
              TId3v2: FnOnce(&'a Id3v2) -> Option<T>,
    {
        if let Some(v) = self.id3v2 {
            id3v2(v)
        } else if let Some(v) = self.id3v1 {
            id3v1(v)
        } else {
            None
        }
    }
}