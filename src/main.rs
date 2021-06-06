use byteorder::ReadBytesExt;
use rusb::{Context, Device, DeviceHandle, Direction, Result, TransferType, UsbContext};
use std::{io::Cursor, time::Duration};

mod warthog;

fn main() -> Result<()> {
    let mut context = Context::new()?;

    let (mut device, mut handle) = open_device(&mut context, warthog::VID, warthog::PID).expect("Failed to open USB device");

    println!(
        "Found Warthog throttle on bus {}.{}.{}",
        device.bus_number(),
        device.address(),
        device.port_number(),
    );
    print_device_info(&mut handle)?;

    let (readable_endpoints, writable_endpoints) = find_endpoints(&mut device)?;
    let readable_endpoint = readable_endpoints
        .first()
        .expect("No readable endpoint found on device");
    let r_has_kernel_driver = match handle.kernel_driver_active(readable_endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(readable_endpoint.iface)?;
            true
        }
        _ => false,
    };
    println!("has kernel driver? {}", r_has_kernel_driver);
    let writable_endpoint = writable_endpoints
        .first()
        .expect("No readable endpoint found on device");
    let w_has_kernel_driver = match handle.kernel_driver_active(writable_endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(writable_endpoint.iface)?;
            true
        }
        _ => false,
    };
    println!("has kernel driver? {}", w_has_kernel_driver);

    // Claim and configure the device
    configure_endpoint(&mut handle, &readable_endpoint)?;
    // control device here

    let data = read_interrupt(&mut handle, readable_endpoint.address)?;
    println!("{:02X?}", data);

    print_data(data);

    // Cleanup
    handle.release_interface(readable_endpoint.iface)?;
    if r_has_kernel_driver {
        handle.attach_kernel_driver(readable_endpoint.iface)?;
    }

    configure_endpoint(&mut handle, &writable_endpoint)?;

    // write_interrupt(&mut handle, writable_endpoint.address, 0b0001000, 1)?;
    write_interrupt(&mut handle, writable_endpoint.address, 0, 1)?;

    // Cleanup
    handle.release_interface(writable_endpoint.iface)?;
    if w_has_kernel_driver {
        handle.attach_kernel_driver(writable_endpoint.iface)?;
    }

    Ok(())
}

fn open_device<T: UsbContext>(
    context: &mut T,
    vid: u16,
    pid: u16,
) -> Option<(Device<T>, DeviceHandle<T>)> {
    let devices = match context.devices() {
        Ok(d) => d,
        Err(_) => return None,
    };

    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(d) => d,
            Err(_) => continue,
        };

        if device_desc.vendor_id() == vid && device_desc.product_id() == pid {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

fn print_device_info<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<()> {
    let device_desc = handle.device().device_descriptor()?;
    let timeout = Duration::from_secs(1);
    let languages = handle.read_languages(timeout)?;

    println!("Active configuration: {}", handle.active_configuration()?);

    if !languages.is_empty() {
        let language = languages[0];
        println!("Language: {:?}", language);

        println!(
            "Manufacturer: {}",
            handle
                .read_manufacturer_string(language, &device_desc, timeout)
                .unwrap_or("Not found".to_string())
        );
        println!(
            "Product: {}",
            handle
                .read_product_string(language, &device_desc, timeout)
                .unwrap_or("Not found".to_string())
        );
    }
    Ok(())
}

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
}

// returns all readable endpoints for given usb device and descriptor
fn find_endpoints<T: UsbContext>(device: &mut Device<T>) -> Result<(Vec<Endpoint>, Vec<Endpoint>)> {
    let device_desc = device.device_descriptor()?;
    let mut readable_endpoints = vec![];
    let mut writable_endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // println!("{:#?}", config_desc);
        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                // println!("{:#?}", interface_desc);
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    // println!("{:#?}", endpoint_desc);
                    if endpoint_desc.transfer_type() == TransferType::Interrupt {
                        let found = Endpoint {
                            config: config_desc.number(),
                            iface: interface_desc.interface_number(),
                            setting: interface_desc.setting_number(),
                            address: endpoint_desc.address(),
                        };
                        match endpoint_desc.direction() {
                            Direction::In => readable_endpoints.push(found),
                            Direction::Out => writable_endpoints.push(found),
                        }
                    }
                }
            }
        }
    }

    Ok((readable_endpoints, writable_endpoints))
}

fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)
}

const PACKET_DATA_LENGTH: usize = 36;

fn read_interrupt<T: UsbContext>(handle: &mut DeviceHandle<T>, address: u8) -> Result<Vec<u8>> {
    let timeout = Duration::from_secs(1);
    let mut buf = [0u8; PACKET_DATA_LENGTH];

    handle
        .read_interrupt(address, &mut buf, timeout)
        .map(|_| buf.to_vec())
}

fn write_interrupt<T: UsbContext>(handle: &mut DeviceHandle<T>, address: u8, leds: u8, brightness: u8) -> Result<usize> {
    let timeout = Duration::from_secs(1);
    let mut buf = [0u8; PACKET_DATA_LENGTH];

    // buf[PACKET_DATA_LENGTH - 1] = 1;
    // buf[PACKET_DATA_LENGTH - 2] = 6;
    // buf[PACKET_DATA_LENGTH - 3] = leds;
    // buf[PACKET_DATA_LENGTH - 4] = brightness;
    buf[0] = 1;
    buf[1] = 6;
    buf[2] = leds;
    buf[3] = brightness;

    handle.write_interrupt(address, &buf, timeout)
}

fn print_data(data: Vec<u8>) {
    let mut rdr = Cursor::new(data);

    rdr.set_position(26);
    let leds = rdr.read_u8().unwrap_or_default();
    println!("LEDs: {:07b}", leds);

    // Clamped to [0,5], where 0 is off and 5 is the brightest
    rdr.set_position(27);
    let brightness = rdr.read_u8().unwrap_or_default();
    println!("Brightness: {:?}", brightness);
}
