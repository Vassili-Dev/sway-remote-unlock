use mdns_sd::{ServiceDaemon, ServiceInfo};

use remote_unlock_lib::prelude::*;

pub fn service_host_name(config: &Config) -> Result<ByteArray<{ Config::BUFFER_SIZE }>, Error> {
    let mut buff = ByteArray::<{ Config::BUFFER_SIZE }>::new();
    buff.append_slice(b"remote-unlock.")?;

    buff.append_slice(config.server_hostname().as_bytes())?;
    buff.append_slice(b".local.")?;

    Ok(buff)
}
pub fn start_discovery_daemon(config: &Config) -> Result<ServiceDaemon, Error> {
    let service_host_name_buff = service_host_name(config)?;

    let service_info = ServiceInfo::new(
        config.service_type(),
        config.server_hostname(),
        service_host_name_buff.as_str()?,
        "",
        config.server_port(),
        None,
    )?
    .enable_addr_auto();

    let daemon = ServiceDaemon::new()?;
    daemon.register(service_info)?;

    Ok(daemon)
}
