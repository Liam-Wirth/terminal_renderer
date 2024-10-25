use crossterm::{
    cursor, event::{self, Event, KeyCode, ModifierKeyCode}, execute, style::SetBackgroundColor, terminal
};
use std::io::{stdout, Write};
use std::time::Duration;

use terminal_renderer::{core::{camera::Camera, scene::Scene}, renderers::renderer::{cycle_render_mode, set_render_mode, RenderMode}};

use terminal_renderer::renderers::renderer::get_render_mode;
fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide, SetBackgroundColor(crossterm::style::Color::Rgb{r: 0,b: 0,g: 0}))?;
    terminal::enable_raw_mode()?;

    let mut scene = Scene::new();
    let mut camera = Camera::new();
    set_render_mode(RenderMode::Solid);




    loop {
        // Handle input
        let render_mode = get_render_mode();
        if event::poll(Duration::from_millis(1))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break, // Quit the loop
                    // TODO: fps like input, extract into separate file, manage keystate via
                    // hashset
                    KeyCode::Up => camera.turn_up(0.1),
                    KeyCode::Down => camera.turn_down(0.1),
                    KeyCode::Left => camera.turn_left(0.1),
                    KeyCode::Right => camera.turn_right(0.1),

                    KeyCode::Char('w') => camera.move_forward(1.0),
                    KeyCode::Char('s') => camera.move_backward(1.0),
                    KeyCode::Char('a') => camera.strafe_left(1.0),
                    KeyCode::Char('d') => camera.strafe_right(1.0),

                    KeyCode::Char(' ') => camera.move_up(1.0),
                    KeyCode::Modifier(ModifierKeyCode::LeftControl) => camera.move_down(1.0),


                    KeyCode::Char('p') => cycle_render_mode(),


                    _ => {}
                }
            }
        }

        // Render the scene
        // TODO: move the boolean value for wireframe to an enum member, make it be entity specific
        //render_scene(&mut stdout, &scene, &camera)?;

        stdout.flush()?;
    }

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
