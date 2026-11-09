use logos::Logos;

#[derive(Logos, Clone, Copy, Debug, PartialEq, Eq)]
enum Spelling {
    #[token("main")]
    Main,
    #[token("project/")]
    Project,
    #[token("/")]
    Slash,
    #[regex("[^/]+")]
    Segment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Base {
    Main,
    Project,
    Other,
}

impl Base {
    const MAIN: [Spelling; 1] = [Spelling::Main];
    const PROJECT: [Spelling; 2] = [Spelling::Project, Spelling::Segment];

    fn spelled(lexed: &[Spelling]) -> Base {
        Some(lexed)
            .filter(|whole| *whole == Self::MAIN)
            .map(|_main| Base::Main)
            .or_else(|| {
                lexed.get(..Self::PROJECT.len()).filter(|head| *head == Self::PROJECT).map(|_project| Base::Project)
            })
            .unwrap_or(Base::Other)
    }
}

impl From<&str> for Base {
    fn from(name: &str) -> Base {
        let spelled: Result<Vec<Spelling>, ()> = Spelling::lexer(name).collect();
        Base::spelled(&spelled.unwrap_or_default())
    }
}
