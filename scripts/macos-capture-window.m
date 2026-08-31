#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

#include <dlfcn.h>
#include <mach/mach_time.h>

typedef CGImageRef (*CaptureWindow)(CGRect, CGWindowListOption, CGWindowID,
                                    CGWindowImageOption);

int main(int argc, const char *argv[]) {
  @autoreleasepool {
    if (argc != 6) {
      fprintf(stderr, "usage: %s <window-id> <output-dir> <seconds> <fps> <scale>\n", argv[0]);
      return 2;
    }

    CGWindowID window_id = (CGWindowID)strtoul(argv[1], NULL, 10);
    NSString *output_dir = [NSString stringWithUTF8String:argv[2]];
    double seconds = strtod(argv[3], NULL);
    int fps = atoi(argv[4]);
    double scale = strtod(argv[5], NULL);
    if (!window_id || seconds <= 0 || fps <= 0 || scale <= 0) return 2;

    NSError *error = nil;
    if (![[NSFileManager defaultManager] createDirectoryAtPath:output_dir
                                   withIntermediateDirectories:YES
                                                    attributes:nil
                                                         error:&error]) {
      fprintf(stderr, "%s\n", error.localizedDescription.UTF8String);
      return 1;
    }

    CaptureWindow capture = (CaptureWindow)dlsym(RTLD_DEFAULT, "CGWindowListCreateImage");
    if (!capture) return 1;
    mach_timebase_info_data_t timebase;
    mach_timebase_info(&timebase);
    uint64_t deadline = mach_absolute_time();
    uint64_t interval = lround((1000000000.0 / fps) * timebase.denom / timebase.numer);
    int frame_count = lround(seconds * fps);

    for (int frame = 0; frame < frame_count; frame++) {
      @autoreleasepool {
        CGImageRef image = capture(
            CGRectNull, kCGWindowListOptionIncludingWindow, window_id,
            kCGWindowImageBoundsIgnoreFraming | kCGWindowImageNominalResolution);
        if (!image) {
          fprintf(stderr, "unable to capture window %u\n", window_id);
          return 1;
        }

        size_t width = MAX(1, lround(CGImageGetWidth(image) * scale));
        size_t height = MAX(1, lround(CGImageGetHeight(image) * scale));
        CGColorSpaceRef color_space = CGColorSpaceCreateDeviceRGB();
        CGContextRef context = CGBitmapContextCreate(NULL, width, height, 8, width * 4,
                                                     color_space,
                                                     (CGBitmapInfo)kCGImageAlphaPremultipliedLast);
        CGContextSetInterpolationQuality(context, kCGInterpolationHigh);
        CGContextDrawImage(context, CGRectMake(0, 0, width, height), image);
        CGImageRef scaled = CGBitmapContextCreateImage(context);

        NSString *name = [NSString stringWithFormat:@"frame-%05d.png", frame];
        NSURL *url = [NSURL fileURLWithPath:[output_dir stringByAppendingPathComponent:name]];
        CGImageDestinationRef destination = CGImageDestinationCreateWithURL(
            (__bridge CFURLRef)url, (__bridge CFStringRef)UTTypePNG.identifier, 1, NULL);
        CGImageDestinationAddImage(destination, scaled, NULL);
        BOOL saved = CGImageDestinationFinalize(destination);

        CFRelease(destination);
        CGImageRelease(scaled);
        CGContextRelease(context);
        CGColorSpaceRelease(color_space);
        CGImageRelease(image);
        if (!saved) return 1;
      }

      deadline += interval;
      mach_wait_until(deadline);
    }
  }
  return 0;
}
