#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
use std::{thread, time::Duration};
use chrono::{Datelike, Timelike};
use nvml_wrapper::{Nvml, Device, error::NvmlError};
use sysinfo::{System, SystemExt, CpuExt};
use tray_icon::{icon::Icon, menu::Menu, TrayIcon};
mod usb;

//创建托盘图标
fn create_tray() -> TrayIcon {
    let d = include_bytes!("icon.ico");
    std::fs::write("./icon.ico", d).unwrap();
    let icon = Icon::from_path("icon.ico", None).unwrap();

    tray_icon::TrayIconBuilder::new()
        .with_menu(Box::new(Menu::new()))
        .with_tooltip("IoT Power Tool")
        .with_icon(icon)
        .build()
        .unwrap()
}

//获取刷新时间，不能小于最小刷新间隔
fn get_wait_duration(wait_ms: u64) -> Duration {
    if System::MINIMUM_CPU_UPDATE_INTERVAL > Duration::from_millis(wait_ms){
        System::MINIMUM_CPU_UPDATE_INTERVAL
    }else{
        Duration::from_millis(wait_ms)
    }
}
//获取cpu使用率百分比
fn get_cpu_usage(sys : &mut System) -> u8 {
    sys.refresh_cpu();
    let mut cpu_usage = 0.0;
    for cpu in sys.cpus() {
        cpu_usage += cpu.cpu_usage();
    }
    (cpu_usage / sys.cpus().len() as f32) as u8
}
//获取内存使用率百分比
fn get_mem_usage(sys : &mut System) -> u8 {
    sys.refresh_memory();
    (sys.used_memory() * 100 / sys.total_memory()) as u8
}

//获取显卡
fn get_nv_device(init: &Result<Nvml, NvmlError>) -> Option<Device> {
    let nvml = match init {
        Ok(nvml) => nvml,
        Err(_) => return None,
    };
    // Get the first `Device` (GPU) in the system
    match nvml.device_by_index(0) {
        Ok(device) => Some(device),
        Err(_) => None,
    }
}
//获取显卡使用率百分比
fn get_nv_usage(device: &Device) -> u8 {
    match device.utilization_rates() {
        Ok(usage) => usage.gpu as u8,
        Err(_) => 0,
    }
}

fn main() {
    let mut tray_icon = create_tray();

    //usb线程
    usb::connect_usb();

    let nvml = Nvml::init();
    let gpu = get_nv_device(&nvml);

    //刷新时间
    let refresh_wait_time = get_wait_duration(500);
    let mut sys = System::new_all();

    loop {
        let cpu_usage = get_cpu_usage(&mut sys);
        let mem_usage = get_mem_usage(&mut sys);
        let mut gpu_usage = 0;

        println!("Used RAM: {}%", mem_usage);
        println!("Used CPU: {}%", cpu_usage);

        if !gpu.is_none() {
            let Some(gpu) = &gpu else { panic!() };
            gpu_usage = get_nv_usage(gpu);
            println!("Used GPU: {}%", gpu_usage);
        }

        tray_icon.set_tooltip(Some(
            format!("IoT Power Tool\nUsed CPU: {}%\nUsed RAM: {}%\nUsed GPU: {}%", 
            cpu_usage, 
            mem_usage, 
            gpu_usage)
        )).unwrap();

        let time_now = chrono::Local::now();
        usb::send_monitor_data(cpu_usage, mem_usage, gpu_usage, 
            time_now.year() as u16, 
            time_now.month() as u8, 
            time_now.day() as u8, 
            time_now.hour() as u8, 
            time_now.minute() as u8);
        thread::sleep(refresh_wait_time);
    }

}
