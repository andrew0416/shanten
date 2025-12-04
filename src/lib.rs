use pyo3::prelude::*;

mod shanten;

/// Python에서 손 메트릭 평가
///
/// Returns:
///   (normal, chiitoi, kokushi, tanyao, (honitsu_man, honitsu_pin, honitsu_sou))
#[pyfunction]
fn eval_hand_py(hand: Vec<u8>) -> PyResult<(i8, i8, i8, i8, (i8, i8, i8))> {
    if hand.len() != 34 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "hand must be length 34 (0..33 tile counts)",
        ));
    }

    let mut tiles = [0u8; 34];
    for (i, &v) in hand.iter().enumerate() {
        tiles[i] = v;
    }

    let m = shanten::eval_hand(&tiles);
    Ok((
        m.normal_shanten,
        m.chiitoi_shanten,
        m.kokushi_shanten,
        m.tanyao_distance,
        (
            m.honitsu_distance[0],
            m.honitsu_distance[1],
            m.honitsu_distance[2],
        ),
    ))
}

/// Python에서 버림 후보 메트릭 평가
///
/// Returns:
///   List[ (tile_index, normal, chiitoi, kokushi, tanyao, (h_man,h_pin,h_sou)) ]
#[pyfunction]
fn eval_discards_py(
    hand: Vec<u8>,
) -> PyResult<Vec<(u8, i8, i8, i8, i8, (i8, i8, i8))>> {
    if hand.len() != 34 {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "hand must be length 34 (0..33 tile counts)",
        ));
    }

    let mut tiles = [0u8; 34];
    for (i, &v) in hand.iter().enumerate() {
        tiles[i] = v;
    }

    let dm = shanten::eval_discards(&tiles);
    let mut out = Vec::with_capacity(dm.len());

    for d in dm {
        out.push((
            d.tile_index,
            d.normal_shanten,
            d.chiitoi_shanten,
            d.kokushi_shanten,
            d.tanyao_distance,
            (
                d.honitsu_distance[0],
                d.honitsu_distance[1],
                d.honitsu_distance[2],
            ),
        ));
    }

    Ok(out)
}

#[pymodule]
fn shanten_pyo(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval_hand_py, m)?)?;
    m.add_function(wrap_pyfunction!(eval_discards_py, m)?)?;
    Ok(())
}