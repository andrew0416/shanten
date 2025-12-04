//! Standalone shanten & metrics engine, ported from Mortal/libriichi.

use std::io::Read;
use std::sync::LazyLock;

use flate2::read::GzDecoder;

const JIHAI_TABLE_SIZE: usize = 78_032;
const SUHAI_TABLE_SIZE: usize = 1_940_777;

static JIHAI_TABLE: LazyLock<Vec<[u8; 10]>> = LazyLock::new(|| {
    read_table(
        include_bytes!("data/shanten_jihai.bin.gz"),
        JIHAI_TABLE_SIZE,
    )
});
static SUHAI_TABLE: LazyLock<Vec<[u8; 10]>> = LazyLock::new(|| {
    read_table(
        include_bytes!("data/shanten_suhai.bin.gz"),
        SUHAI_TABLE_SIZE,
    )
});

fn read_table(gzipped: &[u8], length: usize) -> Vec<[u8; 10]> {
    let mut gz = GzDecoder::new(gzipped);
    let mut raw = vec![];
    gz.read_to_end(&mut raw).unwrap();

    let mut ret = Vec::with_capacity(length);
    let mut entry = [0; 10];
    for (i, b) in raw.into_iter().enumerate() {
        entry[i * 2 % 10] = b & 0b1111;
        entry[i * 2 % 10 + 1] = (b >> 4) & 0b1111;
        if (i + 1) % 5 == 0 {
            ret.push(entry);
        }
    }
    assert_eq!(ret.len(), length);

    ret
}

pub fn ensure_init() {
    assert_eq!(JIHAI_TABLE.len(), JIHAI_TABLE_SIZE);
    assert_eq!(SUHAI_TABLE.len(), SUHAI_TABLE_SIZE);
}

fn add_suhai(lhs: &mut [u8; 10], index: usize, m: usize) {
    let tab = SUHAI_TABLE.get(index).copied().unwrap_or_default();

    for j in (5..=(5 + m)).rev() {
        let mut sht = (lhs[j] + tab[0]).min(lhs[0] + tab[j]);
        for k in 5..j {
            sht = sht.min(lhs[k] + tab[j - k]).min(lhs[j - k] + tab[k]);
        }
        lhs[j] = sht;
    }

    for j in (0..=m).rev() {
        let mut sht = lhs[j] + tab[0];
        for k in 0..j {
            sht = sht.min(lhs[k] + tab[j - k]);
        }
        lhs[j] = sht;
    }
}

fn add_jihai(lhs: &mut [u8; 10], index: usize, m: usize) {
    let tab = JIHAI_TABLE.get(index).copied().unwrap_or_default();

    let j = m + 5;
    let mut sht = (lhs[j] + tab[0]).min(lhs[0] + tab[j]);
    for k in 5..j {
        sht = sht.min(lhs[k] + tab[j - k]).min(lhs[j - k] + tab[k]);
    }
    lhs[j] = sht;
}

fn sum_tiles(tiles: &[u8]) -> usize {
    tiles.iter().fold(0, |acc, &x| acc * 5 + x as usize)
}

/// `len_div3` must be within [0, 4].
#[must_use]
pub fn calc_normal(tiles: &[u8; 34], len_div3: u8) -> i8 {
    let len_div3 = len_div3 as usize;

    let mut ret = SUHAI_TABLE
        .get(sum_tiles(&tiles[..9]))
        .copied()
        .unwrap_or_default();
    add_suhai(&mut ret, sum_tiles(&tiles[9..2 * 9]), len_div3);
    add_suhai(&mut ret, sum_tiles(&tiles[2 * 9..3 * 9]), len_div3);
    add_jihai(&mut ret, sum_tiles(&tiles[3 * 9..]), len_div3);

    (ret[5 + len_div3] as i8) - 1
}

#[must_use]
pub fn calc_chitoi(tiles: &[u8; 34]) -> i8 {
    let mut pairs = 0;
    let mut kinds = 0;
    tiles.iter().filter(|&&c| c > 0).for_each(|&c| {
        kinds += 1;
        if c >= 2 {
            pairs += 1;
        }
    });

    let redunct = 7_u8.saturating_sub(kinds) as i8;
    7 - pairs + redunct - 1
}

#[must_use]
pub fn calc_kokushi(tiles: &[u8; 34]) -> i8 {
    // 1m, 9m, 1p, 9p, 1s, 9s, E, S, W, N, P, F, C
    const TERMINAL_INDICES: [usize; 13] = [
        0, 8,   // 1m, 9m
        9, 17,  // 1p, 9p
        18, 26, // 1s, 9s
        27, 28, 29, 30, 31, 32, 33, // 자패 7종
    ];

    let mut pairs = 0;
    let mut kinds = 0;

    for &i in TERMINAL_INDICES.iter() {
        let c = tiles[i];
        if c > 0 {
            kinds += 1;
            if c >= 2 {
                pairs += 1;
            }
        }
    }

    let redunct = (pairs > 0) as i8;
    14 - kinds - redunct - 1
}

#[must_use]
pub fn calc_all(tiles: &[u8; 34], len_div3: u8) -> i8 {
    let mut shanten = calc_normal(tiles, len_div3);
    if shanten <= 0 || len_div3 < 4 {
        return shanten;
    }

    shanten = shanten.min(calc_chitoi(tiles));
    if shanten > 0 {
        shanten.min(calc_kokushi(tiles))
    } else {
        shanten
    }
}

// =========================
// Extra metrics (hand / discard)
// =========================

#[derive(Debug, Clone, Copy)]
pub struct HandMetrics {
    pub normal_shanten: i8,
    pub chiitoi_shanten: i8,
    pub kokushi_shanten: i8,
    pub tanyao_distance: i8,
    /// [man, pin, sou]
    pub honitsu_distance: [i8; 3],
}

#[derive(Debug, Clone, Copy)]
pub struct DiscardMetrics {
    /// 0..33 (1m..9m, 1p..9p, 1s..9s, 자패 7종)
    pub tile_index: u8,
    pub normal_shanten: i8,
    pub chiitoi_shanten: i8,
    pub kokushi_shanten: i8,
    pub tanyao_distance: i8,
    /// [man, pin, sou]
    pub honitsu_distance: [i8; 3],
}

// ----- index helpers -----

#[inline]
fn is_terminal_or_honor(idx: usize) -> bool {
    matches!(
        idx,
        0 | 8 |       // 1m, 9m
        9 | 17 |      // 1p, 9p
        18 | 26 |     // 1s, 9s
        27..=33       // 자패
    )
}

#[inline]
fn is_man(idx: usize) -> bool {
    idx <= 8
}

#[inline]
fn is_pin(idx: usize) -> bool {
    (9..=17).contains(&idx)
}

#[inline]
fn is_sou(idx: usize) -> bool {
    (18..=26).contains(&idx)
}

#[inline]
fn is_honor(idx: usize) -> bool {
    idx >= 27
}

/// suit: 0=만, 1=통, 2=삭
#[inline]
fn is_main_suit(idx: usize, suit: u8) -> bool {
    match suit {
        0 => is_man(idx),
        1 => is_pin(idx),
        2 => is_sou(idx),
        _ => false,
    }
}

// ----- tanyao distance -----

/// 탕야오에 얼마나 가까운지 distance (작을수록 좋음).
///
/// 정의:
///   - T = 1·9·자패 개수 (탕야오에서 반드시 제거해야 하는 패 수)
///   - mid_only = 2~8 수패만 남긴 손
///   - shanten_mid = calc_all(mid_only, len_div3_mid)
///   => distance = T + shanten_mid
#[must_use]
pub fn tanyao_distance(tiles: &[u8; 34]) -> i8 {
    let mut mid_only = [0u8; 34];
    let mut t_count: u8 = 0;

    for (i, &c) in tiles.iter().enumerate() {
        if c == 0 {
            continue;
        }
        if is_terminal_or_honor(i) {
            t_count = t_count.saturating_add(c);
        } else {
            mid_only[i] = c;
        }
    }

    let count_mid: u16 = mid_only.iter().map(|&x| x as u16).sum();
    if count_mid == 0 {
        // 전부 T뿐이면 탕야오 하려면 전부 갈아껴야 하므로
        // 약간 큰 상수 더해줌 (튜닝 포인트)
        return t_count as i8 + 8;
    }

    let len_div3_mid: u8 = (count_mid / 3) as u8;
    let shanten_mid = calc_all(&mid_only, len_div3_mid);

    t_count as i8 + shanten_mid
}

// ----- honitsu distance -----

/// 혼일색(한 슈트+자패) distance (작을수록 좋음).
/// suit: 0=만, 1=통, 2=삭
///
/// 정의:
///   - off_color = 해당 슈트를 제외한 다른 수패들의 개수 (언젠가 제거해야 함)
///   - filtered = 해당 슈트+자패만 남긴 손
///   - shanten_filtered = calc_all(filtered, len_div3_f)
///   => distance = off_color + shanten_filtered
#[must_use]
pub fn honitsu_distance_for_suit(tiles: &[u8; 34], suit: u8) -> i8 {
    let mut filtered = [0u8; 34];
    let mut off_color: u8 = 0;

    for (i, &c) in tiles.iter().enumerate() {
        if c == 0 {
            continue;
        }
        let main = is_main_suit(i, suit);
        let jihai = is_honor(i);
        if main || jihai {
            filtered[i] = c;
        } else {
            off_color = off_color.saturating_add(c);
        }
    }

    let count_f: u16 = filtered.iter().map(|&x| x as u16).sum();
    if count_f == 0 {
        // 해당 슈트+자패가 아예 없으면 혼일색으로는 사실상 불가능
        return off_color as i8 + 8;
    }

    let len_div3_f: u8 = (count_f / 3) as u8;
    let shanten_filtered = calc_all(&filtered, len_div3_f);

    off_color as i8 + shanten_filtered
}

// ----- high-level eval -----

#[must_use]
pub fn eval_hand(tiles: &[u8; 34]) -> HandMetrics {
    let count: u16 = tiles.iter().map(|&x| x as u16).sum();
    let len_div3: u8 = (count / 3) as u8;

    let normal = calc_normal(tiles, len_div3);
    let chiitoi = calc_chitoi(tiles);
    let kokushi = calc_kokushi(tiles);
    let tanyao = tanyao_distance(tiles);
    let honitsu = [
        honitsu_distance_for_suit(tiles, 0),
        honitsu_distance_for_suit(tiles, 1),
        honitsu_distance_for_suit(tiles, 2),
    ];

    HandMetrics {
        normal_shanten: normal,
        chiitoi_shanten: chiitoi,
        kokushi_shanten: kokushi,
        tanyao_distance: tanyao,
        honitsu_distance: honitsu,
    }
}

#[must_use]
pub fn eval_discards(tiles: &[u8; 34]) -> Vec<DiscardMetrics> {
    let mut result = Vec::new();

    for i in 0..34 {
        if tiles[i] == 0 {
            continue;
        }

        // i 번 타일을 1장 버린다고 가정 → 새로운 손 구성
        let mut tmp = *tiles;
        tmp[i] -= 1;

        let count: u16 = tmp.iter().map(|&x| x as u16).sum();
        let len_div3: u8 = (count / 3) as u8;

        let normal = calc_normal(&tmp, len_div3);
        let chiitoi = calc_chitoi(&tmp);
        let kokushi = calc_kokushi(&tmp);
        let tanyao = tanyao_distance(&tmp);
        let honitsu = [
            honitsu_distance_for_suit(&tmp, 0),
            honitsu_distance_for_suit(&tmp, 1),
            honitsu_distance_for_suit(&tmp, 2),
        ];

        result.push(DiscardMetrics {
            tile_index: i as u8,
            normal_shanten: normal,
            chiitoi_shanten: chiitoi,
            kokushi_shanten: kokushi,
            tanyao_distance: tanyao,
            honitsu_distance: honitsu,
        });
    }

    result
}
