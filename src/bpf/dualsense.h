#ifndef ____DUALSENSE__H
#define ____DUALSENSE__H

#include "vmlinux.h"

#define DS_INPUT_REPORT_BT 0x31
#define DS_INPUT_REPORT_BT_SIZE 78

#define PS_INPUT_CRC32_SEED 0xA1;

#define PACKED __attribute__((__packed__))

struct PACKED dualsense_touch_point {
	uint8_t contact;
	uint8_t x_lo;
	uint8_t x_hi:4, y_lo:4;
	uint8_t y_hi;
};

/* Main DualSense input report excluding any BT/USB specific headers. */
struct PACKED dualsense_input_report {
	uint8_t x, y;
	uint8_t rx, ry;
	uint8_t z, rz;
	uint8_t seq_number;
	uint8_t buttons[4];
	uint8_t reserved[4];

	/* Motion sensors */
	__le16 gyro[3]; /* x, y, z */
	__le16 accel[3]; /* x, y, z */
	__le32 sensor_timestamp;
	uint8_t reserved2;

	/* Touchpad */
	struct dualsense_touch_point points[2];

	uint8_t reserved3[12];
	uint8_t status;
	uint8_t reserved4[10];
};

#endif /* ____DUALSENSE__H */