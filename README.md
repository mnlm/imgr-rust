## imgr

Rename image files to their EXIF creation date.

Recursively traverses the specified path and tries to rename all images based on the date and time the image was taken.

EXIF fields are used in the following order of priority:

1. DateTimeOriginal - *The date and time when the original image data was generated*
2. DateTimeDigitized - *The date and time when the image was stored as digital data*
3. DateTime - *Date and time when the image file was created or last edited*

By default images will be renamed using the format: `%Y-%m-%d_%H-%M-%S`. Specify a `--format` option to change this, see [documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for details.

If a file with this name already exists it will be skipped. Non image files will also be skipped.