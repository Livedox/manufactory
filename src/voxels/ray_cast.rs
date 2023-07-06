use super::{voxel::Voxel, chunks::Chunks};

use nalgebra_glm as glm;

pub fn ray_cast<'a>(
    chunks: &'a Chunks,
    origin: &glm::Vec3,
    direction: &glm::Vec3,
    radius: f32
)
-> Option<(f32, f32, f32, Option<&'a Voxel>, glm::TVec3<f32>)>
{
    let px = origin.x;
    let py = origin.y;
    let pz = origin.z;

    let dx = direction.x;
    let dy = direction.y;
    let dz = direction.z;

    let mut t = 0.0;
    let mut ix = px.floor();
    let mut iy = py.floor();
    let mut iz = pz.floor();

    let stepx = if dx > 0.0 { 1.0 } else { -1.0 };
    let stepy = if dy > 0.0 { 1.0 } else { -1.0 };
    let stepz = if dz > 0.0 { 1.0 } else { -1.0 };

    let tx_delta = if dx == 0.0 { f32::INFINITY } else { (1.0 / dx).abs() };
    let ty_delta = if dy == 0.0 { f32::INFINITY } else { (1.0 / dy).abs() };
    let tz_delta = if dz == 0.0 { f32::INFINITY } else { (1.0 / dz).abs() };

    let xdist = if stepx > 0.0 { ix + 1.0 - px } else { px - ix };
    let ydist = if stepy > 0.0 { iy + 1.0 - py } else { py - iy };
    let zdist = if stepz > 0.0 { iz + 1.0 - pz } else { pz - iz };

    let mut tx_max = if tx_delta < f32::INFINITY { tx_delta * xdist } else { f32::INFINITY };
    let mut ty_max = if ty_delta < f32::INFINITY { ty_delta * ydist } else { f32::INFINITY };
    let mut tz_max = if tz_delta < f32::INFINITY { tz_delta * zdist } else { f32::INFINITY };

    let mut stepped_index = -1;

    while t <= radius {
        let voxel = chunks.voxel_global(ix as i32, iy as i32, iz as i32);
        let id = if let Some(voxel) = voxel {voxel.id} else {0};
        if id != 0 {
            let mut face = glm::vec3(0.0, 0.0, 0.0);
			if stepped_index == 0 { face.x = -stepx };
			if stepped_index == 1 { face.y = -stepy };
			if stepped_index == 2 { face.z = -stepz };
            return Some((ix, iy, iz, voxel, face));
        }

        if tx_max < ty_max {
			if tx_max < tz_max {
				ix += stepx;
				t = tx_max;
				tx_max += tx_delta;
				stepped_index = 0;
			} else {
				iz += stepz;
				t = tz_max;
				tz_max += tz_delta;
				stepped_index = 2;
			}
		} else if ty_max < tz_max {
  				iy += stepy;
  				t = ty_max;
  				ty_max += ty_delta;
  				stepped_index = 1;
  			} else {
  				iz += stepz;
  				t = tz_max;
  				tz_max += tz_delta;
  				stepped_index = 2;
  			}
    }
    None
}