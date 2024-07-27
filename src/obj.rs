use crate::{primitives::Pos3, Triangle};

pub fn parse_obj(filename: &str) -> std::io::Result<Vec<Triangle<Pos3>>> {
    let mut vertices = Vec::new();
    let mut faces = Vec::<(usize, usize, usize)>::new();

    for l in std::fs::read_to_string(filename)?.lines() {
        let line = l.trim();
        if line.is_empty() {
            // skip emtpy lines
            continue;
        }

        if line.starts_with('#') {
            // skip comment lines
            continue;
        }

        let mut parts = line.split(' ');
        let mut next = || {
            parts
                .next()
                .ok_or(std::io::Error::from(std::io::ErrorKind::Other))
        };
        let indicator = next()?;
        match indicator {
            "f" => {
                faces.push((
                    next()?.parse().expect("int"),
                    next()?.parse().expect("int"),
                    next()?.parse().expect("int"),
                ));
            }
            "v" => {
                vertices.push(Pos3::new(
                    next()?.parse().expect("f64"),
                    next()?.parse().expect("f64"),
                    next()?.parse().expect("f64"),
                ));
            }
            _ => panic!("Unexpected indicator {}", indicator),
        }
    }

    // Obj face indices are one-based, so shift to zero-based array indices
    Ok(faces
        .into_iter()
        .map(|(i1, i2, i3)| Triangle::new(vertices[i1 - 1], vertices[i2 - 1], vertices[i3 - 1]))
        .collect())
}