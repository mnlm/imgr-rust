## imgr

CLI tool to rename image files to their EXIF creation date.

Recursively traverses the specified path and tries to rename all images based on the date and time the image was taken.

EXIF fields are used in the following order of priority:

1. DateTimeOriginal - *The date and time when the original image data was generated*
2. DateTimeDigitized - *The date and time when the image was stored as digital data*
3. DateTime - *Date and time when the image file was created or last edited*

## Usage

By default images will be renamed using the format `%Y-%m-%d_%H-%M-%S`.

```
$ imgr images/
11:23:00 [INFO] images/picture.jpg - images/2026-01-01_01-01-01.jpg
[00:00:00] [###################################] 1/1   Finished renaming images
```

Specify a `--format` option to change this, see [documentation](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) for details.

```
$ imgr --format %Y-%m-%d images/
11:24:00 [INFO] images/picture.jpg - images/2026-01-01.jpg
[00:00:00] [###################################] 1/1   Finished renaming images
```

> [!NOTE]  
> If a file with the same name already exists it will be skipped. Non image files will also be skipped.