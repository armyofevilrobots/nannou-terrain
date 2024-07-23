use nannou::{
    noise::{HybridMulti, NoiseFn /*Perlin*/},
    prelude::*,
};
use nannou_egui::{egui, Egui};

/// Decomposes perlin noise into triangle terrain
pub fn generate_terrain(
    span_x: usize,
    span_y: usize,
    spacing: f64,
    altitude: f64,
    height: f64,
    noise_gen: &HybridMulti,
    noise_scale: f64,
) -> Vec<(DVec3, DVec3, DVec3)> {
    let mut triangles: Vec<(DVec3, DVec3, DVec3)> = Vec::new();

    for j in 0..(span_x - 1) {
        for i in 0..(span_y - 1) {
            let px = i as f64 * spacing;
            let py = j as f64 * spacing;
            let (x1, y1, z1) = (
                px,
                py,
                abs(height * noise_gen.get([px * noise_scale, py * noise_scale, altitude * 0.5])),
            );

            let px = (i + 1) as f64 * spacing;
            let py = j as f64 * spacing;
            let (x2, y2, z2) = (
                px,
                py,
                abs(height * noise_gen.get([px * noise_scale, py * noise_scale, altitude * 0.5])),
            );

            let px = (i + 1) as f64 * spacing;
            let py = (j + 1) as f64 * spacing;
            let (x3, y3, z3) = (
                px,
                py,
                abs(height * noise_gen.get([px * noise_scale, py * noise_scale, altitude * 0.5])),
            );

            let px = (i) as f64 * spacing;
            let py = (j + 1) as f64 * spacing;
            let (x4, y4, z4) = (
                px,
                py,
                abs(height * noise_gen.get([px * noise_scale, py * noise_scale, altitude * 0.5])),
            );
            // Push some clockwise triangles (well, svg coord clockwise anyhoo).
            triangles.push((
                DVec3::new(x1, y1, z1),
                DVec3::new(x2, y2, z2),
                DVec3::new(x3, y3, z3),
            ));
            triangles.push((
                DVec3::new(x1, y1, z1),
                DVec3::new(x3, y3, z3),
                DVec3::new(x4, y4, z4),
            ));
        }
    }
    triangles
}


pub fn triangle_slope_decompose(
    triangle: &(DVec3, DVec3, DVec3),
    max_dz: f64,
) -> Vec<(DVec3, DVec3, DVec3)> {
    // Just stupid brute force here..
    if abs(triangle.0.z - triangle.1.z) > max_dz {
        // println!("Splitting on line 0-1");
        let mid = DVec3::new(
            (triangle.0.x + triangle.1.x) / 2.,
            (triangle.0.y + triangle.1.y) / 2.,
            (triangle.0.z + triangle.1.z) / 2.,
        );
        let tri1 = (mid.clone(), triangle.1.clone(), triangle.2.clone());
        let mut tri2 = (mid, triangle.2.clone(), triangle.0.clone());
        // println!("Pre decomposition of 0-1 is: {:?} and {:?}", &tri1, &tri2);
        // let mut tri1_dec = triangle_slope_decompose(&tri1, max_dz);
        // let mut tri2_dec = triangle_slope_decompose(&tri2, max_dz);

        // tri1_dec.append(&mut tri2_dec);
        // println!("Post decomposition of 0-1 is: {:?}", &tri1_dec);
        // tri1_dec
        vec![tri1, tri2]
    } else if abs(triangle.1.z - triangle.2.z) > max_dz {
        // println!("Splitting on 1-2");
        let tri1 = (
            triangle.0.clone(),
            DVec3::new(
                (triangle.1.x + triangle.2.x) / 2.,
                (triangle.1.y + triangle.2.y) / 2.,
                (triangle.1.z + triangle.2.z) / 2.,
            ),
            triangle.2.clone(),
        );
        let mut tri2 = (
            triangle.1.clone(),
            DVec3::new(
                (triangle.1.x + triangle.2.x) / 2.,
                (triangle.1.y + triangle.2.y) / 2.,
                (triangle.1.z + triangle.2.z) / 2.,
            ),
            triangle.0.clone(),
        );
        // let mut tri1_dec = triangle_slope_decompose(&tri1, max_dz);
        // let mut tri2_dec = triangle_slope_decompose(&tri2, max_dz);
        // tri1_dec.append(&mut tri2_dec);
        // tri1_dec
        vec![tri1, tri2]
    } else {
        // println!("Nope. S'OK!");
        vec![triangle.clone()]
    }
}

pub fn line_plane_intersections(line: &(DVec3, DVec3), plane_z: f64) -> Option<DVec3> {
    let (point0, point1) = if line.0.z <= line.1.z {
        (line.0, line.1)
    } else {
        (line.1, line.0)
    };

    if point0.z <= plane_z && plane_z <= point1.z {
        let dy = point1.y - point0.y;
        let dx = point1.x - point0.x;
        let delta = (plane_z - point0.z) / (point1.z - point0.z);
        Some(DVec3::new(
            point0.x + dx * delta,
            point0.y + dy * delta,
            plane_z,
        ))
    } else {
        None
    }
}

pub fn lines_from_terrain(triangles: &Vec<(DVec3, DVec3, DVec3)>, plane_z: f64) -> Vec<(DVec3, DVec3)> {
    let mut all_lines = Vec::<(DVec3, DVec3)>::new();
    for triangle in triangles {
        let mut points = vec![
            line_plane_intersections(
                &(
                    DVec3::new(triangle.0.x, triangle.0.y, triangle.0.z),
                    DVec3::new(triangle.1.x, triangle.1.y, triangle.1.z),
                ),
                plane_z,
            ),
            line_plane_intersections(
                &(
                    DVec3::new(triangle.1.x, triangle.1.y, triangle.1.z),
                    DVec3::new(triangle.2.x, triangle.2.y, triangle.2.z),
                ),
                plane_z,
            ),
            line_plane_intersections(
                &(
                    DVec3::new(triangle.2.x, triangle.2.y, triangle.2.z),
                    DVec3::new(triangle.0.x, triangle.0.y, triangle.0.z),
                ),
                plane_z,
            ),
        ];
        // First collect the not null points
        let points: Vec<DVec3> = points
            .iter()
            .filter(|point| point.is_some())
            .map(|point| point.unwrap())
            .collect();
        if points.len() >= 2 {
            // Then convert them to a tuple
            let lines = (*points.first().unwrap(), *points.last().unwrap());
            all_lines.push(lines)
        }
    }
    all_lines
}



#[cfg(test)]
mod test {
    use crate::{line_plane_intersections, triangle_slope_decompose};
    use nannou::prelude::DVec3;

    #[test]
    fn test_line_plane_intersections() {
        let line = (DVec3::new(0., 0., 0.), DVec3::new(10., 0., 10.));
        let intersection = line_plane_intersections(&line, 5.);
        assert!(intersection.unwrap().x == 5.);

        let line = (DVec3::new(0., 0., 0.), DVec3::new(10., 0., -10.));
        let intersection = line_plane_intersections(&line, -5.);
        println!("INTERSECTION: {:?}", intersection.unwrap());
        assert!(intersection.unwrap().z == -5.);
    }

    #[test]
    fn test_triangle_slope_decomposition() {
        let tri = (
            DVec3::new(0., 0., 0.),
            DVec3::new(10., 5., 5.),
            DVec3::new(10., -5., -5.),
        );
        println!("TRI OUT: {:?}", triangle_slope_decompose(&tri, 2.));
        let tri = (
            DVec3::new(0., 0., 0.),
            DVec3::new(10., 5., 15.),
            DVec3::new(10., -5., -15.),
        );
        println!("TRI OUT: {:?}", triangle_slope_decompose(&tri, 2.));
    }
}
