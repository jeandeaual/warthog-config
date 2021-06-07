use byteorder::ReadBytesExt;
use rusb::{Device, DeviceHandle, Direction, Result, TransferType, UsbContext};
use std::{io::Cursor, time::Duration};

use crate::warthog::ThrottleLEDState;

pub fn open_device<T: UsbContext>(
    context: &mut T,
    vendor_id: u16,
    product_id: u16,
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

        if device_desc.vendor_id() == vendor_id && device_desc.product_id() == product_id {
            match device.open() {
                Ok(handle) => return Some((device, handle)),
                Err(_) => continue,
            }
        }
    }

    None
}

pub fn print_device_info<T: UsbContext>(handle: &mut DeviceHandle<T>) -> Result<()> {
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
                .unwrap_or_else(|_| "Not found".to_string())
        );
        println!(
            "Product: {}",
            handle
                .read_product_string(language, &device_desc, timeout)
                .unwrap_or_else(|_| "Not found".to_string())
        );
    }

    Ok(())
}

#[derive(Debug)]
pub struct Endpoint {
    pub config: u8,
    pub iface: u8,
    pub setting: u8,
    pub address: u8,
}

// returns all readable endpoints for given usb device and descriptor
pub fn find_endpoints<T: UsbContext>(device: &mut Device<T>) -> Result<(Vec<Endpoint>, Vec<Endpoint>)> {
    let device_desc = device.device_descriptor()?;
    let mut readable_endpoints = vec![];
    let mut writable_endpoints = vec![];
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if cfg!(debug_assertions) {
            println!("{:#?}", config_desc);
        }

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                if cfg!(debug_assertions) {
                    println!("{:#?}", interface_desc);
                }

                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    if cfg!(debug_assertions) {
                        println!("{:#?}", endpoint_desc);
                    }

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

pub fn configure_endpoint<T: UsbContext>(
    handle: &mut DeviceHandle<T>,
    endpoint: &Endpoint,
) -> Result<()> {
    handle.set_active_configuration(endpoint.config)?;
    handle.claim_interface(endpoint.iface)?;
    handle.set_alternate_setting(endpoint.iface, endpoint.setting)
}

const PACKET_DATA_LENGTH: usize = 36;

pub fn read_interrupt<T: UsbContext>(handle: &mut DeviceHandle<T>, address: u8) -> Result<Vec<u8>> {
    let timeout = Duration::from_secs(1);
    let mut buf = [0_u8; PACKET_DATA_LENGTH];

    handle
        .read_interrupt(address, &mut buf, timeout)
        .map(|_| buf.to_vec())
}

pub fn write_interrupt<T: UsbContext>(handle: &mut DeviceHandle<T>, address: u8, leds: ThrottleLEDState, brightness: u8) -> Result<usize> {
    let timeout = Duration::from_secs(1);
    let mut buf = [0_u8; PACKET_DATA_LENGTH];

    buf[0] = 1;
    buf[1] = 6;
    buf[2] = leds.into();
    buf[3] = brightness;

    handle.write_interrupt(address, &buf, timeout)
}

pub fn print_data(data: Vec<u8>) {
    let mut rdr = Cursor::new(data);

    rdr.set_position(26);
    let leds = rdr.read_u8().unwrap_or_default();
    println!("LEDs: {}", ThrottleLEDState::from(leds));

    // Clamped to [0,5], where 0 is off and 5 is the brightest
    // Default: 1
    rdr.set_position(27);
    let brightness = rdr.read_u8().unwrap_or_default();
    println!("Brightness: {:?}", brightness);
}
