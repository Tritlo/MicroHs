/* Minimal support code for pure-WASI Emscripten builds. */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/time.h>
#include <time.h>
#include <unistd.h>

#define FFS __builtin_ffsl

uintptr_t
wasi_gettimemicro(void)
{
  struct timespec ts;

  if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0)
    return 0;
  return (uintptr_t)ts.tv_sec * 1000000 + (uintptr_t)ts.tv_nsec / 1000;
}
#define GETTIMEMICRO wasi_gettimemicro

char*
wasi_tmpname(const char* pre, const char* suf)
{
  static unsigned counter = 0;
  const char *tmpdir = getenv("TMPDIR");
  size_t len;
  char *path;

  if (!tmpdir)
    tmpdir = "/tmp";
  len = strlen(tmpdir) + strlen(pre) + strlen(suf) + 32;
  path = malloc(len);
  if (!path)
    return 0;
  snprintf(path, len, "%s/%s%u%s", tmpdir, pre, counter++, suf);
  return path;
}
#define TMPNAME wasi_tmpname

#define CLOCK_INIT() do { } while(0)
#define CLOCK_T int64_t
#define CLOCK_GET wasi_clock_get
#define CLOCK_SLEEP(usecs) do { (void)(usecs); } while(0)

CLOCK_T
wasi_clock_get(void)
{
  return (CLOCK_T)wasi_gettimemicro();
}

void
wasi_getcputime(long *sec, long *nsec)
{
  struct timespec ts;

  if (clock_gettime(CLOCK_PROCESS_CPUTIME_ID, &ts) == 0) {
    *sec = ts.tv_sec;
    *nsec = ts.tv_nsec;
    return;
  }
  *sec = 0;
  *nsec = 0;
}
#define GETCPUTIME wasi_getcputime
