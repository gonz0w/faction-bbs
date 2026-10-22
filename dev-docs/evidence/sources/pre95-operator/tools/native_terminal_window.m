/* macOS fixture focus helper. No screenshots, input injection or settings. */
#import <Cocoa/Cocoa.h>
int main(int argc, char **argv) { @autoreleasepool {
    if (argc == 2 && !strcmp(argv[1], "front")) {
        printf("%d\n", NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier);
        return 0;
    }
    if (argc == 4 && !strcmp(argv[1], "restore")) {
        pid_t prior = atoi(argv[2]), expected = atoi(argv[3]);
        if (NSWorkspace.sharedWorkspace.frontmostApplication.processIdentifier != expected) return 0;
        NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:prior];
        return app && [app activateWithOptions:0] ? 0 : 1;
    }
    return 2;
} }
