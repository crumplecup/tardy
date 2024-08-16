use tardy::{trace_init, App};
use winit::event_loop;

#[tokio::main]
async fn main() -> polite::Polite<()> {
    trace_init();
    let event_loop = event_loop::EventLoop::new()?;
    event_loop.set_control_flow(event_loop::ControlFlow::Wait);
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
