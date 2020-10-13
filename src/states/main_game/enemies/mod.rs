pub mod brute;
pub mod tester;

pub enum Enemy {
    Brute(brute::Brute),
    Tester(tester::Tester),
}

pub struct Hive {

}
