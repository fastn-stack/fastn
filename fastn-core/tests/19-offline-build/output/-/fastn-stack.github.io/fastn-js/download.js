// Default download format is kept as .jpeg
// To download as other formats, use other functions mentioned below
function download_as_image(element_id, filename) {
    // Get the HTML element you want to convert to an image
    var element = document.getElementById(element_id);

    // Use htmlToImage library to convert the element to an image
    htmlToImage.toJpeg(element)
      .then(function (dataUrl) {
        // `dataUrl` contains the image data in base64 format
        var link = document.createElement('a');
        link.download = filename;
        link.href = dataUrl;
        link.click();
      })
      .catch(function (error) {
        console.error('Error downloading image:', error);
      });
}

function download_as_jpeg(element_id, filename) {
    var element = document.getElementById(element_id);

    htmlToImage.toJpeg(element)
      .then(function (dataUrl) {
        var link = document.createElement('a');
        link.download = filename;
        link.href = dataUrl;
        link.click();
      })
      .catch(function (error) {
        console.error('Error downloading image:', error);
      });
}

function download_as_png(element_id, filename) {
    var element = document.getElementById(element_id);

    htmlToImage.toPng(element)
      .then(function (dataUrl) {
        // `dataUrl` contains the image data in base64 format
        var link = document.createElement('a');
        link.download = filename;
        link.href = dataUrl;
        link.click();
      })
      .catch(function (error) {
        console.error('Error downloading image:', error);
      });
}

function download_as_svg(element_id, filename) {
    var element = document.getElementById(element_id);

    htmlToImage.toSvg(element)
      .then(function (dataUrl) {
        var link = document.createElement('a');
        link.download = filename;
        link.href = dataUrl;
        link.click();
      })
      .catch(function (error) {
        console.error('Error downloading image:', error);
      });
}

function download_text(filename, text) {
    const blob = new Blob([fastn_utils.getStaticValue(text)], { type: 'text/plain' });
    const link = document.createElement('a');
    link.href = window.URL.createObjectURL(blob);
    link.download = filename;
    link.click();
}
