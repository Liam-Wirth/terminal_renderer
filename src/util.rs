use glam::Mat4;

pub fn format_mat4(name: &str, mat: &Mat4) -> String {
    let mut output = String::new();

    output.push_str(&format!("{}:\n", name));

    output.push_str(&format!(
        "x_axis: [{:8.3},{:8.3},{:8.3},{:8.3}]\n",
        mat.x_axis.x, mat.x_axis.y, mat.x_axis.z, mat.x_axis.w
    ));

    output.push_str(&format!(
        "y_axis: [{:8.3},{:8.3},{:8.3},{:8.3}]\n",
        mat.y_axis.x, mat.y_axis.y, mat.y_axis.z, mat.y_axis.w
    ));

    output.push_str(&format!(
        "z_axis: [{:8.3},{:8.3},{:8.3},{:8.3}]\n",
        mat.z_axis.x, mat.z_axis.y, mat.z_axis.z, mat.z_axis.w
    ));

    output.push_str(&format!(
        "w_axis: [{:8.3},{:8.3},{:8.3},{:8.3}]\n",
        mat.w_axis.x, mat.w_axis.y, mat.w_axis.z, mat.w_axis.w
    ));

    output
}
