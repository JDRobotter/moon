pub struct MoonEphemeris {
    // starting unix timestamp
    pub start: u64,
    // time between each entry in seconds
    pub period: u32,
    // moon angles in decidegrees
    pub angles: &'static [u16],
}
