mod defs;
pub use defs::MoonEphemeris;

mod data;
pub use data::MOON_EPHEMERIS;

/// Compute full modulo [0;+36000[ of provided angle in centidegrees
fn modulo_full(mut a: i32) -> i32 {
    loop {
        if a < 0 {
            a += 36000;
        } else if a >= 36000 {
            a -= 36000;
        } else {
            return a;
        }
    }
}

/// Compute symetric modulo [-18000;+18000[ of provided angle in centidegrees
fn modulo_half_half(mut a: i32) -> i32 {
    loop {
        if a < -18000 {
            a += 36000;
        } else if a >= 18000 {
            a -= 36000;
        } else {
            return a;
        }
    }
}

/// Compute moon shadow angle at provided timestamp
/// Returned value is in centi-degrees
pub fn shadow_angle_from_unix_timestamp(data: &MoonEphemeris, unix: i64) -> Option<u32> {
    // offset timestamp by starting date in ephemerid
    let offset_s = unix - (data.start as i64);

    // if result is negative provided timestamp predate ephemerids
    (offset_s >= 0).then_some(())?;

    // divide offset by ephemerid period to get angle index in array
    let index = i64::div_euclid(offset_s, data.period as i64);
    // get previous moon angle in centidegrees
    let pa = 10 * *data.angles.get(index as usize)? as i32;

    // compute elapsed second since angle entry timestamp
    let elapsed_s = i64::rem_euclid(offset_s, data.period as i64);
    let elapsed_s = elapsed_s as i32;

    // return previous moon angle if offset is zero
    if elapsed_s == 0 {
        return Some(pa as u32);
    }

    // get next moon angle in centidegrees
    let na = 10 * *data.angles.get((index + 1) as usize)? as i32;

    // compute angle delta, get [-pi;pi[ equivalent angle
    let delta = na - pa;
    let delta = modulo_half_half(delta);

    // use a linear ratio between previous and next angle using elapsed time
    let offset = elapsed_s * delta / (data.period as i32);

    // returned angle is in centidegrees ranging [0..360deg[
    Some(modulo_full(pa + offset) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modulo() {
        assert_eq!(modulo_full(-18000), 18000);
        assert_eq!(modulo_full(-9000), 27000);
        assert_eq!(modulo_full(0), 0);
        assert_eq!(modulo_full(9000), 9000);
        assert_eq!(modulo_full(18000), 18000);
        assert_eq!(modulo_full(36000), 0);
        assert_eq!(modulo_full(19000), 19000);
        assert_eq!(modulo_full(-19000), 17000);

        assert_eq!(modulo_half_half(-18000), -18000);
        assert_eq!(modulo_half_half(-9000), -9000);
        assert_eq!(modulo_half_half(0), 0);
        assert_eq!(modulo_half_half(9000), 9000);
        assert_eq!(modulo_half_half(18000), -18000);
        assert_eq!(modulo_half_half(36000), 0);
        assert_eq!(modulo_half_half(19000), -17000);
        assert_eq!(modulo_half_half(-19000), 17000);
    }

    const EPHEMERIS: MoonEphemeris = MoonEphemeris {
        start: 1234567,
        period: 3600,
        angles: &[0, 900, 1800, 2700, 0],
    };

    fn sa(unix: i64) -> Option<u32> {
        shadow_angle_from_unix_timestamp(&EPHEMERIS, unix)
    }

    #[test]
    fn simple_known_values() {
        const N: usize = EPHEMERIS.angles.len();
        const T0: i64 = EPHEMERIS.start as i64;
        const P: u32 = EPHEMERIS.period;

        assert_eq!(sa(T0 + 0 * 3600), Some(0));
        assert_eq!(sa(T0 + 1 * 3600), Some(9000));
        assert_eq!(sa(T0 + 2 * 3600), Some(18000));
        assert_eq!(sa(T0 + 3 * 3600), Some(27000));
        assert_eq!(sa(T0 + 4 * 3600), Some(0));
    }

    #[test]
    fn simple_intermediate_values() {
        const N: usize = EPHEMERIS.angles.len();
        const T0: i64 = EPHEMERIS.start as i64;
        const P: u32 = EPHEMERIS.period;

        assert_eq!(sa(T0 + 1 * 30 * 60), Some(4500));
        assert_eq!(sa(T0 + 3 * 30 * 60), Some(13500));
        assert_eq!(sa(T0 + 5 * 30 * 60), Some(22500));
    }

    const EPHEMERIS2: MoonEphemeris = MoonEphemeris {
        start: 1234567,
        period: 3600,
        angles: &[3500, 100],
    };

    fn sa2(unix: i64) -> Option<u32> {
        shadow_angle_from_unix_timestamp(&EPHEMERIS2, unix)
    }

    #[test]
    fn simple_crossover() {
        const N: usize = EPHEMERIS2.angles.len();
        const T0: i64 = EPHEMERIS2.start as i64;
        const P: u32 = EPHEMERIS2.period;

        assert_eq!(sa2(T0 + 0 * 60 * 60), Some(35000));
        assert_eq!(sa2(T0 + 1 * 15 * 60), Some(35500));
        assert_eq!(sa2(T0 + 1 * 30 * 60), Some(0));
        assert_eq!(sa2(T0 + 1 * 45 * 60), Some(500));
        assert_eq!(sa2(T0 + 1 * 60 * 60), Some(1000));
    }

    const EPHEMERIS3: MoonEphemeris = MoonEphemeris {
        start: 1234567,
        period: 3600,
        angles: &[100, 3500],
    };

    fn sa3(unix: i64) -> Option<u32> {
        shadow_angle_from_unix_timestamp(&EPHEMERIS3, unix)
    }

    #[test]
    fn backward_crossover() {
        const N: usize = EPHEMERIS3.angles.len();
        const T0: i64 = EPHEMERIS3.start as i64;
        const P: u32 = EPHEMERIS3.period;

        assert_eq!(sa3(T0 + 0 * 60 * 60), Some(1000));
        assert_eq!(sa3(T0 + 1 * 15 * 60), Some(500));
        assert_eq!(sa3(T0 + 1 * 30 * 60), Some(0));
        assert_eq!(sa3(T0 + 1 * 45 * 60), Some(35500));
        assert_eq!(sa3(T0 + 1 * 60 * 60), Some(35000));
    }
}
