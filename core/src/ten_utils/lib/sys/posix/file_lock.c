//
// This file is part of the TEN Framework project.
// See https://github.com/TEN-framework/ten_framework/LICENSE for license
// information.
//
#include "ten_utils/lib/file_lock.h"

#include <errno.h>
#include <fcntl.h>

#include "ten_utils/log/log.h"

static int ten_file_lock_reg(int fd, int cmd, short type, off_t offset,
                             short whence, off_t len) {
  struct flock lock;

  lock.l_type = type;
  lock.l_start = offset;
  lock.l_whence = whence;
  lock.l_len = len;

  return fcntl(fd, cmd, &lock);
}

int ten_file_writew_lock(int fd) {
  // Lock all bytes in the file.
  int rc = ten_file_lock_reg(fd, F_SETLKW, F_WRLCK, 0, SEEK_SET, 0);
  if (rc == -1) {
    TEN_LOGE("Failed to lock file: %d", errno);
  }
  return rc;
}

int ten_file_unlock(int fd) {
  int rc = ten_file_lock_reg(fd, F_SETLK, F_UNLCK, 0, SEEK_SET, 0);
  if (rc == -1) {
    TEN_LOGE("Failed to unlock file: %d", errno);
  }
  return rc;
}
