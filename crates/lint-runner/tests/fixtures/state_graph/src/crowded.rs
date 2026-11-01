impl Ajar {
    pub fn a(self) -> Open { Open }
    pub fn b(self) -> Closed { Closed }
    pub fn c(self) -> Ajar { Ajar }
    pub fn d(self) -> Done { Done }
    pub fn e(self) -> Left { Left }
    pub fn f(self) -> Lost { Lost }
    pub fn g(self) -> Kept { Kept }
}
pub struct Done;
pub struct Left;
pub struct Lost;
pub struct Kept;
impl State for Done {}
impl State for Left {}
impl State for Lost {}
impl State for Kept {}
