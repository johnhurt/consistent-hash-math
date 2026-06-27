import rehypeKatex from 'rehype-katex'
import html from 'rehype-stringify'
import remarkMath from 'remark-math'
import remarkParse from 'remark-parse'
import remarkRehype from 'remark-rehype'
import rehypeRaw from 'rehype-raw'
import { unified } from 'unified'
import 'github-markdown-css'
import 'katex/dist/katex.min.css'

import markdownContents from './contents.md'

const file = await unified()
  .use(remarkParse)
  .use(remarkMath)
  .use(remarkRehype, { allowDangerousHtml: true })
  .use(rehypeKatex)
  .use(rehypeRaw)
  .use(html)
  .process(markdownContents);

document.getElementById('markdown-body').innerHTML = file.value;

let worker;
let workerReady = false;
let pendingDiagrams = 0;

function showSpinner() {
  pendingDiagrams++;
  const spinner = document.getElementById("spinner");
  if (spinner) {
    spinner.classList.add("active");
  }
}

function hideSpinner() {
  pendingDiagrams = Math.max(0, pendingDiagrams - 1);
  if (pendingDiagrams === 0) {
    const spinner = document.getElementById("spinner");
    if (spinner) {
      spinner.classList.remove("active");
    }
  }
}

if (window.Worker) {
  worker = new Worker(new URL("./worker.js", import.meta.url));

  worker.onmessage = (r) => {
    const [id, contents] = r.data;

    if (id == "initialized") {
      workerReady = true;
      return;
    }

    hideSpinner();

    const e = document.getElementById(id + "-diagram");
    if (!e) {
      return;
    }

    e.innerHTML = contents;
  };
}

function getInputsForDiv(id) {
  let d = document.getElementById(id);
  let result = {};

  if (!d) {
    return;
  }

  let to_search = [d];

  while (to_search.length > 0) {
    const curr = to_search.pop();

    if (curr.name) {
      const parsed = parseFloat(curr.value);

      if (typeof parsed === "number" && isNaN(parsed) === false) {
        result[curr.name] = parsed;
      }
      else if (curr.value == "true") {
        result[curr.name] = true;
      }
      else {
        result[curr.name] = curr.value;
      }


    } else {
      curr.childNodes.forEach(c => to_search.push(c));
    }
  }

  return result;
}

// Update a slider's displayed output value. Used in oninput handlers where
// a plain `this.nextElementSibling.value = this.value` would be verbose.
window.updateOutput = function(input) {
  input.nextElementSibling.value = input.value;
}

// Keep a dependent slider's value and max clamped to the value of a primary
// slider. Call from the primary slider's oninput with both elements.
window.clampDependent = function(primary, dependent) {
  dependent.max = primary.value;
  dependent.value = Math.min(+dependent.value, +primary.value);
  window.updateOutput(dependent);
}

function isDarkMode() {
  return window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
}

function rerunDiagram(id) {
  let inputs = getInputsForDiv(id);
  inputs.dark_mode = isDarkMode();
  showSpinner();
  worker.postMessage([id, JSON.stringify(inputs)])
}


function initialize() {
  for (const d of document.getElementsByClassName("diagram-container")) {
    let diagram_div = d.getElementsByClassName("diagram")[0];
    let id = d.id;

    diagram_div.id = id + "-diagram";

    let b = d.getElementsByTagName("button")[0];

    if (b) {
      b.addEventListener("click", () => rerunDiagram(id));
    }

    let inputs = d.getElementsByTagName("input");
    for (const i of inputs) {
      i.addEventListener("change", () => rerunDiagram(id));
    }
  }

  return true;
}

function run_first_diagram_load() {
  if (!workerReady) {
    return false;
  }

  for (const d of document.getElementsByClassName("diagram-container")) {
    rerunDiagram(d.id);
  }

  return true;
}


function run_first_diagram_load_with_retry() {
  if (run_first_diagram_load()) {
    return;
  }
  setTimeout(() => {
    run_first_diagram_load_with_retry();
  }, 10);
}

// Function to handle theme change logic
function handleThemeChange() {
  for (const d of document.getElementsByClassName("diagram-container")) {
    let id = d.id;
    rerunDiagram(id);
  }
}

// Get the MediaQueryList object for the dark mode preference
const darkModePreference = window.matchMedia("(prefers-color-scheme: dark)");

// Add an event listener to the MediaQueryList object to watch for changes
if (darkModePreference.addEventListener) {
  darkModePreference.addEventListener("change", handleThemeChange);
}

initialize()
run_first_diagram_load_with_retry();
