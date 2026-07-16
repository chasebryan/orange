#define _GNU_SOURCE

#include <errno.h>
#include <fcntl.h>
#include <linux/landlock.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/syscall.h>
#include <unistd.h>

#ifndef LANDLOCK_ACCESS_FS_IOCTL_DEV
#define LANDLOCK_ACCESS_FS_IOCTL_DEV (1ULL << 15)
#endif

#define ORANGE_MAXIMUM_CPU_SECONDS 600U
#define ORANGE_MAXIMUM_FILE_BYTES (512U * 1024U * 1024U)
#define ORANGE_MAXIMUM_OPEN_FILES 1024U
#define ORANGE_MAXIMUM_PROCESSES 256U

static void fail(const char *stage)
{
    const int error_number = errno;

    (void)dprintf(STDERR_FILENO,
                  "orange filesystem sandbox failed at %s (errno %d)\n",
                  stage, error_number);
    _exit(125);
}

static uint64_t handled_rights(int abi)
{
    uint64_t rights = LANDLOCK_ACCESS_FS_EXECUTE |
                      LANDLOCK_ACCESS_FS_WRITE_FILE |
                      LANDLOCK_ACCESS_FS_READ_FILE |
                      LANDLOCK_ACCESS_FS_READ_DIR |
                      LANDLOCK_ACCESS_FS_REMOVE_DIR |
                      LANDLOCK_ACCESS_FS_REMOVE_FILE |
                      LANDLOCK_ACCESS_FS_MAKE_CHAR |
                      LANDLOCK_ACCESS_FS_MAKE_DIR |
                      LANDLOCK_ACCESS_FS_MAKE_REG |
                      LANDLOCK_ACCESS_FS_MAKE_SOCK |
                      LANDLOCK_ACCESS_FS_MAKE_FIFO |
                      LANDLOCK_ACCESS_FS_MAKE_BLOCK |
                      LANDLOCK_ACCESS_FS_MAKE_SYM;

    if (abi >= 2) {
        rights |= LANDLOCK_ACCESS_FS_REFER;
    }
    if (abi >= 3) {
        rights |= LANDLOCK_ACCESS_FS_TRUNCATE;
    }
    if (abi >= 5) {
        rights |= LANDLOCK_ACCESS_FS_IOCTL_DEV;
    }
    return rights;
}

static void cap_resource(int resource, rlim_t maximum, const char *stage)
{
    struct rlimit limit;

    if (getrlimit(resource, &limit) < 0) {
        fail(stage);
    }
    if (limit.rlim_max == RLIM_INFINITY || limit.rlim_max > maximum) {
        limit.rlim_cur = maximum;
        limit.rlim_max = maximum;
    } else {
        limit.rlim_cur = limit.rlim_max;
    }
    if (setrlimit(resource, &limit) < 0) {
        fail(stage);
    }
}

static void cap_resources(void)
{
    cap_resource(RLIMIT_CORE, 0, "limit-core");
    cap_resource(RLIMIT_CPU, ORANGE_MAXIMUM_CPU_SECONDS, "limit-cpu");
    cap_resource(RLIMIT_FSIZE, ORANGE_MAXIMUM_FILE_BYTES, "limit-file-size");
    cap_resource(RLIMIT_NOFILE, ORANGE_MAXIMUM_OPEN_FILES, "limit-open-files");
    cap_resource(RLIMIT_NPROC, ORANGE_MAXIMUM_PROCESSES, "limit-processes");
}

static uint64_t allowed_rights(uint64_t handled, mode_t mode, int access_mode)
{
    uint64_t rights = 0;

    if (S_ISDIR(mode)) {
        rights |= LANDLOCK_ACCESS_FS_READ_DIR;
    }
    if (access_mode == 0) {
        return rights & handled;
    }
    rights |= LANDLOCK_ACCESS_FS_EXECUTE | LANDLOCK_ACCESS_FS_READ_FILE;
    if (access_mode == 1) {
        return rights & handled;
    }

    rights |= LANDLOCK_ACCESS_FS_WRITE_FILE | LANDLOCK_ACCESS_FS_TRUNCATE;
    if (S_ISDIR(mode)) {
        rights |= LANDLOCK_ACCESS_FS_REMOVE_DIR |
                  LANDLOCK_ACCESS_FS_REMOVE_FILE |
                  LANDLOCK_ACCESS_FS_MAKE_DIR |
                  LANDLOCK_ACCESS_FS_MAKE_REG |
                  LANDLOCK_ACCESS_FS_MAKE_SOCK |
                  LANDLOCK_ACCESS_FS_MAKE_FIFO |
                  LANDLOCK_ACCESS_FS_MAKE_SYM |
                  LANDLOCK_ACCESS_FS_REFER;
    }
    if (S_ISCHR(mode) || S_ISBLK(mode)) {
        rights |= LANDLOCK_ACCESS_FS_IOCTL_DEV;
    }
    return rights & handled;
}

static void add_path_rule(int ruleset_fd, uint64_t handled,
                          const char *path, int access_mode)
{
    struct landlock_path_beneath_attr rule = {0};
    struct stat status;
    int path_fd;

    if (path[0] != '/') {
        errno = EINVAL;
        fail("path");
    }
    path_fd = open(path, O_PATH | O_CLOEXEC);
    if (path_fd < 0) {
        fail("open-rule");
    }
    if (fstat(path_fd, &status) < 0) {
        fail("stat-rule");
    }
    rule.parent_fd = path_fd;
    rule.allowed_access = allowed_rights(handled, status.st_mode, access_mode);
    if (syscall(SYS_landlock_add_rule, ruleset_fd,
                LANDLOCK_RULE_PATH_BENEATH, &rule, 0) < 0) {
        fail("add-rule");
    }
    if (close(path_fd) < 0) {
        fail("close-rule");
    }
}

int main(int argc, char **argv)
{
    struct landlock_ruleset_attr ruleset = {0};
    uint64_t handled;
    int abi;
    int ruleset_fd;
    int index = 1;
    int rule_count = 0;

    abi = (int)syscall(SYS_landlock_create_ruleset, NULL, 0,
                       LANDLOCK_CREATE_RULESET_VERSION);
    if (abi < 3) {
        errno = abi < 0 ? errno : EOPNOTSUPP;
        fail("abi");
    }
    handled = handled_rights(abi);
    ruleset.handled_access_fs = handled;
    ruleset_fd = (int)syscall(SYS_landlock_create_ruleset, &ruleset,
                              sizeof(ruleset), 0);
    if (ruleset_fd < 0) {
        fail("create-ruleset");
    }

    while (index < argc && argv[index][0] == '-') {
        int access_mode;

        if (strcmp(argv[index], "--") == 0) {
            ++index;
            break;
        }
        if (index + 1 >= argc) {
            errno = EINVAL;
            fail("arguments");
        }
        if (strcmp(argv[index], "--dir") == 0) {
            access_mode = 0;
        } else if (strcmp(argv[index], "--ro") == 0) {
            access_mode = 1;
        } else if (strcmp(argv[index], "--rw") == 0) {
            access_mode = 2;
        } else {
            errno = EINVAL;
            fail("arguments");
        }
        add_path_rule(ruleset_fd, handled, argv[index + 1], access_mode);
        index += 2;
        ++rule_count;
    }
    if (rule_count == 0 || index >= argc || argv[index][0] != '/') {
        errno = EINVAL;
        fail("arguments");
    }
    if (prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) < 0) {
        fail("no-new-privileges");
    }
    if (syscall(SYS_landlock_restrict_self, ruleset_fd, 0) < 0) {
        fail("restrict-self");
    }
    if (close(ruleset_fd) < 0) {
        fail("close-ruleset");
    }
    cap_resources();
    if (syscall(SYS_close_range, 3U, ~0U, 0U) < 0) {
        fail("close-descriptors");
    }
    execv(argv[index], &argv[index]);
    fail("execute");
}
