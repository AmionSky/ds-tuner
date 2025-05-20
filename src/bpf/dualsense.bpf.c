#include "vmlinux.h"
#include "hid_bpf.h"
#include "hid_bpf_helpers.h"
#include <bpf/bpf_tracing.h>

#define VID_SONY 0x054C
#define PID_DUALSENSE 0x0CE6

#define DS_OUTPUT_REPORT_BT 0x31
#define DS_OUTPUT_REPORT_BT_SIZE 78

#define PS_OUTPUT_CRC32_SEED 0xA2

HID_BPF_CONFIG(
    HID_DEVICE(BUS_BLUETOOTH, HID_GROUP_GENERIC, VID_SONY, PID_DUALSENSE),
);

SEC(HID_BPF_DEVICE_EVENT)
int BPF_PROG(edit_values_event, struct hid_bpf_ctx *hid_ctx)
{
    __u8 *data = hid_bpf_get_data(hid_ctx, 0, DS_OUTPUT_REPORT_BT_SIZE);

    if (!data || data[0] != DS_OUTPUT_REPORT_BT)
        return 0; // EPERM or the wrong report ID

    bpf_printk("%s: its running again", __func__);

    for (size_t i = 0; i < 78; i++)
    {
        bpf_printk("%d: %02X", i+1, data[i]);
    }
    
    


    return 0;
}


HID_BPF_OPS(edit_values) = {
    .hid_device_event = (void *)edit_values_event,
};

SEC("syscall")
int probe(struct hid_bpf_probe_args *ctx)
{
    if (ctx->rdesc_size > 4 &&
        ctx->rdesc[0] == 0x05 && // Usage Page
        ctx->rdesc[1] == 0x01 && // Generic Desktop
        ctx->rdesc[2] == 0x09 && // Usage
        ctx->rdesc[3] == 0x05)   // Game Pad
        ctx->retval = 0;
    else
        ctx->retval = -EINVAL;

    return 0;
}

char _license[] SEC("license") = "GPL";

/*
HID_BPF_CONFIG(
    HID_DEVICE(BUS_BLUETOOTH, HID_GROUP_GENERIC, VID_SONY, PID_DUALSENSE),
);

#define DATA_REPORT_ID 49

SEC(HID_BPF_RDESC_FIXUP)
int BPF_PROG(ignore_button_fix_rdesc, struct hid_bpf_ctx *hctx)
{
    // bpf_printk works like printf but shows up in
    // sudo cat /sys/kernel/debug/tracing/trace_pipe
    // bpf_printk("%s: fixing an rdesc", __func__);

    return 0;
}

SEC(HID_BPF_DEVICE_EVENT)
int BPF_PROG(ignore_button_fix_event, struct hid_bpf_ctx *hid_ctx)
{
    __u8 *data = hid_bpf_get_data(hid_ctx, 0, 78);
    
    if (!data || data[0] != DATA_REPORT_ID)
        return 0; // EPERM or the wrong report ID

    __u8 report[6] = {DATA_REPORT_ID, 0x0, 0x0, 0x0, 127, 127};
    //__builtin_memcpy(data, report, sizeof(report));

    data[6] = 12;
    bpf_printk("%s: its running", __func__);

    return 0;
}

HID_BPF_OPS(ignore_button) = {
    .hid_device_event = (void *)ignore_button_fix_event,
    .hid_rdesc_fixup = (void *)ignore_button_fix_rdesc,
};

// If your device only has a single HID interface you can skip
// the probe function altogether
SEC("syscall")
int probe(struct hid_bpf_probe_args *ctx)
{
    if (ctx->rdesc_size > 4 &&
        ctx->rdesc[0] == 0x05 && // Usage Page
        ctx->rdesc[1] == 0x01 && // Generic Desktop
        ctx->rdesc[2] == 0x09 && // Usage
        ctx->rdesc[3] == 0x05)   // Game Pad
        ctx->retval = 0;
    else
        ctx->retval = -EINVAL;

    return 0;
}

char _license[] SEC("license") = "GPL";

*/