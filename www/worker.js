import * as wasm from "consistent-hash-proof";

const APP = wasm.new_app();

onmessage = async (m) => {
    let [id, options] = m.data;
    let contents = await wasm.eval_message(APP, id, options);
    postMessage([id, contents]);
};

postMessage(["initialized", "yes"]);
