use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, SetBackgroundColor},
    terminal,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io::{stdout, Write},
    path::Path,
    time::Duration,
};

//use nalgebra::{Point3, Vector3, Vector2};
use terminal_renderer::{
    core::{camera::Camera, entity::Entity, scene::Scene},
    renderers::{
        cpu_termrenderer::render_scene,
        renderer::{get_render_mode, set_render_mode, RenderMode},
    },
};

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        Hide,
        SetBackgroundColor(Color::Black)
    )?;
    terminal::enable_raw_mode()?;

    let mut scene = Scene::new();
    let mut camera = Camera::new();

    // Add an entity to the scene for testing, e.g., a cube
    let mut cube = Entity::create_cube();
    let mut dodec = Entity::create_dodecahedron();
    let obj_path = Path::new("assets/models/teapot.obj");
    //let tea = Mesh::from_obj_file_alt(obj_path).unwrap();
    //let mut tea = Entity::from_mesh(tea);
    dodec.transform.translate(0., 0., -3.);
    cube.transform.translate(0., 0., 3.);
    //cube.transform.scale_uniform(1.5);
    let swap = dodec;
    scene.entities.push(swap);
    //scene.entities.push(cube);

    // Set the initial render mode
    set_render_mode(RenderMode::Solid);

    // Game Loop
    let mut pause_spin = false;
    loop {
        // Handle input for controlling the camera
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break, // Quit the loop
                    KeyCode::Up => camera.turn_up(0.1),
                    KeyCode::Down => camera.turn_down(0.1),
                    KeyCode::Left => camera.turn_left(0.1),
                    KeyCode::Right => camera.turn_right(0.1),
                    KeyCode::Char('w') => camera.move_forward(1.0),
                    KeyCode::Char('s') => camera.move_backward(1.0),
                    KeyCode::Char('a') => camera.strafe_left(1.0),
                    KeyCode::Char('d') => camera.strafe_right(1.0),
                    KeyCode::Char(' ') => camera.move_up(1.0),
                    KeyCode::Char('c') => camera.move_down(1.0),
                    KeyCode::Char('p') => set_render_mode(match get_render_mode() {
                        RenderMode::Solid => RenderMode::Wireframe,
                        RenderMode::Wireframe => RenderMode::Solid,
                    }),
                    KeyCode::Char('7') => {
                        pause_spin = !pause_spin;
                    }
                    KeyCode::Char('5') => {}

                    _ => {}
                }
            }
        }

        if !pause_spin {
            scene.entities[0].transform.rotate(0.01, 0.03, 0.05);
            _ = scene.entities[0].transform.get_matrix();
        }
        // Clear the screen and render the scene
        terminal::Clear(terminal::ClearType::All);
        render_scene(&mut stdout, &mut scene, &camera)?;

        stdout.flush()?;
    }

    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
