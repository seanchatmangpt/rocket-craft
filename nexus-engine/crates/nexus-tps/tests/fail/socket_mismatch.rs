use nexus_tps::poka_yoke::{ArmSocket, LegMount};

fn main() {
    let socket = ArmSocket;
    let mount = LegMount;
    // Attempting ArmSocket.connect(LegMount) must fail to compile due to type mismatch
    let _ = socket.connect(mount);
}
