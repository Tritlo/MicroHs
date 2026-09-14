/* WASI Preview 1 configuration. */
#define WANT_STDIO 1
#define WANT_FD 1
#define WANT_FLOAT32 1
#define WANT_FLOAT64 1
#define WANT_MATH 1
#define WANT_INT64 1
#define WANT_MD5 1
#define WANT_TICK 1
#define WANT_DIR 1
#define WANT_TIME 1
#define WANT_SIGINT 0
#define WANT_TAGNAMES 1
#define WANT_ERRNO 1
#define WANT_OVERFLOW 1
#define WANT_IO_POLL 0
#define WANT_SOCKET 0
#define WANT_GMP 0
#define WANT_IMATH 1
#define WANT_KPERF 0

#define GCRED 1
#define INTTABLE 1
#define SANITY 1
#define STACKOVL 1

#include <inttypes.h>
#include <sys/types.h>
#include <unistd.h>
#include <string.h>
#include <strings.h>
#include <stdlib.h>
#include <fcntl.h>
#include <errno.h>
#include <time.h>
#include <sys/time.h>
#include <stdio.h>
#include <locale.h>
#include <limits.h>
#include <stdbool.h>
