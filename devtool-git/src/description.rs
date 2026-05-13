// Copyright (c) 2023 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE
// LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
// WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
#[derive(Debug, PartialEq)]
pub struct Offset {
    pub commit: String,
    pub count: i32,
}

#[derive(Debug, PartialEq)]
pub struct GitDescription {
    pub description: String,
    pub tag: String,
    pub offset: Option<Offset>,
}

impl GitDescription {
    pub fn parse<S>(s: S) -> Option<Self>
    where
        S: AsRef<str>,
    {
        let s = s.as_ref();
        if s.is_empty() {
            return None;
        }

        if let Some((before_commit, commit)) = s.rsplit_once('-')
            && commit.starts_with('g')
            && let Some((tag, count_str)) = before_commit.rsplit_once('-')
            && let Ok(count) = count_str.parse::<i32>()
            && !tag.is_empty()
        {
            return Some(Self {
                description: String::from(s),
                tag: String::from(tag),
                offset: Some(Offset {
                    commit: String::from(commit),
                    count,
                }),
            });
        }

        Some(Self {
            description: String::from(s),
            tag: String::from(s),
            offset: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{GitDescription, Offset};
    use rstest::rstest;

    #[rstest]
    #[case(None, "")]
    #[case(Some(GitDescription {
        description: String::from("v0.0.21"),
        tag: String::from("v0.0.21"),
        offset: None
    }), "v0.0.21")]
    #[case(Some(GitDescription {
        description: String::from("v0.0.21-1-gdf3eff3"),
        tag: String::from("v0.0.21"),
        offset: Some(Offset {
            commit: String::from("gdf3eff3"),
            count: 1
        })
    }), "v0.0.21-1-gdf3eff3")]
    #[case(Some(GitDescription {
        description: String::from("0.0.21"),
        tag: String::from("0.0.21"),
        offset: None
    }), "0.0.21")]
    #[case(Some(GitDescription {
        description: String::from("0.0.21-5-gabcdef"),
        tag: String::from("0.0.21"),
        offset: Some(Offset {
            commit: String::from("gabcdef"),
            count: 5
        })
    }), "0.0.21-5-gabcdef")]
    #[case(Some(GitDescription {
        description: String::from("v1.0-beta"),
        tag: String::from("v1.0-beta"),
        offset: None
    }), "v1.0-beta")]
    #[case(Some(GitDescription {
        description: String::from("v1.0-beta-3-gabcdef"),
        tag: String::from("v1.0-beta"),
        offset: Some(Offset {
            commit: String::from("gabcdef"),
            count: 3
        })
    }), "v1.0-beta-3-gabcdef")]
    #[case(Some(GitDescription {
        description: String::from("release-2.0-rc1"),
        tag: String::from("release-2.0-rc1"),
        offset: None
    }), "release-2.0-rc1")]
    #[case(Some(GitDescription {
        description: String::from("release-2.0-rc1-7-g1234abc"),
        tag: String::from("release-2.0-rc1"),
        offset: Some(Offset {
            commit: String::from("g1234abc"),
            count: 7
        })
    }), "release-2.0-rc1-7-g1234abc")]
    fn test_basics(#[case] expected_result: Option<GitDescription>, #[case] input: &str) {
        assert_eq!(expected_result, GitDescription::parse(input));
    }
}
