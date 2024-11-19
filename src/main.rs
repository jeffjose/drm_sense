use diretto::{Connector, Device};
use rustix::{
    fs::{self, Mode, OFlags},
    io,
};

fn main() -> Result<(), io::Errno> {
    let fd = fs::open(
        "/dev/dri/card0",
        OFlags::RDWR | OFlags::NONBLOCK,
        Mode::empty(),
    )?;
    let drm_device = unsafe { Device::new_unchecked(fd) };

    println!("Opened device /dev/dri/card0");

    let version = drm_device.version()?;

    println!(
        "Driver: {} ({}) version {}.{}.{} ({})",
        version.name.to_string_lossy(),
        version.desc.to_string_lossy(),
        version.major,
        version.minor,
        version.patchlevel,
        version.date.to_string_lossy()
    );

    let res = drm_device.get_resources()?;

    // Collect available connectors so we don't iterate again later
    let connectors = res
        .connectors
        .iter()
        .map(|id| drm_device.get_connector(*id, true))
        .collect::<io::Result<Vec<Connector>>>()?;

    for connector in &connectors {
        println!(
            "Found connector connecter_id: {} connector_type:{} connector_type_id:{}",
            connector.connector_id, connector.connector_type, connector.connector_type_id
        );

        for (i, (prop, prop_value)) in connector
            .props
            .iter()
            .zip(connector.prop_values.iter())
            .enumerate()
        {
            println!("  Prop: ({}:{})", prop, prop_value)
        }

        for mode in &connector.modes {
            println!(
                "  Found mode {}@{} for connector {}",
                mode.name().to_string_lossy(),
                mode.vertical_refresh_rate(),
                connector.connector_id
            )
        }
    }

    // Find the first connected monitor
    // FIXME: support more monitors
    let connector = connectors
        .into_iter()
        .find(|connector| connector.connection == 1) // 1 means connected
        .unwrap();

    // FIXME: The first mode is usually the prefered one but we should employ a better strategy
    let mode = connector.modes.first().expect("Connector has no modes");

    // This should somehow be passed to wgpu to choose the correct mode
    println!("Refresh rate: {}", mode.wsi_refresh_rate());

    let planes = drm_device.get_plane_resources()?;
    for plane in &planes {
        println!("Plane : {}", plane);
    }
    Ok(())
}
