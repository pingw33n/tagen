use std::borrow::Cow;

use crate::timestamp::Timestamp;

#[derive(Debug, Default)]
pub struct TagsRef<'a> {
    pub id3v1: Option<&'a crate::id3::v1::Tag>,
    pub id3v2: Option<&'a crate::id3::v2::Tag>,
    pub vcomment: Option<&'a crate::vcomment::Tag>,
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

    fn choose<T, Id3v1, Id3v2>(&self,
        id3v1: Id3v1,
        id3v2: Id3v2,
    ) -> Option<T>
        where Id3v1: FnOnce(&'a crate::id3::v1::Tag) -> Option<T>,
              Id3v2: FnOnce(&'a crate::id3::v2::Tag) -> Option<T>,
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