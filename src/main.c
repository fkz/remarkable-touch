#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdbool.h>
#include <string.h>
#include <fcntl.h>
#include <linux/input.h>
#include <time.h>

void writeEvent(int fd, struct input_event event)
{
  struct timeval tv;
  gettimeofday(&tv, NULL);
  event.time = tv;
  write(fd, &event, sizeof(struct input_event));
}

void writeSwipe(int fd, int from[2], int to[2]) {
  struct input_event event;

  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_SLOT, .value = 0x7FFFFFFF}; //Use max signed int slot
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_TRACKING_ID, .value = time(NULL)};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_X, .value = from[0]};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_Y, .value = from[1]};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_SYN, .code = SYN_REPORT, .value = 1};
  writeEvent(fd, event);

  usleep(100000);

  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_X, .value = to[0]};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_Y, .value = to[1]};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_SYN, .code = SYN_REPORT, .value = 1};
  writeEvent(fd, event);

  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_TRACKING_ID, .value = -1};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_SYN, .code = SYN_REPORT, .value = 1};
  writeEvent(fd, event);
}

int main(int argc, char *argv[]) {
  printf("Going to swipe :)\n");

  int fd = open("/dev/input/event2", O_WRONLY);

  int right[2];
  int left[2];

  right[0] = 1200;
  right[1] = 1024;
  left[0] = 200;
  left[1] = 1024;

  writeSwipe(fd, right, left);
}