/* WASI platform operations. Included by eval.c. */
#include <sys/stat.h>
#include <dirent.h>

#define FFS __builtin_ffs

/* Read one byte from stdin. WASI does not expose terminal raw mode. */
static int
getraw(void)
{
  unsigned char c;
  return read(0, &c, 1) == 1 ? c : -1;
}
#define GETRAW getraw

uintptr_t
gettimemicro(void)
{
  struct timeval tv;
  (void)gettimeofday(&tv, NULL);
  return (uintptr_t)((uint64_t)tv.tv_sec * 1000000 + tv.tv_usec);
}
#define GETTIMEMICRO gettimemicro

#define CLOCK_INIT() do { } while (0)
#define CLOCK_T int64_t
#define CLOCK_GET clock_get
#define CLOCK_SLEEP usleep
CLOCK_T
clock_get(void)
{
  struct timeval tv;
  (void)gettimeofday(&tv, NULL);
  return (int64_t)tv.tv_sec * 1000000 + tv.tv_usec;
}

/* Preview 1 has no process CPU clock. */
void
getcputime(long *sec, long *nsec)
{
  *sec = 0;
  *nsec = 0;
}
#define GETCPUTIME getcputime

/* These operations need interfaces that Preview 1 does not provide. */
char *
tmpname(const char *pre, const char *suf)
{
  (void)pre;
  (void)suf;
  errno = ENOSYS;
  return NULL;
}
#define TMPNAME tmpname

int
get_permissions(const char *path)
{
  struct stat st;
  if (stat(path, &st) == -1)
    return -1;
  return ((st.st_mode & S_IRUSR) ? 4 : 0) |
         ((st.st_mode & S_IWUSR) ? 2 : 0) |
         ((st.st_mode & S_IXUSR) ? (S_ISDIR(st.st_mode) ? 8 : 1) : 0);
}

int
set_permissions(const char *path, int perms)
{
  (void)path;
  (void)perms;
  errno = ENOSYS;
  return -1;
}
int
system(const char *command)
{
  if (!command)
    return 0;
  errno = ENOSYS;
  return -1;
}
