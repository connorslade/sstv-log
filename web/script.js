const PROTOCOL_INFO = {
  Martin1: {
    size: [320, 256],
  },
};

let socket = new WebSocket("/events");
let progress = document.querySelector("#progress");

socket.addEventListener("message", async (event) => {
  if (typeof event.data == "string") {
    let [type, data] = event.data.split(":", 2);
    if (type == "decode_start") {
      console.log(`Starting decode of ${data}`);
    } else if (type == "decode_progress") {
      progress.value = parseFloat(data) * 100;
    }
  } else {
    progress.value = 0;

    let canvas = canvas_for("Martin1");
    let rgb = await event.data.bytes();
    image_to_canvas(canvas, rgb);

    document.querySelector("body")?.appendChild(canvas);
  }
});

fetch("/images")
  .then((r) => r.json())
  .then((d) => {
    let history = document.querySelector("#history");
    for (let [idx, info] of Object.entries(d)) {
      let container = document.createElement("div");

      let canvas = canvas_for(info.mode);
      container.appendChild(canvas);

      let text = document.createElement("p");
      text.innerText = `Received ${new Date(info.timestamp * 1000).toLocaleString()}`;
      container.appendChild(text);

      history.appendChild(container);

      fetch(`/image/${idx}`)
        .then((r) => r.bytes())
        .then((b) => {
          image_to_canvas(canvas, b);
        });
    }
  });

function canvas_for(mode) {
  let info = PROTOCOL_INFO[mode];
  let canvas = document.createElement("canvas");
  canvas.width = info.size[0];
  canvas.height = info.size[1];

  return canvas;
}

function image_to_canvas(canvas, rgb) {
  let rgba = new Uint8ClampedArray(rgb_to_rgba(rgb));
  let image = new ImageData(rgba, 320);

  let ctx = canvas.getContext("2d");
  ctx.putImageData(image, 0, 0);
}

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
