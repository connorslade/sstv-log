let socket = new WebSocket("/events");

socket.addEventListener("message", (event) => {
  console.log(event);

  if (typeof event.data == "string") {
    let [type, data] = event.data.split(":", 2);
    if (type == "decode_start") {
      console.log("Starting decode");
    } else if (type == "decode_progress") {
      let progress = parseFloat(data) * 100;
      document.querySelector("#progress").value = progress;
    }
  } else {
    window.imageData = event.data;

    console.log("got image data");
    let image = new ImageData(new Uint8ClampedArray(event.data), 320);

    let canvas = document.createElement("canvas");
    canvas.width = image.width;
    canvas.height = image.height;
    let ctx = canvas.getContext("2d");
    ctx.putImageData(image, 0, 0);
    document.querySelector("body")?.appendChild(canvas);
  }
});
