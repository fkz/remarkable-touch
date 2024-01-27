#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <stdbool.h>
#include <string.h>
#include <fcntl.h>
#include <linux/input.h>
#include <time.h>
#include <sys/time.h>

void writeEvent(int fd, struct input_event event)
{
  struct timeval tv;
  gettimeofday(&tv, NULL);
  event.time = tv;
  write(fd, &event, sizeof(struct input_event));
}

void writeSwipe(int fd, int from[2], int to[2]) {
  struct input_event event;

  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_SLOT, .value = 1096}; //Use max signed int slot
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_TRACKING_ID, .value = time(NULL)};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_X, .value = from[0]};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_ABS, .code = ABS_MT_POSITION_Y, .value = from[1]};
  writeEvent(fd, event);
  event = (struct input_event) {.type = EV_SYN, .code = SYN_REPORT, .value = 1};
  writeEvent(fd, event);

  usleep(50000);

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

bool moreThanOneTouch(long touchBitmap) {
  return touchBitmap & (touchBitmap - 1) != 0;
}

int main() {
  int fd = open("/dev/input/event2", O_RDWR);

  struct input_event event;

  long filledSlots = 0;
  int currentSlot = 0;  
  struct timeval initialTouch;

  int x = -1;
  int y = -1;
  int diff = 0;

  struct timeval timeDifference;

  int right[2];
  int left[2];

  right[0] = 1200;
  right[1] = 1024;
  left[0] = 200;
  left[1] = 1024;


  while (true) {
    read(fd, &event, sizeof(event));
    if (event.type != EV_ABS) continue;
    
    if (event.code == ABS_MT_SLOT) {
      if (event.value > 31) {
        printf("slot number %d should never happen\n", event.value);
      }
      currentSlot = event.value;
      filledSlots |= (1l << currentSlot);
    }

    if (event.code == ABS_MT_POSITION_X) {
      if (x == -1) x = event.value;
      int d = abs(event.value - x);
      if (d > diff) diff = d;
    }
    if (event.code == ABS_MT_POSITION_Y) {
      if (y == -1) y = event.value;
      int d = abs(event.value - y);
      if (d > diff) diff = d;
    }

    if (event.code == ABS_MT_TRACKING_ID) {
      if (event.value == -1) {
        filledSlots &= 0xFFFFFFFF ^ (1l << currentSlot);
        if (filledSlots == 0) {
          timersub(&event.time, &initialTouch, &timeDifference);
          if (timeDifference.tv_sec == 0 && timeDifference.tv_usec < 200000) {
            int usec = timeDifference.tv_usec;
            printf("Quick touch at (%d, %d) [diff: %d] [%d usec]\n", x, y, diff, usec);

            if (diff < 8) {
              if (x > 1000 && y < 400 && y >= 0) {
                usleep(50000);
                writeSwipe(fd, right, left);
              } else if (x < 400 && x >= 0 && y < 400 && y >= 0) {
                usleep(50000);
                writeSwipe(fd, left, right);
              }
            }
          } else {
            int sec = timeDifference.tv_sec;
            printf("Long touch at (%d, %d) [diff: %d] [%d sec]\n", x, y, diff, sec);
          }
          x = -1; y = -1; diff = 0;
        }
      } else {
        if (!moreThanOneTouch(filledSlots)) {
          initialTouch = event.time;
        }
      }
    }
  }
}

int main2(int argc, char *argv[]) {
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