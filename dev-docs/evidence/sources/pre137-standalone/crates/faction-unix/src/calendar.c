#define _POSIX_C_SOURCE 200809L
#include <stdint.h>
#include <time.h>

/* Once at startup, before shared credential helpers. Never modifies TZ or the
 * host clock. libc owns its timezone state; callers supply all result storage. */
void faction_calendar_init(void) { tzset(); }

int faction_calendar_date(int64_t seconds, int local, int32_t *year,
    int32_t *month, int32_t *day) {
    time_t stamp;
    struct tm result;
    if (seconds < 0 || seconds > INT64_C(253402300799) ||
        (local != 0 && local != 1) || !year || !month || !day)
        return -1;
    stamp = (time_t)seconds;
    if ((int64_t)stamp != seconds ||
        !(local ? localtime_r(&stamp, &result) : gmtime_r(&stamp, &result)) ||
        result.tm_year < -1899 || result.tm_year > 8099 ||
        result.tm_mon < 0 || result.tm_mon > 11 ||
        result.tm_mday < 1 || result.tm_mday > 31)
        return -1;
    *year = result.tm_year + 1900;
    *month = result.tm_mon + 1;
    *day = result.tm_mday;
    return 0;
}
