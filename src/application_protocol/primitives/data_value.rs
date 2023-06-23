pub enum ApplicationDataValue {
    Bool(bool),
    Real(f32),
    Double(f64),
    Date(Date),
    Time(Time),
}

pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub wday: u8, // 1 (Monday) to 7 (Sunday)
}

pub struct Time {
    pub hour: u8,
    pub min: u8,
    pub sec: u8,
    pub hundredths: u8,
}
