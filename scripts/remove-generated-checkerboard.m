#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

static BOOL isCheckerboard(const uint8_t *pixels, size_t offset) {
  int red = pixels[offset];
  int green = pixels[offset + 1];
  int blue = pixels[offset + 2];
  int minimum = MIN(red, MIN(green, blue));
  int maximum = MAX(red, MAX(green, blue));
  return minimum >= 225 && maximum - minimum <= 18;
}

static BOOL isNeutralHalo(const uint8_t *pixels, size_t offset) {
  int red = pixels[offset];
  int green = pixels[offset + 1];
  int blue = pixels[offset + 2];
  int minimum = MIN(red, MIN(green, blue));
  int maximum = MAX(red, MAX(green, blue));
  return pixels[offset + 3] > 0 && minimum >= 175 && maximum - minimum <= 22;
}

int main(int argc, const char *argv[]) {
  @autoreleasepool {
    if (argc != 3) {
      fprintf(stderr, "usage: remove-generated-checkerboard input.png output.png\n");
      return 2;
    }

    NSURL *input = [NSURL fileURLWithPath:[NSString stringWithUTF8String:argv[1]]];
    NSURL *output = [NSURL fileURLWithPath:[NSString stringWithUTF8String:argv[2]]];
    CGImageSourceRef source = CGImageSourceCreateWithURL((__bridge CFURLRef)input, NULL);
    CGImageRef image = source ? CGImageSourceCreateImageAtIndex(source, 0, NULL) : NULL;
    if (!image) {
      fprintf(stderr, "could not read input PNG\n");
      if (source) CFRelease(source);
      return 1;
    }

    size_t width = CGImageGetWidth(image);
    size_t height = CGImageGetHeight(image);
    size_t bytesPerRow = width * 4;
    uint8_t *pixels = calloc(height, bytesPerRow);
    uint8_t *transparent = calloc(width * height, 1);
    size_t *queue = malloc(width * height * sizeof(size_t));
    CGColorSpaceRef colorSpace = CGColorSpaceCreateDeviceRGB();
    CGContextRef context = CGBitmapContextCreate(
        pixels, width, height, 8, bytesPerRow, colorSpace,
        kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
    CGColorSpaceRelease(colorSpace);
    if (!pixels || !transparent || !queue || !context) {
      fprintf(stderr, "could not allocate image buffer\n");
      return 1;
    }
    CGContextDrawImage(context, CGRectMake(0, 0, width, height), image);

    size_t head = 0, tail = 0;
#define ENQUEUE(X, Y) do { \
      size_t pixel = (Y) * width + (X); \
      if (!transparent[pixel] && isCheckerboard(pixels, (Y) * bytesPerRow + (X) * 4)) { \
        transparent[pixel] = 1; \
        queue[tail++] = pixel; \
      } \
    } while (0)

    for (size_t x = 0; x < width; x++) {
      ENQUEUE(x, 0);
      ENQUEUE(x, height - 1);
    }
    for (size_t y = 0; y < height; y++) {
      ENQUEUE(0, y);
      ENQUEUE(width - 1, y);
    }
    while (head < tail) {
      size_t pixel = queue[head++];
      size_t x = pixel % width;
      size_t y = pixel / width;
      if (x > 0) ENQUEUE(x - 1, y);
      if (x + 1 < width) ENQUEUE(x + 1, y);
      if (y > 0) ENQUEUE(x, y - 1);
      if (y + 1 < height) ENQUEUE(x, y + 1);
    }
#undef ENQUEUE

    for (size_t index = 0; index < tail; index++) {
      size_t pixel = queue[index];
      size_t offset = (pixel / width) * bytesPerRow + (pixel % width) * 4;
      pixels[offset] = pixels[offset + 1] = pixels[offset + 2] = pixels[offset + 3] = 0;
    }

    size_t haloCount = 0;
    for (int pass = 0; pass < 3; pass++) {
      size_t passCount = 0;
      memset(transparent, 0, width * height);
      for (size_t y = 1; y + 1 < height; y++) {
        for (size_t x = 1; x + 1 < width; x++) {
          size_t pixel = y * width + x;
          size_t offset = y * bytesPerRow + x * 4;
          if (!isNeutralHalo(pixels, offset)) continue;
          BOOL touchesAlpha = pixels[offset - 1] == 0 || pixels[offset + 7] == 0 ||
                              pixels[offset - bytesPerRow + 3] == 0 ||
                              pixels[offset + bytesPerRow + 3] == 0;
          if (touchesAlpha) {
            transparent[pixel] = 1;
            passCount++;
          }
        }
      }
      for (size_t pixel = 0; pixel < width * height; pixel++) {
        if (!transparent[pixel]) continue;
        size_t offset = (pixel / width) * bytesPerRow + (pixel % width) * 4;
        pixels[offset] = pixels[offset + 1] = pixels[offset + 2] = pixels[offset + 3] = 0;
      }
      haloCount += passCount;
      if (!passCount) break;
    }

    CGImageRef cleaned = CGBitmapContextCreateImage(context);
    CGImageDestinationRef destination = CGImageDestinationCreateWithURL(
        (__bridge CFURLRef)output, (__bridge CFStringRef)UTTypePNG.identifier, 1, NULL);
    if (!cleaned || !destination) {
      fprintf(stderr, "could not create output PNG\n");
      return 1;
    }
    CGImageDestinationAddImage(destination, cleaned, NULL);
    BOOL saved = CGImageDestinationFinalize(destination);
    printf("removed %zu checkerboard and %zu halo pixels from %zux%zu PNG\n",
           tail, haloCount, width, height);

    CFRelease(destination);
    CGImageRelease(cleaned);
    CGContextRelease(context);
    CGImageRelease(image);
    CFRelease(source);
    free(queue);
    free(transparent);
    free(pixels);
    return saved ? 0 : 1;
  }
}
