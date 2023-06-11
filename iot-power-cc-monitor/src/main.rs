use std::{thread, time::Duration};

use nvml_wrapper::{Nvml, Device, error::NvmlError};
use sysinfo::{System, SystemExt, CpuExt};

//获取刷新时间，不能小于最小刷新间隔
fn get_wait_duration(wait_ms: u64) -> Duration {
    if System::MINIMUM_CPU_UPDATE_INTERVAL > Duration::from_millis(wait_ms){
        System::MINIMUM_CPU_UPDATE_INTERVAL
    }else{
        Duration::from_millis(wait_ms)
    }
}
//获取cpu使用率百分比
fn get_cpu_usage(sys : &mut System) -> i8 {
    sys.refresh_cpu();
    let mut cpu_usage = 0.0;
    for cpu in sys.cpus() {
        cpu_usage += cpu.cpu_usage();
    }
    (cpu_usage / sys.cpus().len() as f32) as i8
}
//获取内存使用率百分比
fn get_mem_usage(sys : &mut System) -> i8 {
    sys.refresh_memory();
    (sys.used_memory() * 100 / sys.total_memory()) as i8
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
fn get_nv_usage(device: &Device) -> i8 {
    match device.utilization_rates() {
        Ok(usage) => usage.gpu as i8,
        Err(_) => 0,
    }
}

fn main() {
    let nvml = Nvml::init();
    let gpu = get_nv_device(&nvml);


    //刷新时间
    let refresh_wait_time = get_wait_duration(500);
    let mut sys = System::new_all();

    for _ in 0..100 {
        println!("Used RAM: {}%", get_mem_usage(&mut sys));
        println!("Used CPU: {}%", get_cpu_usage(&mut sys));

        if !gpu.is_none() {
            let Some(gpu) = &gpu else { panic!() };
            println!("Used GPU: {}%", get_nv_usage(gpu));
        }

        thread::sleep(refresh_wait_time);
    }

}
