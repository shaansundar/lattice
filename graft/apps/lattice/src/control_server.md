# apps/lattice/src/control_server.rs

- MAX_LINE · constant · L12-L12 — const MAX_LINE: usize = 64 * 1024 * 1024;
- start · function · L14-L26 — pub fn start(port: u16, ctx: egui::Context) -> std::io::Result<Receiver<ControlRequest>>
- serve · function · L28-L67 — fn serve(stream: TcpStream, tx: Sender<ControlRequest>, ctx: egui::Context)
