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
