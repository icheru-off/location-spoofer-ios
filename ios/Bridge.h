// C interface of the Rust core (core/src/lib.rs). All calls block.
#ifndef LOCATION_SPOOFER_BRIDGE_H
#define LOCATION_SPOOFER_BRIDGE_H

#include <stddef.h>
#include <stdint.h>

// Result codes
#define GP_OK 0
#define GP_ERR_ARGS 1
#define GP_ERR_PAIRING_FILE 2
#define GP_ERR_UNREACHABLE 3
#define GP_ERR_PAIRING_REJECTED 4
#define GP_ERR_TUNNEL 5
#define GP_ERR_DVT 6
#define GP_ERR_NOT_CONNECTED 7
#define GP_ERR_COMMAND 8
#define GP_ERR_TIMEOUT 9

// Connection states
#define GP_STATE_IDLE 0
#define GP_STATE_CONNECTED 1
#define GP_STATE_LOST 2

int32_t gp_connect(const char *pairing_path, const char *ip, uint16_t port);
int32_t gp_set_location(double latitude, double longitude);
int32_t gp_clear_location(void);
void gp_disconnect(void);
int32_t gp_state(void);
void gp_last_error(char *buf, size_t len);
int32_t gp_pairing_file_valid(const char *path);

#endif
