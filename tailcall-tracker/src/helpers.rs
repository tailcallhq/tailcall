use machineid_rs::{Encryption, HWIDComponent, IdBuilder};
use sysinfo::System;

const PARAPHRASE: &str = "tc_key";
const DEFAULT_CLIENT_ID: &str = "<anonymous>";

pub fn get_client_id() -> String {
    let mut builder = IdBuilder::new(Encryption::SHA256);
    builder
        .add_component(HWIDComponent::SystemID)
        .add_component(HWIDComponent::CPUCores);
    builder
        .build(PARAPHRASE)
        .unwrap_or(DEFAULT_CLIENT_ID.to_string())
}
pub fn get_cpu_cores() -> String {
    let sys = System::new_all();
    sys.physical_core_count().unwrap_or(2).to_string()
}
pub fn get_os_name() -> String {
    System::long_os_version().unwrap_or("Unknown".to_string())
}
