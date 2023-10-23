use crate::object::NEXRAD_RADAR_RANGE_M;
use crate::RENDER_RATIO_TO_M;
use std::f32::consts::PI;
use three_d::{
    degrees, vec3, Camera, Context, DirectionalLight, OrbitControl, Srgba, Vec3, Window,
};

pub fn get_camera_and_control(window: &Window) -> (Camera, OrbitControl) {
    let camera = Camera::new_perspective(
        window.viewport(),
        vec3(0.0, 2.0, 5.0),
        vec3(0.0, -2.0, -5.0),
        vec3(0.0, 1.0, 0.0),
        degrees(45.0),
        0.1,
        1000.0,
    );

    let control = OrbitControl::new(Vec3::new(0.0, 0.0, 0.0), 0.01, 10.0);

    (camera, control)
}

pub fn get_sun_light(context: &Context) -> DirectionalLight {
    DirectionalLight::new(context, 1.0, Srgba::WHITE, &vec3(0.0, -1.0, -1.0))
}

pub fn do_auto_orbit(angle_deg: &mut f64, camera: &mut Camera) {
    *angle_deg += 0.2;
    if *angle_deg > 360.0 {
        *angle_deg = 0.0;
    }

    let angle = *angle_deg as f32 * (PI / 180.0);
    let position_x = angle.cos() * NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M * 1.5;
    let position_y = angle.sin() * NEXRAD_RADAR_RANGE_M * RENDER_RATIO_TO_M * 1.5;
    camera.set_view(
        Vec3::new(position_x, 2.0, position_y),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
}
