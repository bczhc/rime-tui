use rime_tui::tui::TuiApp;
use rime_tui::xinput::XInput;

fn main() -> anyhow::Result<()> {
    let mut app = TuiApp::new()?;
    app.redraw()?;

    let input = XInput::new(None);
    loop {
        let Some(event) = input.next_event() else {
            continue
        };
        app.redraw()?;
    }

    app.stop()?;

    Ok(())
}
