use std::{time::Duration, thread, sync::Mutex};
use lazy_static::lazy_static;
use rusb::{Context, UsbContext};

//设备的pid和vid
pub static VID: [u16;1] = [0x1209];
pub static PID: [u16;1] = [0x7301];

lazy_static!{
    //要发送的数据缓冲
    pub static ref SEND_BUFF: Mutex<Vec<Vec<u8>>> = Mutex::new(vec![]);
    pub static ref USB_CONNECTED: Mutex<bool> = Mutex::new(false);
}

fn check_vid_pid(v : u16, p: u16) -> bool
{
    for i in 0..VID.len() {
        for j in 0..PID.len() {
            if VID[i] == v && PID[j] == p {
                return true;
            }
        }
    }
    return false;
}

//连接usb设备的线程
pub fn connect_usb() {
    thread::spawn(|| {
        'main: loop {
            {//没连上
                *USB_CONNECTED.lock().unwrap() = false;
            }
            let context = match Context::new() {
                Ok(r) => r,
                Err(e) => {
                    println!("could not initialize libusb: {}", e);
                    continue 'main;
                }
            };
            let devices = match context.devices() {
                Ok(d) => d,
                Err(_) => continue 'main,
            };
            for device in devices.iter() {
                let device_desc = match device.device_descriptor() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                //需要对上pid vid
                if !check_vid_pid(device_desc.vendor_id(),device_desc.product_id()) {
                    continue;
                }
                //获取曲柄
                let mut handle = match device.open() {
                    Ok(d) => d,
                    Err(_) => continue,
                };
                //重启usb设备
                if handle.reset().is_err() {
                    continue;
                }
                //打开usb
                if handle.set_active_configuration(1).is_err() {
                    continue;
                }
                if handle.claim_interface(0).is_err() {
                    continue;
                }
                if handle.set_alternate_setting(0, 0).is_err() {
                    continue;
                }

                thread::sleep(Duration::from_secs(2));
                {//连上了，切换状态
                    *USB_CONNECTED.lock().unwrap() = true;
                }
                //循环检查要不要发消息
                loop {
                    //要发数据
                    let mut data = SEND_BUFF.lock().unwrap();
                    for i in 0..data.len() {
                        if data.len() == 0 {
                            println!("data.len() == 0 ??");
                            continue;
                        }
                        let r = handle.write_bulk(2, data[i].as_slice(), Duration::from_millis(500)).is_ok();
                        if !r {//发失败了，说明USB关了
                            continue 'main;
                        }
                    }
                    data.clear();
                }
            }
            thread::sleep(Duration::from_secs(1));//延时，防止卡死
        }
    });
}

pub fn send_monitor_data(cpu: u8, ram: u8, gpu: u8, year: u16, mon: u8, day: u8, hour: u8, min: u8) {
    {//没连上就别发了
        let is_connected = *USB_CONNECTED.lock().unwrap();
        if !is_connected {
            return;
        }
    }
    let mut t = vec![0x55,0xaa,0x40,0xf0];
    t.push(cpu);
    t.push(ram);
    t.push(gpu);
    t.push((year % 0x100).try_into().unwrap());
    t.push((year / 0x100).try_into().unwrap());
    t.push(mon);
    t.push(day);
    t.push(hour);
    t.push(min);
    for _ in 0..49 { t.push(0x00); }
    t.push(0x0a);t.push(0x0d);
    let mut buff = SEND_BUFF.lock().unwrap();
    buff.push(t);
}


