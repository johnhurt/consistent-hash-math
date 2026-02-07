import * as wasm from "consistent-hash-proof";

const APP = wasm.new_app();

onmessage = async (m) => {
    let [id, options] = m.data;
    console.log("message", options);
    let contents = await wasm.eval_message(APP, id, options);
    postMessage([id, contents]);
};

console.log("worker started");
postMessage(["initialized", "yes"]);