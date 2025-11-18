const PROTOCOL_INFO = {
  Martin1: {
    size: [320, 256],
  },
};

let socket = new WebSocket("/events");
let progress = document.querySelector("#progress");
let history = document.querySelector("#history");

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

    let canvas = image_container("Martin1", new Date());
    let rgb = await event.data.bytes();
    image_to_canvas(canvas, rgb);
  }
});

load_history();

function load_history(before) {
  let url = "/images?limit=15";
  if (before) url.push(`&before=${before}`);
  fetch(url)
    .then((r) => r.json())
    .then((d) => {
      for (let [idx, info] of Object.entries(d)) {
        let canvas = image_container(
          info.mode,
          new Date(info.timestamp * 1000),
        );
        fetch(`/image/${idx}`)
          .then((r) => r.bytes())
          .then((rgb) => {
            image_to_canvas(canvas, rgb);
          });
      }
    });
}

function image_container(mode, date, first) {
  let container = document.createElement("div");

  let info = PROTOCOL_INFO[mode];
  let canvas = document.createElement("canvas");
  canvas.width = info.size[0];
  canvas.height = info.size[1];
  container.appendChild(canvas);

  let text = document.createElement("p");
  text.innerText = `Received ${date.toLocaleString()}`;
  container.appendChild(text);

  if (first) history.insertBefore(container, history.firstChild);
  else history.appendChild(container);

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
