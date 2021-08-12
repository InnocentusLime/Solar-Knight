// NOTE It's still a separate crate because nobody knows if that's everything
// that the game will have for teams

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Team {
    Hive,
    Earth,
}
