#include <stdio.h>
#include <stdarg.h>

typedef void (*retro_log_printf_t)(unsigned int level, const char *fmt, ...);

struct retro_log_callback {
    retro_log_printf_t log;
};

static void emusim_libretro_log_fn(unsigned int level, const char *fmt, ...) {
    va_list args;
    va_start(args, fmt);
    fprintf(stderr, "[CORE-LOG %u] ", level);
    vfprintf(stderr, fmt, args);
    va_end(args);
}

void emusim_init_log_callback(struct retro_log_callback *cb) {
    if (cb) {
        cb->log = emusim_libretro_log_fn;
    }
}
