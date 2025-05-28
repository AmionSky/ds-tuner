#ifndef ____DUALSENSE__H
#define ____DUALSENSE__H

#include "vmlinux.h"

#define DS_INPUT_REPORT_USB 0x01
#define DS_INPUT_REPORT_USB_SIZE 64

#define DS_INPUT_REPORT_BT 0x31
#define DS_INPUT_REPORT_BT_SIZE 78

#define PS_INPUT_CRC32_SEED 0xA1;

/* Button masks for DualSense input report. */
#define DS_BUTTONS0_SQUARE		(1 << 4)
#define DS_BUTTONS0_CROSS		(1 << 5)
#define DS_BUTTONS0_CIRCLE		(1 << 6)
#define DS_BUTTONS0_TRIANGLE	(1 << 7)
#define DS_BUTTONS1_L1			(1 << 0)
#define DS_BUTTONS1_R1			(1 << 1)
#define DS_BUTTONS1_L2			(1 << 2)
#define DS_BUTTONS1_R2			(1 << 3)
#define DS_BUTTONS1_CREATE		(1 << 4)
#define DS_BUTTONS1_OPTIONS		(1 << 5)
#define DS_BUTTONS1_L3			(1 << 6)
#define DS_BUTTONS1_R3			(1 << 7)
#define DS_BUTTONS2_PS_HOME		(1 << 0)
#define DS_BUTTONS2_TOUCHPAD	(1 << 1)
#define DS_BUTTONS2_MIC_MUTE	(1 << 2)

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

// CRC function declarations
bool check_crc(const u8 *data, size_t len);
void update_crc(u8 *data, size_t len);

#endif /* ____DUALSENSE__H */