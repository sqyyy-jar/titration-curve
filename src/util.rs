pub use self::Either::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}
