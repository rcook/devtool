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
#[derive(Debug)]
pub struct Offset {
    pub commit: String,
    pub count: i32,
}

#[derive(Debug)]
pub struct GitDescription {
    pub description: String,
    pub tag: String,
    pub offset: Option<Offset>,
}

impl GitDescription {
    pub fn parse(s: &str) -> Option<Self> {
        let parts = s.split('-').collect::<Vec<_>>();

        match parts.len() {
            1 => Some(Self {
                description: String::from(s),
                tag: String::from(parts[0]),
                offset: None,
            }),
            3 => Some(Self {
                description: String::from(s),
                tag: String::from(parts[0]),
                offset: Some(Offset {
                    commit: String::from(parts[2]),
                    count: parts[1].parse::<i32>().ok()?,
                }),
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GitDescription;
    use rstest::rstest;

    #[rstest]
    #[case("v0.0.21-1-gdf3eff3")]
    fn test_basics(#[case] input: &str) {
        GitDescription::parse(input);
    }
}
