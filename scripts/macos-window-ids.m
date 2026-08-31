#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>

int main(int argc, const char *argv[]) {
  @autoreleasepool {
    NSString *owner = argc > 1 ? [NSString stringWithUTF8String:argv[1]] : nil;
    CFArrayRef windows = CGWindowListCopyWindowInfo(
        kCGWindowListOptionAll | kCGWindowListExcludeDesktopElements, kCGNullWindowID);
    for (NSDictionary *window in (__bridge NSArray *)windows) {
      if (owner && ![window[(id)kCGWindowOwnerName] isEqualToString:owner]) continue;
      NSNumber *number = window[(id)kCGWindowNumber];
      NSString *owner_name = window[(id)kCGWindowOwnerName] ?: @"";
      NSString *title = window[(id)kCGWindowName] ?: @"";
      NSDictionary *bounds = window[(id)kCGWindowBounds];
      printf("%s\t%s\t%s\t%s\n", number.stringValue.UTF8String,
             owner_name.UTF8String, title.UTF8String, bounds.description.UTF8String);
    }
    if (windows) CFRelease(windows);
  }
  return 0;
}
