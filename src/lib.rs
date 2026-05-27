use pyo3::prelude::*;
use pyo3::conversion::IntoPyObjectExt;
use pyo3::types::{PyList, PyModule, PyTuple};

mod shanten;

fn triple_py(py: Python<'_>, values: [i8; 3]) -> PyResult<Py<PyAny>> {
    Ok(PyTuple::new(py, values)?.into_any().unbind())
}

/// Python에서 손 메트릭 평가
///
/// Returns a 16-item tuple:
///   (normal, chiitoi, kokushi, tanyao, (honitsu_man, honitsu_pin, honitsu_sou),
///    yakuhai, pinfu, toitoi, (chinitsu_man, chinitsu_pin, chinitsu_sou),
///    iipeko, sanshoku, ittsu, chanta, junchan, honroutou, shosangen)
#[pyfunction]
fn eval_hand_py(py: Python<'_>, hand: Vec<u8>) -> PyResult<Py<PyAny>> {
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
    let values = vec![
        m.normal_shanten,
        m.chiitoi_shanten,
        m.kokushi_shanten,
        m.tanyao_distance,
    ];
    let mut out: Vec<Py<PyAny>> = values
        .into_iter()
        .map(|value| value.into_py_any(py))
        .collect::<PyResult<_>>()?;
    out.push(triple_py(py, m.honitsu_distance)?);
    out.extend(
        [
            m.yakuhai_distance,
            m.pinfu_distance,
            m.toitoi_distance,
        ]
        .into_iter()
        .map(|value| value.into_py_any(py))
        .collect::<PyResult<Vec<_>>>()?,
    );
    out.push(triple_py(py, m.chinitsu_distance)?);
    out.extend(
        [
            m.iipeko_distance,
            m.sanshoku_distance,
            m.ittsu_distance,
            m.chanta_distance,
            m.junchan_distance,
            m.honroutou_distance,
            m.shosangen_distance,
        ]
        .into_iter()
        .map(|value| value.into_py_any(py))
        .collect::<PyResult<Vec<_>>>()?,
    );

    Ok(PyTuple::new(py, out)?.into_any().unbind())
}

/// Python에서 버림 후보 메트릭 평가
///
/// Returns a list of 17-item tuples:
///   (tile_index, normal, chiitoi, kokushi, tanyao, (h_man,h_pin,h_sou),
///    yakuhai, pinfu, toitoi, (c_man,c_pin,c_sou),
///    iipeko, sanshoku, ittsu, chanta, junchan, honroutou, shosangen)
#[pyfunction]
fn eval_discards_py(py: Python<'_>, hand: Vec<u8>) -> PyResult<Py<PyAny>> {
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
        let mut row: Vec<Py<PyAny>> = vec![d.tile_index.into_py_any(py)?];
        row.extend(
            [
                d.normal_shanten,
                d.chiitoi_shanten,
                d.kokushi_shanten,
                d.tanyao_distance,
            ]
            .into_iter()
            .map(|value| value.into_py_any(py))
            .collect::<PyResult<Vec<_>>>()?,
        );
        row.push(triple_py(py, d.honitsu_distance)?);
        row.extend(
            [
                d.yakuhai_distance,
                d.pinfu_distance,
                d.toitoi_distance,
            ]
            .into_iter()
            .map(|value| value.into_py_any(py))
            .collect::<PyResult<Vec<_>>>()?,
        );
        row.push(triple_py(py, d.chinitsu_distance)?);
        row.extend(
            [
                d.iipeko_distance,
                d.sanshoku_distance,
                d.ittsu_distance,
                d.chanta_distance,
                d.junchan_distance,
                d.honroutou_distance,
                d.shosangen_distance,
            ]
            .into_iter()
            .map(|value| value.into_py_any(py))
            .collect::<PyResult<Vec<_>>>()?,
        );

        out.push(PyTuple::new(py, row)?.into_any().unbind());
    }

    Ok(PyList::new(py, out)?.into_any().unbind())
}

#[pymodule]
fn shanten_pyo(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval_hand_py, m)?)?;
    m.add_function(wrap_pyfunction!(eval_discards_py, m)?)?;
    Ok(())
}
