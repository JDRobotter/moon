from astropy.time import Time
import time
from astropy.coordinates import solar_system_ephemeris, EarthLocation, AltAz
from astropy.coordinates import get_body, get_body_barycentric
import numpy as np
from PIL import Image, ImageDraw


def draw_moon(im, p, angle_degrees, date):
    draw = ImageDraw.Draw(im)
    px, py = p

    B = "yellow"
    A = "grey"

    a = np.radians(angle_degrees)

    if a > np.pi:
        a = a - np.pi
        B, A = A, B

    if a < np.pi / 2:
        dx = 50 * (1.0 - np.cos(a))
        draw.circle((50 + px, 50 + py), 50, fill=A)
        draw.chord(((dx + px, 0 + py), (100 - dx + px, 100 + py)), 90, -90, fill=B)
        draw.chord(((px, py), (100 + px, 100 + py)), -90, +90, fill=B)
    else:
        a = np.pi - a
        dx = 50 * (1.0 - np.cos(a))
        draw.circle((50 + px, 50 + py), 50, fill=B)
        draw.chord(((px, py), (100 + px, 100 + py)), 90, -90, fill=A)
        draw.chord(((dx + px, 0 + py), (100 - dx + px, 100 + py)), -90, 90, fill=A)

    angle_degrees = int(angle_degrees)
    draw.text((px, py), f"{angle_degrees}", fill="red")
    draw.text((px, py + 10), f"{date}", fill="red")


def normalize_0_2pi(a):
    while 1:
        if a < 0:
            a += 2 * np.pi
        elif a > 2 * np.pi:
            a -= 2 * np.pi
        else:
            return a


def main():
    # genangles()
    # gen_dates()
    gen_ephemeris()


def get_elevation(t) -> float:
    """Return moon elevation at given time t in degrees"""
    with solar_system_ephemeris.set("builtin"):

        # get moon position
        loc = EarthLocation.from_geodetic(-0.55, 44.85)
        moon = get_body("moon", t, loc)

        # convert moon position to an elevation / azimuth frame
        altazframe = AltAz(obstime=t, location=loc, pressure=0)
        moonaz = moon.transform_to(altazframe)

        # elevation is in degrees
        elevation = moonaz.alt.degree

        return elevation


def get_shadow_angle(t) -> int:
    with solar_system_ephemeris.set("builtin"):
        # -- fetch every body position at provided time from ephemerids
        moon = get_body_barycentric("moon", t)
        sun = get_body_barycentric("sun", t)
        earth = get_body_barycentric("earth", t)

        # -- compute sun-moon-earth angle
        # get vectors to sun and earth from moon
        v1 = (moon - sun).get_xyz().value
        v2 = (earth - moon).get_xyz().value
        # normalize vectors
        v1n = v1 / np.linalg.norm(v1)
        v2n = v2 / np.linalg.norm(v2)
        # a dot product between the 2 normalized vectors gives us
        # the absolute cosine of angle
        dp = np.dot(v1n, v2n)
        alpha = np.acos(dp)
        # yet angle sign is unknown and can be deduced by checking
        # if the cross product between our two vector is aligned
        # with (arbitrary) up vector or not
        cp = np.cross(v1n, v2n)
        up = np.array([0, 0, 1])
        if np.dot(up, cp) < 0:
            alpha = -alpha

        # reference is the right side of moon shadow
        alpha = -alpha

        # normalize angle
        alpha = normalize_0_2pi(alpha)

    return alpha


def gen_ephemeris():

    # epherids will start at curent unix timestamp
    now = int(time.time())

    # N = 10 * 365 * 24  # 10 years
    N = 24  # 1 day
    T = 3600

    print("use crate::defs::MoonEphemeris;")
    print("pub const MOON_EPHEMERIS: MoonEphemeris = MoonEphemeris {")
    print(f"    start: {now},")
    print(f"    period: {T},")

    # compute moon angles each hour for 10 years
    angles = []
    for h in range(0, N):

        t = Time(now + h * T, format="unix")
        # -- shadow angle
        alpha = get_shadow_angle(t)
        # convert angle to decidegrees
        alpha = int(10 * np.degrees(alpha))

        # -- moon elevation
        elevation = get_elevation(t)
        # elevation will be stored as a i8 in degrees
        # elevation is ranging from -90 to +90 so representation should fit
        elevation = int(elevation)

        angles.append((alpha, elevation))

    # output values
    print("    shadow: &[")
    for a, _ in angles:
        print(f"        {a},")
    print("    ],")

    print("    elevation: &[")
    for _, e in angles:
        print(f"        {e},")
    print("    ],")

    print("};")


def gen_angles():
    # -- TEST draw all angles --
    W, H = (500, 1000)
    with Image.new("RGB", (W, H)) as im:
        x = 0
        y = 0
        for angle in range(0, 360, 10):
            try:
                draw_moon(im, (x, y), angle, "")
            except Exception as e:
                print(e)

            x += 105
            if (x + 100) > W:
                x = 0
                y += 105
    im.save("moon.png")


def gen_dates():

    W, H = 500, 1000
    with Image.new("RGB", (W, H)) as im:
        ix = 0
        iy = 0
        now = time.time()
        for day in range(0, 30):

            t = Time(now + day * 24 * 3600, format="unix")

            alpha = get_shadow_angle(t)

            alpha_degrees = np.degrees(alpha)
            print(alpha_degrees)
            date = str(t.strftime("%Y-%m-%d"))
            draw_moon(im, (ix, iy), alpha_degrees, date)

            ix += 105
            if (ix + 100) > W:
                ix = 0
                iy += 105

    im.save("moon.png")


if __name__ == "__main__":
    main()
