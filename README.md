<p align="center">
    <img src="./assets/Podium.svg">
</p>


# Podium

Podium is a smart indexer and searcher for your files.

A search engine doesn't expect you to know the name of the website you're searching for, you only search for the content and it does the work of finding the website.
Podium is the same, you don't need to know the name of the file you're looking for, or where you saved it. You only need to know what it's about. Podium does the work of figuring out where it is.

Podium doesn't interfere with how you already organize your files, but it can help you when you're stuck.

## Features

* **Private** - Your files and data never leave your computer
* **Flexible** - Works on all desktop environments
* **Fast** - New files are processed in under 1 second
* **Extensible** - Built with plugins at its core
* **Smart** - Uses modern AI models to accurately identify the content of your files

### Supported file types

| Type                     | Extensions                                       |
|--------------------------|--------------------------------------------------|
| Image - object detection | .tif, .tiff, .jpg, .jpeg, .png, .bmp, .ico, .gif |
| Image - exif metadata    | .tif, .tiff, .jpg, .jpeg                         |
| Spreadsheed              | .csv, .xlsx                                      |
| Text                     | .txt, .docx                                      |
| Slideshow                | .pptx                                            |
| PDF                      | .pdf                                             |


### Performance

| File          | Type                     | Processing time (avg) | File Size |
|---------------|--------------------------|-----------------------|-----------|
| [Cats.pdf](https://github.com/teovoinea/podium/blob/master/test_files/Cats.pdf)      | PDF                      | 26 ms                 | 21 KB     |
| [Cats.pptx](https://github.com/teovoinea/podium/blob/master/test_files/Cats.pptx)     | Slideshow                | 5 ms                 | 33 KB     |
| [Cats.xslx](https://github.com/teovoinea/podium/blob/master/test_files/Cats.xlsx)     | Spreadsheet              | 210 us                | 9.2 KB    |
| [IMG_2551.jpeg](https://github.com/teovoinea/podium/blob/master/test_files/IMG_2551.jpeg) | Image - object detection | 306 ms                | 1.7 MB    |
| [IMG_2551.jpeg](https://github.com/teovoinea/podium/blob/master/test_files/IMG_2551.jpeg) | Image - exif metadata    | 1.62 ms               | 1.7 MB    |
| [data.csv](https://github.com/teovoinea/podium/blob/master/test_files/data.csv)      | Spreadsheet              | 21 us                 | 379 B     |
| [file.txt](https://github.com/teovoinea/podium/blob/master/test_files/file.txt)      | Text                     | 200 ns               | 39 B      |

Processing time was measured on a 2018 13" MacBook Pro and is intended only to give you an idea of how podium will run on your own machine.
