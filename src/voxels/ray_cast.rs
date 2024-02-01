use super::{voxel::Voxel, chunks::Chunks};

use nalgebra_glm as glm;

pub fn ray_cast(
    chunks: &Chunks,
    origin: &[f32; 3],
    direction: &[f32; 3],
    radius: f32
)
-> Option<((f32, f32, f32), Option<Voxel>, glm::TVec3<f32>)>
{
    let px = origin[0];
    let py = origin[1];
    let pz = origin[2];

    let dx = direction[0];
    let dy = direction[1];
    let dz = direction[2];

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
        let voxel = chunks.voxel_global((ix as i32, iy as i32, iz as i32).into());
        let id = if let Some(voxel) = voxel {voxel.id} else {0};
        let mut condition = id != 0;
        let block = &chunks.content.blocks[id as usize];
        if block.is_voxel_size() {
            let min_p = block.min_point();
            let min = [ix + min_p.0, iy + min_p.1, iz + min_p.2];
            let max_p = block.max_point();
            let max = [ix+max_p.0, iy+max_p.1, iz+max_p.2];
            condition = condition && intersect_ray_rectangular_parallelepiped(origin, direction, &min, &max);
        }
        if condition {
            let mut face = glm::vec3(0.0, 0.0, 0.0);
			if stepped_index == 0 { face.x = -stepx };
			if stepped_index == 1 { face.y = -stepy };
			if stepped_index == 2 { face.z = -stepz };
            return Some(((ix, iy, iz), voxel, face));
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

fn intersect_ray_rectangular_parallelepiped(
    origin: &[f32; 3],
    direction: &[f32; 3],
    min_point: &[f32; 3],
    max_point: &[f32; 3]
) -> bool {
    let mut tmin = f32::MIN;
    let mut tmax = f32::MAX;

    for i in 0..3 {
        if direction[i].abs() >= f32::EPSILON {
            let lo = (min_point[i] - origin[i]) / direction[i];
            let hi = (max_point[i] - origin[i]) / direction[i];
            tmin = tmin.max(lo.min(hi));
            tmax = tmax.min(lo.max(hi));
        } else if origin[i] < min_point[i] || origin[i] > max_point[i] {
            return false;
        }
    }

    (tmin <= tmax) && (tmax > 0.0)
}