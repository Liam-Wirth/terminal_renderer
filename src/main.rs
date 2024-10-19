use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, terminal,
};
use std::io::{stdout, Write};
use std::time::Duration;

use terminal_renderer::core::{camera::Camera, scene::Scene};

use terminal_renderer::renderers::cpu_termrenderer::render_scene;

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let mut scene = Scene::new();
    let mut camera = Camera::new();

    loop {
        // Handle input
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break, // Quit the loop
                    // TODO: fps like input, extract into separate file, manage keystate via
                    // hashset
                    KeyCode::Up => camera.move_forward(1.0),
                    KeyCode::Down => camera.move_backward(1.0),
                    KeyCode::Left => camera.turn_left(0.1),
                    KeyCode::Right => camera.turn_right(0.1),

                    KeyCode::Char('w') => camera.move_forward(1.0),
                    KeyCode::Char('s') => camera.move_backward(1.0),
                    KeyCode::Char('a') => camera.strafe_left(1.0),
                    KeyCode::Char('d') => camera.strafe_right(1.0),

                    _ => {}
                }
            }
        }

        // Render the scene
        render_scene(&mut stdout, &scene, &camera, true)?;

        stdout.flush()?;
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
