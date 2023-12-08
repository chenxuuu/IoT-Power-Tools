import datetime
import threading
import time
from threading import Thread
import usb.core
import usb.util
from psutil import *
import pynvml

lock = threading.Lock()
SEND_BUFF = bytearray()


def connect_usb():
    global SEND_BUFF
    while True:
        time.sleep(1)
        dev = usb.core.find(idVendor=0x1209, idProduct=0x7301)
        if dev is None:
            print("Device not found")
            continue
        print("DEV======================")
        print(dev)
        dev.set_configuration()
        cfg = dev.get_active_configuration()
        print("CFG======================")
        print(cfg)
        intf = cfg[(0, 0)]
        print("INTERFACE======================")
        print(intf)
        ep = usb.util.find_descriptor(
            intf,
            custom_match=lambda e: usb.util.endpoint_direction(e.bEndpointAddress) == usb.util.ENDPOINT_OUT
        )
        print("ENDPOINT======================")
        print(ep)
        while True:
            try:
                lock.acquire()
                ret = dev.write(ep, SEND_BUFF, 1000)
                lock.release()
            except Exception as e:
                print(e)
            time.sleep(0.05)


def send_monitor_data():
    global SEND_BUFF
    pynvml.nvmlInit()
    while True:
        try:
            time.sleep(0.1)
            arr = bytearray()
            # header
            arr.append(0x55)
            arr.append(0xAA)
            arr.append(0x40)
            arr.append(0xF0)

            # CPU
            cpu = int(cpu_percent(interval=1))
            arr.append(cpu)

            # Memory
            mem = int(virtual_memory()[2])
            arr.append(mem)

            # GPU
            try:
                handle = pynvml.nvmlDeviceGetHandleByIndex(0)
                gpu_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
                gpu_total = gpu_info.total
                gpu_used = gpu_info.used
                gpu_percent = int(gpu_used / gpu_total * 100)
                arr.append(gpu_percent)
            except:
                arr.append(0x00)

            # Year
            today = datetime.datetime.today()
            year = today.year
            arr.append(year % 0x100)
            arr.append(int(year / 0x100))

            # Month
            month = today.month
            arr.append(month)

            # Day
            day = today.day
            arr.append(day)

            # Hour
            hour = today.hour
            arr.append(hour)

            # Minute
            minute = today.minute
            arr.append(minute)
            for i in range(0, 49):
                arr.append(0x00)
            arr.append(0x0a)
            arr.append(0x0d)

            lock.acquire()
            SEND_BUFF = arr
            lock.release()

        except Exception as e:
            print(e)


def main():
    t1 = Thread(target=send_monitor_data)
    t1.start()

    t2 = Thread(target=connect_usb)
    t2.start()

    # cpu: u8,
    # ram: u8,
    # gpu: u8,
    # year: u16,
    # mon: u8,
    # day: u8,
    # hour: u8,
    # min: u8,

    while True:
        time.sleep(1)


if __name__ == '__main__':
    main()
