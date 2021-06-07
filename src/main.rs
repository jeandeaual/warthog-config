use rusb::Context;
use std::fmt;

mod usb;
mod warthog;

struct CustomError(String);

impl fmt::Debug for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn main() -> Result<(), CustomError> {
    let mut context = Context::new()
        .map_err(|err| CustomError(format!("can't create a USB context: {}", err)))?;

    // Open the USB device
    let (mut device, mut handle) = usb::open_device(
        &mut context,
        warthog::VID,
        warthog::THROTTLE_PID
    ).expect("Failed to open USB device");

    println!(
        "Found Warthog throttle on bus {}.{}.{}",
        device.bus_number(),
        device.address(),
        device.port_number(),
    );

    if cfg!(debug_assertions) {
        usb::print_device_info(&mut handle)
            .map_err(|err| CustomError(format!("can't print the device info: {}", err)))?;
    }

    // Get the USB endpoints for reading and writing
    let (readable_endpoints, writable_endpoints) = usb::find_endpoints(&mut device)
        .map_err(|err| CustomError(format!("can't find USB endpoints: {}", err)))?;
    let readable_endpoint = readable_endpoints
        .first()
        .expect("No readable endpoint found on device");
    let r_endpoint_has_kernel_driver = match handle.kernel_driver_active(readable_endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(readable_endpoint.iface)
                .map_err(|err| CustomError(format!("can't detach kernel driver for interface {}: {}", readable_endpoint.iface, err)))?;
            true
        }
        _ => false,
    };
    let writable_endpoint = writable_endpoints
        .first()
        .expect("No readable endpoint found on device");
    let w_endpoint_has_kernel_driver = match handle.kernel_driver_active(writable_endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(writable_endpoint.iface)
                .map_err(|err| CustomError(format!("can't detach kernel driver for interface {}: {}", writable_endpoint.iface, err)))?;
            true
        }
        _ => false,
    };

    // Claim and configure the device
    usb::configure_endpoint(&mut handle, &readable_endpoint)
        .map_err(|err| CustomError(format!("can't configure readable endpoint: {}", err)))?;

    let data = usb::read_interrupt(&mut handle, readable_endpoint.address)
        .map_err(|err| CustomError(format!("can't read interrupt: {}", err)))?;
    println!("{:02X?}", data);

    usb::print_data(data);

    // Cleanup
    handle.release_interface(readable_endpoint.iface)
        .map_err(|err| CustomError(format!("can't release the readable interface: {}", err)))?;
    if r_endpoint_has_kernel_driver {
        handle.attach_kernel_driver(readable_endpoint.iface)
            .map_err(|err| CustomError(format!("can't attach the kernel driver on the readable interface: {}", err)))?;
    }

    // Claim and configure the device
    usb::configure_endpoint(&mut handle, &writable_endpoint)
        .map_err(|err| CustomError(format!("can't configure readable endpoint: {}", err)))?;

    // Disable the LEDs and set the brightness level to 1
    usb::write_interrupt(&mut handle, writable_endpoint.address, warthog::ThrottleLEDState::empty(), 1)
        .map_err(|err| CustomError(format!("can't write interrupt: {}", err)))?;

    // Cleanup
    handle.release_interface(writable_endpoint.iface)
        .map_err(|err| CustomError(format!("can't release the writable interface: {}", err)))?;
    if w_endpoint_has_kernel_driver {
        handle.attach_kernel_driver(writable_endpoint.iface)
            .map_err(|err| CustomError(format!("can't attach the kernel driver on the writable interface: {}", err)))?;
    }

    Ok(())
}
