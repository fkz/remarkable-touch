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

void writeSwipe(int fd, int from[2], int to[2], int stepCount, int microsecondsBetweenSteps) {
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

  for (int i = 1; i <= stepCount; ++i) {
    usleep(microsecondsBetweenSteps);
    int x = from[0] + (to[0] - from[0]) * i / stepCount;
    int y = from[1] + (to[1] - from[1]) * i / stepCount;

    event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_X, .value = x};
    writeEvent(fd, event);
    event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_Y, .value = y};
    writeEvent(fd, event);
    event = (struct input_event) {.type = EV_SYN, .code = SYN_REPORT, .value = 1};
    writeEvent(fd, event);
  }

  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_TRACKING_ID, .value = -1};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_SYN, .code = SYN_REPORT, .value = 1};
  writeEvent(fd, event);
}

int main(int argc, char *argv[]) {
  printf("Going to swipe :)\n");

  int fd = open("/dev/input/event2", O_WRONLY);

  int from[2];
  int to[2];

  from[0] = 1200;
  from[1] = 1024;
  to[0] = 200;
  to[1] = 1024;

  int stepCount;
  int microSecondsBetweenSteps;

  if (argc != 3) {
    printf("Wrong number of arguments\n");
    return 1;
  }

  stepCount = atoi(argv[1]);
  microSecondsBetweenSteps = atoi(argv[2]);

  writeSwipe(fd, from, to, stepCount, microSecondsBetweenSteps);
}