pub struct MoonEphemeris {
    // starting unix timestamp
    pub start: u64,
    // time between each entry in seconds
    pub period: u32,
    // moon shadow angles in decidegrees
    pub shadow: &'static [u16],
    // moon elevation angle in degrees
    pub elevation: &'static [i8],
}
