let socket = new WebSocket("/events");
let progress = document.querySelector("#progress");

socket.addEventListener("message", async (event) => {
  if (typeof event.data == "string") {
    let [type, data] = event.data.split(":", 2);
    if (type == "decode_start") {
      console.log("Starting decode");
    } else if (type == "decode_progress") {
      progress.value = parseFloat(data) * 100;
    }
  } else {
    progress.value = 0;

    let rgb = await event.data.bytes();
    let rgba = new Uint8ClampedArray(rgb_to_rgba(rgb));
    let image = new ImageData(rgba, 320);

    let canvas = document.createElement("canvas");
    canvas.width = image.width;
    canvas.height = image.height;
    let ctx = canvas.getContext("2d");
    ctx.putImageData(image, 0, 0);
    document.querySelector("body")?.appendChild(canvas);
  }
});

function rgb_to_rgba(rgb) {
  let rgba = [];
  for (let i = 0; i < rgb.length / 3; i++) {
    let offset = 3 * i;
    rgba.push(rgb[offset]);
    rgba.push(rgb[offset + 1]);
    rgba.push(rgb[offset + 2]);
    rgba.push(255);
  }

  return rgba;
}
