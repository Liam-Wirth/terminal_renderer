use terminal_renderer::{camera::Camera, cpu_renderer::render_scene};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, terminal,
};
use scene::Scene;
use terminal_renderer::scene;
use std::io::{stdout, Write};
use std::time::Duration;

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
                    KeyCode::Up => camera.move_forward(1.0),
                    KeyCode::Down => camera.move_backward(1.0),
                    KeyCode::Left => camera.turn_left(0.1),
                    KeyCode::Right => camera.turn_right(0.1),
                    _ => {}
                }
            }
        }

        // Render the scene
        render_scene(&mut stdout, &scene, &camera)?;

        stdout.flush()?;
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
