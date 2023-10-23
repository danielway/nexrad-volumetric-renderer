use crate::param::{PointColorMode, VisParams};
use crate::{ColoredPoint, RENDER_RATIO_TO_M};
use three_d::{
    degrees, vec3, ColorMaterial, Context, CpuMaterial, CpuMesh, Gm, InstancedMesh, Mat4, Mesh,
    PhysicalMaterial, PointCloud, Positions, Srgba,
};

const EARTH_RADIUS_M: f32 = 6356752.3;
pub const NEXRAD_RADAR_RANGE_M: f32 = 230000.0;

pub fn get_earth_object(context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    let earth_scaled_radius = EARTH_RADIUS_M * RENDER_RATIO_TO_M;

    let mut earth = Gm::new(
        Mesh::new(context, &CpuMesh::sphere(100)),
        PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 40,
                    g: 100,
                    b: 40,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    earth.set_transformation(
        Mat4::from_translation(vec3(0.0, -earth_scaled_radius, 0.0))
            * Mat4::from_scale(earth_scaled_radius),
    );

    earth
}

pub fn get_radar_indicator_object(context: &Context) -> Gm<Mesh, PhysicalMaterial> {
    let nexrad_radar_diameter_scaled = NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M;

    let mut radar_indicator = Gm::new(
        Mesh::new(context, &CpuMesh::cylinder(100)),
        PhysicalMaterial::new_opaque(
            context,
            &CpuMaterial {
                albedo: Srgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                },
                ..Default::default()
            },
        ),
    );

    radar_indicator.set_transformation(
        Mat4::from_translation(vec3(0.0, 0.0, 0.0))
            * Mat4::from_angle_z(degrees(90.0))
            * Mat4::from_nonuniform_scale(
                0.01,
                nexrad_radar_diameter_scaled,
                nexrad_radar_diameter_scaled,
            ),
    );

    radar_indicator
}

pub fn get_point_cloud_object(
    context: &Context,
    vis_params: &VisParams,
    points: Vec<ColoredPoint>,
) -> Gm<InstancedMesh, ColorMaterial> {
    let mut point_cloud = PointCloud::default();
    point_cloud.positions = Positions::F32(
        points
            .iter()
            .map(|p| vec3(p.pos.x, p.pos.y, p.pos.z))
            .collect::<Vec<_>>(),
    );
    point_cloud.colors = Some(
        points
            .iter()
            .map(|p| {
                let color = match vis_params.point_color_mode {
                    PointColorMode::Raw => p.raw,
                    PointColorMode::Density => p.density,
                    PointColorMode::Hybrid => p.hybrid,
                };

                Srgba::new(color.0, color.1, color.2, 255)
            })
            .collect::<Vec<_>>(),
    );

    let mut point_mesh = CpuMesh::sphere(4);
    point_mesh.transform(&Mat4::from_scale(0.002)).unwrap();

    let point_cloud_gm = Gm {
        geometry: InstancedMesh::new(&context, &point_cloud.into(), &point_mesh),
        material: ColorMaterial::default(),
    };

    point_cloud_gm
}
