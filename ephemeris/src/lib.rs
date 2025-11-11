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

/// Approximate angle at provided timestamp
/// Returned value is 10x stored unit
pub fn approx_angle_from_unix_timestamp<T: Into<i32> + Copy>(
    data: &MoonEphemeris,
    angles: &[T],
    unix: i64,
) -> Option<i32> {
    // offset timestamp by starting date in ephemerid
    let offset_s = unix - (data.start as i64);

    // if result is negative provided timestamp predate ephemerids
    (offset_s >= 0).then_some(())?;

    // divide offset by ephemerid period to get angle index in array
    let index = i64::div_euclid(offset_s, data.period as i64);
    // get previous  angle in centidegrees
    let pa = *angles.get(index as usize)?;
    let pa: i32 = 10 * pa.into();

    // compute elapsed second since angle entry timestamp
    let elapsed_s = i64::rem_euclid(offset_s, data.period as i64);
    let elapsed_s = elapsed_s as i32;

    // return previous shadow angle if offset is zero
    if elapsed_s == 0 {
        return Some(pa);
    }

    // get next shadow angle in centidegrees
    let na = *angles.get((index + 1) as usize)?;
    let na = 10 * na.into();

    // compute angle delta, get [-pi;pi[ equivalent angle
    let delta = na - pa;
    let delta = modulo_half_half(delta);

    // use a linear ratio between previous and next angle using elapsed time
    let offset = elapsed_s * delta / (data.period as i32);

    Some(pa + offset)
}

/// Return moon shadow angle in centidegrees ranging [0,360[ at provided timestamp
pub fn shadow_angle_from_unix_timestamp(data: &MoonEphemeris, unix: i64) -> Option<u32> {
    approx_angle_from_unix_timestamp(data, data.shadow, unix).map(|angle|
        // returned angle is in centidegrees ranging [0..360deg[
        modulo_full(angle) as u32)
}

/// Return moon elevation angle in decidegrees ranging [-90,90[ at provided timestamp
pub fn elevation_from_unix_timestamp(data: &MoonEphemeris, unix: i64) -> Option<i32> {
    approx_angle_from_unix_timestamp(data, data.elevation, unix)
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
        shadow: &[0, 900, 1800, 2700, 0],
        elevation: &[0, 90, 0, -90, 0],
    };

    fn sa(unix: i64) -> Option<u32> {
        shadow_angle_from_unix_timestamp(&EPHEMERIS, unix)
    }

    fn ev(unix: i64) -> Option<i32> {
        elevation_from_unix_timestamp(&EPHEMERIS, unix)
    }

    #[test]
    fn shadow_simple_known_values() {
        const N: usize = EPHEMERIS.shadow.len();
        const T0: i64 = EPHEMERIS.start as i64;
        const P: u32 = EPHEMERIS.period;

        assert_eq!(sa(T0 + 0 * 3600), Some(0));
        assert_eq!(sa(T0 + 1 * 3600), Some(9000));
        assert_eq!(sa(T0 + 2 * 3600), Some(18000));
        assert_eq!(sa(T0 + 3 * 3600), Some(27000));
        assert_eq!(sa(T0 + 4 * 3600), Some(0));
    }

    #[test]
    fn elevation_simple_known_values() {
        const T0: i64 = EPHEMERIS.start as i64;

        assert_eq!(ev(T0 + 0 * 3600), Some(0));
        assert_eq!(ev(T0 + 1 * 3600), Some(900));
        assert_eq!(ev(T0 + 2 * 3600), Some(0));
        assert_eq!(ev(T0 + 3 * 3600), Some(-900));
        assert_eq!(ev(T0 + 4 * 3600), Some(0));
    }

    #[test]
    fn shadow_simple_intermediate_values() {
        const N: usize = EPHEMERIS.shadow.len();
        const T0: i64 = EPHEMERIS.start as i64;
        const P: u32 = EPHEMERIS.period;

        assert_eq!(sa(T0 + 1 * 30 * 60), Some(4500));
        assert_eq!(sa(T0 + 3 * 30 * 60), Some(13500));
        assert_eq!(sa(T0 + 5 * 30 * 60), Some(22500));
    }

    #[test]
    fn elevation_simple_intermediate_values() {
        const T0: i64 = EPHEMERIS.start as i64;

        assert_eq!(ev(T0 + 1 * 30 * 60), Some(450));
        assert_eq!(ev(T0 + 3 * 30 * 60), Some(450));
        assert_eq!(ev(T0 + 5 * 30 * 60), Some(-450));
        assert_eq!(ev(T0 + 7 * 30 * 60), Some(-450));
    }

    const EPHEMERIS2: MoonEphemeris = MoonEphemeris {
        start: 1234567,
        period: 3600,
        shadow: &[3500, 100],
        elevation: &[],
    };

    fn sa2(unix: i64) -> Option<u32> {
        shadow_angle_from_unix_timestamp(&EPHEMERIS2, unix)
    }

    #[test]
    fn shadow_simple_crossover() {
        const N: usize = EPHEMERIS2.shadow.len();
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
        shadow: &[100, 3500],
        elevation: &[],
    };

    fn sa3(unix: i64) -> Option<u32> {
        shadow_angle_from_unix_timestamp(&EPHEMERIS3, unix)
    }

    #[test]
    fn shadow_backward_crossover() {
        const N: usize = EPHEMERIS3.shadow.len();
        const T0: i64 = EPHEMERIS3.start as i64;
        const P: u32 = EPHEMERIS3.period;

        assert_eq!(sa3(T0 + 0 * 60 * 60), Some(1000));
        assert_eq!(sa3(T0 + 1 * 15 * 60), Some(500));
        assert_eq!(sa3(T0 + 1 * 30 * 60), Some(0));
        assert_eq!(sa3(T0 + 1 * 45 * 60), Some(35500));
        assert_eq!(sa3(T0 + 1 * 60 * 60), Some(35000));
    }

    #[test]
    fn real_values_validity() {
        const N: usize = MOON_EPHEMERIS.shadow.len();
        const T0: i64 = MOON_EPHEMERIS.start as i64;
        const P: u32 = MOON_EPHEMERIS.period;

        // assert all values
        let mut ts = T0;
        for _ in 0..N {
            let a = shadow_angle_from_unix_timestamp(&MOON_EPHEMERIS, ts);
            assert!(a.is_some());
            let a = a.unwrap();
            println!("a={a}");
            assert!(a < 36000);

            let e = elevation_from_unix_timestamp(&MOON_EPHEMERIS, ts);
            assert!(e.is_some());
            let e = e.unwrap();
            println!("e={e}");
            assert!((-900 < e) && (e < 900));

            ts += P as i64;
        }
    }
}
