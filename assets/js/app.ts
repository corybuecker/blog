import Prism from "prismjs";
import "prismjs/components/prism-bash";
import "prismjs/components/prism-yaml";
import "prismjs/components/prism-javascript";
import "prismjs/components/prism-docker";
import "prismjs/components/prism-nginx";
import "prismjs/components/prism-elixir";
import "prismjs/components/prism-css";

const localizeTimeElements = () => {
  const timeElements: HTMLCollectionOf<HTMLTimeElement> =
    document.getElementsByTagName("time");
  for (const el of timeElements) {
    const timeString = el.dateTime;
    el.innerText = new Date(timeString).toLocaleDateString();
  }
};

window.addEventListener("DOMContentLoaded", () => {
  localizeTimeElements();
  Prism.highlightAll(false);
});
